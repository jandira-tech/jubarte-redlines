# jubarte for contract redlining: assessment and scored proposals

Planning only. Nothing here was implemented. Written 2026-10-18 against
`origin/main` at `b86fe5ac` (the bench binary is `2b9594bd`, one commit
earlier), the open issues, the unmerged branches, the case folders beside
this file, and the Word fixtures `Step_0..Step_2` in the repository root.

## How to read the scores

Every proposal lists alternatives. Each alternative carries three floats in
`[0, 1]`, higher is better on all three:

- **desirability**: value to the two users of the brief (the lawyer in
  Word, the agent on a clock), net of what it costs them.
- **best practices**: it copies a shape Word itself writes, keeps a strict
  and deterministic contract, is testable without the benchmark, and never
  does something silently.
- **handrolling**: 1.0 means no bespoke machinery (Word's own shape, an
  existing engine path, a standard library); 0.0 means a new engine or a
  new semantic that only jubarte has. If you meant "amount of hand-rolling"
  instead, invert this column.

The recommended alternative is always listed first and marked **(R)**.

---

## 0. Corrections to the brief

Arthur, several statements in the brief are wrong or already settled. I am
listing them before the proposals so the proposals do not inherit them.

1. **`BOOKMARK_CROSSES_REVISION` does not exist in 0.12.0.** The string is
   absent from `src/` on `origin/main`, from the `v0.12.0` tag, and from the
   deployed bench binary (`strings bin/jubarte | grep -c CROSSES_REVISION`
   gives 0). It exists only on the unmerged V2 stack (`feat/v2-08`,
   `42cabd10`, PreparedEdit's `BLOCKING_CODES`) and in agent-written reports
   archived in the bench on 2026-10-07. The swallowed-bookmark *shape* is
   real (#1156 shows the XML on `2b9594bd`), but the claim "validate reports
   `BOOKMARK_CROSSES_REVISION` and the input validated clean" is not from this
   binary. Re-run `jubarte validate` on the s1 output with the deployed binary
   and record the code it actually prints before the proposal below is
   scoped. The fix (markers stay beside the insertion) stands either way.
2. **Problem 1 is already implemented, unmerged.** Branch
   `fix/edit-inside-ins-1136` (`c7cddbc1`, `80497cde`, `a05f4697`,
   `a4467b98`) splits another author's insertion around a nested `w:del`,
   edits the editor's own insertion in place, and carries a 654-line test
   file. The brief treats it as open. The work below assumes that branch
   lands first and builds on it.
3. **Problem 10's Word experiment is already done.** #1159 records that
   Word also deletes the thread when the deletion holding the reference is
   accepted. The open part is reporting and reply consistency, not Word's
   behaviour.
4. **Problem 21 is probably Word-faithful.** The counterparty deleted p3's
   paragraph mark, so p3 and whatever follows it merge on Accept All. A new
   paragraph "after p3" placed after that deleted mark is exactly what Word
   produces when you press Enter there (to be confirmed, experiment E3
   below). The defect is that jubarte did not say so.
5. **"Settling only yields no tracked change by our author" is correct.**
   A round whose right answer is "accept everything" has no tracked change
   by us. The validity gate is wrong; jubarte must not fake a change.
6. **The skill wording "one or two jubarte passes" is not in this
   repository.** It is in the bench's `instruction.md` (lines 117 and 183)
   and `SKILL.md`. The fix belongs to the skill template in
   `~/temp/T/jubarte-syntax/skill-tools/`, not to `skills/jubarte-documents/`.
7. **`read --changed --by AUTHOR` already exists** (`src/cli.rs:118-137`).
   What is missing is the summary and the outline, not the filter.
8. **Problem 16's cause is located**: `revision_processor.rs:3493-3544`
   renumbers annotation ids whenever no revision is left after a settle, so a
   bare `accept`/`reject` renumbers comments, and a targeted one does not.
9. **`validate --rules edits --original X --author Y` already exists**;
   `--baseline` is an extension of it, and #1192 is the same defect as the
   brief's item 23 (FOREIGN_AUTHOR flags the original's revisions).
10. **Open issues already cover five items**: #1108 (hint wording), #1158
    (`--no-page-markers` with `--json`), #1156 (bookmark end swallowed),
    #1159 (comment loss unreported), #1136 (nested edits). The proposals
    cite them; none should be re-filed.

---

## A. Prioritized change list

| rank | change | benefit | effort |
|---|---|---|---|
| 1 | Land #1136: edits inside another author's insertion nest our `w:del`; own insertions edit in place; paragraph delete under `keep` | removes three quarters of all refusals; Word's own shape | M (done, unmerged) |
| 2 | `strike` and `restore` operations: a visible reject of their insertion, a visible restore of their deletion; `reject` keeps Word's meaning | the other side sees what we rejected; 120 rubric weight was lost to invisible settles, but the real win is provenance | M |
| 3 | `edit --skip-failed`: write the ops that passed, write `PLAN.failed.json` with the rest and each fix; dry-run prints the apply command; every write prints the new sha256 | work is never lost at the time limit; plans stay atomic by default | M |
| 4 | Fix the refusal hints: name both decisions and their cost, quote the active plan syntax, offer `strike` first (#1108) | agents stop settling silently because the error told them to | S |
| 5 | Drop the gratuitous option conflicts; with `--json`, usage errors are a JSON object on stdout (#1158) | 165 blind retries in 78 trials vanish | S |
| 6 | `validate --baseline INPUT` and warnings printed once, not on every `read`/`changes`/`comments` | agents stop chasing findings they did not cause | S/M |
| 7 | Keep bookmark, comment and permission markers beside a new insertion, never inside it (#1156) | no range drift; Accept/Reject keep the bookmark where the source had it | S |
| 8 | Plan schema errors that teach: valid fields, nearest field, line; three safe aliases | first-try plans | S |
| 9 | Summary-first reading: `changes --summary`, `read --outline`, `changes --brief` | a 1,000-revision document is planned from two screens instead of ten pages | M |
| 10 | Report every comment a settle removes; keep replies consistent (#1159); reply to a comment in tracked-deleted text is allowed | no orphaned threads, no silent loss | S |
| 11 | One notion of a change: `changes` groups what `read` groups (`r7+8+9+10`), `-p r87+r88` accepted | rejecting half an insertion stops happening | S |
| 12 | `replace` keeps the formatting of the characters it keeps; lenient anchors never rewrite typography | no un-bolded defined terms, no curly-quote churn | M |
| 13 | Insert into an empty paragraph | ordinary Word action | S |
| 14 | Comment ids are stable across any settle; another author's comment is not editable without an explicit flag | replies land in the right thread; no re-attribution | S |
| 15 | `insert_paragraph` with `style` copies nothing else; structural ops are emitted structurally in every mode | no "4." stealing the article numbering, no quote marks hopping paragraphs | M |
| 16 | Numbering: fresh `durableId` on every cloned `w:num`; untouched paragraphs keep their `numId` | no duplicate ids, no 17 phantom formatting changes | M |
| 17 | View fixes: never pair a deletion and an insertion by different authors; always print `on rN` for a nested change | the agent sees whose text it is | S |
| 18 | `PARAGRAPH_RESOLVED` and the other settled-id messages name both ids and both views | one fewer re-read | S |
| 19 | A new paragraph beside a pending-deleted mark is placed as Word places it and the report says what Accept All will do | no surprise merge | S |
| 20 | `changes --json` keeps tabs; `validate --original` ignores revisions the original already had | small, both found in the repro | S |
| 21 | Revision-aware compare as diff → plan → `keep` emission (three-way) | a second safe workflow for people who edit plain text; not a replacement for 1 | L |
| 22 | Help: `jubarte help plan` prints the schema and one example per kind; the 63 KB help-all becomes a reference, not the entry point | 21-25 minutes to the first edit is mostly reading | S |
| 23 | Skill template: small batches, apply after each, strike before reject, what to do at ten minutes left | belongs with the bench skill, not the engine | S |

---

## A-detail. The changes

### 1. Edits inside another author's insertion (land #1136)

**Problem.** 111 of 146 refusal codes in 0.12.0 are
`UNSUPPORTED_STRUCTURE: the text sits inside rN, a tracked insertion by
"…"; settle it first`. Five 0.12.0 trials ended with nothing delivered
because the first edit attempt hit it late (`B-final-blow-late-start/`).
It also refuses the agent's own insertion (s3-t4-g13a, 8 new-build trials)
and a `delete_paragraph` of a paragraph that holds any revision under
`keep` (`src/edit/tracked.rs:94-104`). The refusal comes from
`check_range` and `check_insert_position` in `src/edit.rs:3863-3985`,
which treat any `w:ins`, `w:moveTo` or `w:del` ancestor as opaque.

**Who benefits.** The agent: the most common refusal disappears. The
lawyer: the document carries the same XML Word would have written had a
reviewer typed the edit, so Word's review pane shows it as a nested change,
attributed correctly.

**Design.** No new flag or field. Under `existing_revisions: keep` (and
under the v2 default), a range inside another author's `w:ins` is
editable. The emitter writes Word's shape, which the `Step_2` fixture
pins (Bob replaces "bankers" inside Arthur's pending insertion, Word 16,
tracking on):

```xml
<w:ins w:id="3" w:author="Arthur Souza Rodrigues" w:date="…"><w:r><w:t>agree to pay the </w:t></w:r>
  <w:del w:id="4" w:author="Bob" w:date="…"><w:r><w:delText>bankers</w:delText></w:r></w:del>
</w:ins>
<w:ins w:id="5" w:author="Bob" w:date="…"><w:r><w:t>insurers</w:t></w:r></w:ins>
<w:ins w:id="6" w:author="Arthur Souza Rodrigues" w:date="…"><w:r><w:t> a pro-rata amount…</w:t></w:r></w:ins>
```

- the deleted words stay inside their insertion as our `w:del`;
- our new words are a sibling `w:ins` after it;
- the remainder of their insertion is a new `w:ins` with their author and
  date and a **new `w:id`** (Word numbers each piece; the branch commit
  `23c11a3d` does the same). The view and `changes` present the pieces as one
  logical change (`r3+6`, see change 11), so `accept r3+6` still resolves all
  of Alma's words. Keeping one `w:id` on several elements would trip
  `DUPLICATE_REVISION_ID` and is not what Word writes.
- the editor's own pending insertion is edited in place: text changes,
  no nested mark (Word treats typing in your own insertion as a correction).
- `delete_paragraph` under `keep` of a paragraph holding revisions: plain
  runs get our `w:del`; another author's `w:ins` runs get a nested `w:del`;
  their `w:del` runs stay as they are; our own `w:ins` runs are removed
  outright; the paragraph mark gets `w:del` in its `w:rPr`.
- comments anchored in the range stay (Word keeps comments on deleted
  text); bookmarks stay where they are; fields are deleted whole or not at
  all (the existing field-crossing refusal stays).

**Word fidelity.** #1134 opened this shape in Word: no repair prompt,
Accept All and Reject All give the right text. Reject All on our author
alone restores Alma's insertion intact (our nested `w:del` and sibling
`w:ins` are removed; the split pieces remain hers). The only unprobed
shape is the paragraph mark: when their inserted mark is deleted by us, I
expect `w:rPr` to carry both `w:ins` and `w:del` (experiment E1).

**Edge cases and errors still refused.**

- Inside another author's `w:del`: Word does not let you type into deleted
  text. Refuse with: `r12 is text "Trainer 1 - LargeCo" deleted; to put it
  back visibly use {"kind":"restore","revision":"r12"}; to accept the
  deletion use resolve_revisions.accept; to comment on it use comment`.
- Inside `w:moveTo`/`w:moveFrom`: refuse until experiment E2 says what
  Word writes (I believe a nested `w:del` inside `w:moveTo` is allowed, but I
  have not seen Word produce it).
- A field result inside their insertion: existing refusal, unchanged.

**Verification.** `tests/edit_inside_insertion.rs` already on the branch;
add: the own-insertion case, the mark case once E1 is done, a property
test on the `Step_2` fixture family: for every edit, `reject(ours)` ==
source text and revisions by the other author (compare `changes` output
before and after), `accept_all(output)` == accept_all of the plain edit.

**Effort and risk.** Medium, mostly spent. Risk: the split renumbers
pieces, so an id noted before the write is stale after it; the report's
`paragraph_ids()`/revision mapping (B11) must list the split.

**Alternatives.**

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** Word's nested shape, own insertion edited in place (the branch) | 0.95 | 0.95 | 0.95 |


- **A pros:** removes the refusal class; the file is indistinguishable
  from a Word session. **Cons:** ids split; the view must show nesting
  (change 17) or the agent cannot see whose text it struck.
  Dummy: `edit c.docx -p p0 --anchor bankers --content insurers -a Bob`
  → the XML above.

### 2. `strike` and `restore`: visible rejects

**Problem.** `resolve_revisions.reject` does what Word's Reject button
does: the counterparty's insertion vanishes, their deletion silently comes
back as plain text. Ten delivered 0.12.0 documents (120 rubric weight)
show the right text with no trace of the decision (`C-invisible-settle/`).
The refusal in change 1 steers to settling, so agents settle. The
`compare` experiment shows the mirror image: a two-way compare makes our
rejects visible and our acceptances invisible.

**Who benefits.** The lawyer on the other side sees "you struck our
clause" as a strikethrough under our name, with their insertion still
attributed to them, and can Reject our change to put it back. The agent
gets a one-line way to say "no" that leaves evidence.

**Design.** Two new operation kinds, both under `operations` because they
are tracked changes *by our author*:

```json
{"kind": "strike",  "revision": "r366", "comment": "Not acceptable: parents are not parties."}
{"kind": "restore", "revision": "r12",  "comment": "This carve-out was agreed in round 1."}
```

CLI: `jubarte edit FILE -p r366 --strike [--content "comment"]` and
`-p r12 --restore`. `reject --visible` is accepted as a spelling of
`strike`/`restore` chosen by the revision's kind, so an agent that reaches
for the Word verb still gets the visible form. `resolve_revisions.reject`
is unchanged: it means Word's Reject.

OOXML for `strike` of their insertion:

```xml
<!-- before -->
<w:ins w:id="366" w:author="Trainer 1 - LargeCo" w:date="2026-10-02T09:00:00Z">
  <w:r><w:t>and each of their respective parents</w:t></w:r></w:ins>
<!-- after -->
<w:ins w:id="366" w:author="Trainer 1 - LargeCo" w:date="2026-10-02T09:00:00Z">
  <w:del w:id="901" w:author="Reviewing Counsel (AgentCo)" w:date="2026-10-18T10:00:00Z">
    <w:r><w:delText>and each of their respective parents</w:delText></w:r></w:del></w:ins>
```

OOXML for `restore` of their deletion:

```xml
<!-- before -->
<w:del w:id="12" w:author="Trainer 1 - AgentCo" w:date="…"><w:r><w:rPr><w:b/></w:rPr><w:delText>use, store, copy</w:delText></w:r></w:del>
<!-- after: their deletion untouched, our insertion right after it, same rPr -->
<w:del w:id="12" …>…</w:del>
<w:ins w:id="902" w:author="Reviewing Counsel (LargeCo)" w:date="…"><w:r><w:rPr><w:b/></w:rPr><w:t>use, store, copy</w:t></w:r></w:ins>
```

Handling of the affected range:

- **moves**: `strike` of a `w:moveTo` nests `w:del` inside it (pending E2);
  `restore` of a `w:moveFrom` is refused: restoring moved text duplicates it,
  the agent should reject the move.
- **paragraph marks**: an inserted mark inside the struck range gets
  `w:del` beside their `w:ins` in `w:rPr` (pending E1); a deleted mark to
  restore gets a new paragraph whose mark carries our `w:ins` (pending E4).
- **numbering, pPrChange**: untouched; `strike` is about text.
- **tables**: a row they inserted (`w:trPr/w:ins`) gets `w:trPr/w:del` by
  us; cells are not struck singly.
- **fields**: struck whole, with begin/end inside the `w:del`.
- **bookmarks**: unmoved. **comments** anchored in the range: unmoved
  (Word keeps them on deleted text).

**Word fidelity.** Both shapes are what Word writes when a reviewer with
tracking on deletes inside another's insertion or retypes deleted text.
Accept All: strike → text gone; restore → text once. Reject All: strike →
text gone (it was never in the base); restore → text once. Reject only our
author: their change is pending again, intact. Accept only their author:
strike → our plain deletion pending; restore → their deletion applied, our
insertion pending. All four are the right legal reading.

**Edge cases and errors.**

- `strike` of a revision that is partly our own: refuse, `r40 is by
  "Reviewing Counsel (AgentCo)"; delete the text instead`.
- `strike` of a revision whose insertion already holds a nested deletion
  by us: no-op with a note, not an error.
- `restore` of a deletion whose text has a comment reference only (the
  s1-t3-g15a r13 case): refuse, `r13 holds no text, only comment c4's
  mark; reject r13 to keep c4`.
- A revision listed in both `resolve_revisions` and `strike`: existing
  `REVISION_CONFLICT`.

**Verification.** Fixtures from the `Step_2` family plus one real input
per case folder. Properties: (1) `reject_by(ours)(strike(x)) ≡ x` as XML
of the other author's elements; (2) `accept_all(strike(ins)) ≡
accept_all(reject(ins))` as text; (3) `accept_all(restore(del)) ≡
accept_all(reject(del))` as text; (4) every output opens in Word with no
prompt (`word_pdf.py` on the fixture set, once per release).

**Effort and risk.** Medium. The emitter already writes nested `w:del`
after change 1; `restore` is a copy of runs with `w:delText` → `w:t`. Risk:
marks and moves until E1/E2/E4 are done, so ship text-only first and refuse
the rest with the exact alternative.

**Alternatives.**

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** `strike`/`restore` ops; `reject --visible` as a spelling; `reject` unchanged | 0.95 | 0.95 | 0.95 |

- **A pros:** the op lives where our tracked changes live, takes a
  `comment`, composes with everything else; Word's verbs keep Word's
  meanings. **Cons:** two more kinds to document. Dummy: the JSON above.


### 3. Partial application, the apply command, and the new digest

**Problem.** A plan is all-or-nothing, so one bad op means nothing is
written; agents grew one plan for 30 minutes, dry-ran it clean six times
(s1-t2-g02c: nine clean dry-runs, one write at 60.3 min) and never applied
it. The skill says both "nothing was written" and "only the failed one
needs fixing", which reads two ways. A clean dry-run prints the report and
nothing else (`run_edit`, `src/bin/jubarte.rs:953-955`).

**Who benefits.** The agent: work lands as it is made. The lawyer: the
document on disk is always a coherent document, because every op that
landed was independently valid.

**Design.**

- `edit FILE --plan P --skip-failed`: apply every op that passes; the ops
  that fail are left out; exit 0 if at least one op landed, with the
  report marking each op `ok`/`skipped` and a stderr line
  `skipped 2 of 14 operations; the fixes are in P.failed.json`.
  `P.failed.json` is a complete plan (same author, date, `source_sha256`
  of the **new** output) holding only the failed ops, each with a `"why"`
  field the parser ignores. The agent fixes and re-runs it.
- Default stays atomic. `--skip-failed` is explicit because a skipped op
  can leave a legally incoherent pair (cap changed in 8.1, carve-out not
  changed in 8.2); the report says which.
- An optional `"group": "cap"` on ops: ops of one group stand or fall
  together under `--skip-failed`. An op with a `comment` is one unit
  already.
- After a clean `--dry-run`, print on stderr: `next: jubarte edit
  contract.docx --plan plan.json -o contract.docx --force` (the same argv
  without `--dry-run`).
- After every write, print `sha256: <new>` on stderr and put it in the
  report's summary event, so the next plan's `source_sha256` needs no
  `read`.

Why partial application is safe at op granularity: `check_conflicts`
already refuses overlapping ranges, double deletes and dependent targets,
so the ops that pass are pairwise independent by construction, and
`resolve_revisions` is applied as one unit before them. What it cannot
guarantee is legal coherence across sections; that is what the report and
`group` are for.

**Word fidelity.** Unaffected: each landed op is the same XML as today.

**Edge cases and errors.** `resolve_revisions` that fails (unknown id)
fails the whole plan even under `--skip-failed`, because the ops were
resolved against the settled text. `STALE_SOURCE` is never skipped. A plan
where every op failed exits 3 and writes nothing.

**Verification.** Unit: a 5-op plan with op 3 bad → output equals the
4-op plan's output byte for byte; `P.failed.json` applies cleanly on the
output. Property: `apply(skip_failed(P)) ∘ apply(P.failed fixed)` ≡
`apply(P fixed)` for random disjoint op sets.

**Effort and risk.** Medium. `preview_plan` already runs the whole
pipeline in memory; the change is to keep going past the first failure and
to serialize the leftover plan. Risk: the report format gains a status
value; v2's result protocol should adopt the same word.

**Alternatives.**

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** `--skip-failed` + `P.failed.json` + `group` + `next:` line + `sha256:` after write | 0.90 | 0.90 | 0.90 |


- **A pros:** nothing lost, nothing silently reordered, the failed plan is
  ready to run. **Cons:** an opt-in the agent must know; the skill must
  show it once. Dummy: `edit c.docx --plan p.json -o c.docx --force
  --skip-failed` → `wrote c.docx … skipped 1 of 9 operations; the fixes are
  in p.failed.json`.


### 4. Refusal hints that name both decisions and the visible route (#1108)

**Problem.** The hint says `settle it first ("resolve_revisions":
{"accept": {"ids": ["r366"]}}, or reject), or comment on it`. It names
accept first, says nothing of the cost (accepting makes their words plain
text, so Reject All later keeps them), quotes v1 syntax to v2 plans, and
never mentions the visible route.

**Who benefits.** The agent decides at the refusal; the lawyer gets
decisions made with their consequences stated.

**Design.** Once change 1 lands, this hint only fires for deletions, moves
and fields. Template:

```
UNSUPPORTED_STRUCTURE (op-2): the text is inside r12, text "Trainer 1 - LargeCo" deleted.
  put it back visibly:  {"kind":"restore","revision":"r12"}            (their deletion stays, your insertion shows)
  let it go:            "resolve_revisions":{"accept":{"ids":["r12"]}} (their words leave the document for good)
  put it back silently: "resolve_revisions":{"reject":{"ids":["r12"]}} (their deletion disappears, no trace)
  or comment on it:     {"kind":"comment","paragraph":"p160","find":"…","text":"…"}
```

The fragment is built from the active plan version (v1 vs v2 shapes).

**Alternatives.**

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** one message, three routes, consequences, active syntax | 0.90 | 0.90 | 1.00 |

- **A pros:** the decision is made where it is needed. **Cons:** four
  lines on stderr. 

### 5. Option conflicts, and JSON-shaped usage errors (#1158)

**Problem.** `read --no-page-markers --json` is refused 165 times in 78
trials; the skill's quick start teaches the flag and the exact-text recipe
teaches `--json`. The refusal arrives as a clap usage error on stderr; with
`2>&1 | jq` it is a JSON parse error, so the agent retries blind.
`--tables` with `-p` (9 times) and `--track-changes` with `--json` (7).

**Design.**

- `--no-page-markers` with `--json` or `--tables`: accepted, no-op, help
  line says so (issue #1158 option A). Byte-identical output.
- `--tables -p t0`: works (select the table), closes #1161/#1173.
- `--track-changes accepted|rejected` with `--json`: works; the snapshot's
  paragraphs carry `text` as the chosen settled view and the summary
  records `track_changes: accepted`. Refusing it threw away a useful read.
- With `--json` on any command, a usage or runtime error is printed on
  stdout as one line `{"error": {"code": "USAGE", "message": "…",
  "usage": "jubarte read [OPTIONS] <FILE>"}}` and the same text on stderr;
  exit code unchanged. A parser then sees the error instead of nothing.
- Audit every `conflicts_with` in `src/cli.rs` (lines 84-137, 2458-2560):
  keep only those where both options would make the output ambiguous.

**Alternatives.**

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** no-op the harmless flags, make the rest meaningful, JSON error on stdout with `--json` | 0.90 | 0.85 | 0.95 |


- **A cons:** a flag that does nothing in one mode is a small lie; the
  help line must say "ignored with --json". Dummy: `read c.docx
  --no-page-markers --json | jq .source_sha256` → the digest.


### 6. `validate --baseline`, and warnings once

**Problem.** `COMMENT_METADATA_MISMATCH` (missing `commentsExtensible`
durableId) is in the inputs; the skill demands `no findings`, so agents
spend minutes in XML. `read`, `changes` and `comments` repeat the input's
warnings on every call (23 trials). `validate --original INPUT --author US`
on an untouched turn-2 input reports 518 `FOREIGN_AUTHOR` (#1192).

**Design.**

- `validate OUT --baseline IN`: run both, match findings by `(code,
  part, entity id or paragraph id, message with ids normalised)`, print
  `new:` findings first, then `already in the input: N (not yours)`; exit 2
  only for new findings; `--json` splits them into `new`, `baseline`,
  `resolved`. The `edits` rule set under `--baseline` treats a revision
  present in the input as not foreign.
- `read`/`changes`/`comments`: warnings go to one stderr line, `input
  has 2 validity warnings; jubarte validate FILE lists them`, and
  `--quiet` drops it.
- Plain `validate` exit code: warnings do not exit 2 (#1150 decides this;
  I recommend exit 0 with warnings, exit 2 for errors).
- Do not "repair" the input's metadata during an edit: that rewrites
  parts the plan did not ask for. `validate --repair` stays the place.

**Alternatives.**

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** `--baseline` diff by identity; warnings once | 0.90 | 0.90 | 0.85 |


- **A cons:** finding identity needs care (counts are not identity: the v2
  plan's "same count hides a new finding" test applies). Dummy:
  `validate out.docx --baseline in.docx` → `new: none; already in the input:
  1 COMMENT_METADATA_MISMATCH (not yours)`, exit 0.


### 7. Markers stay beside a new insertion (#1156)

**Problem.** An `insert` at a paragraph edge wraps the adjacent
`bookmarkEnd` (and, at the start, body-level markers) inside the new
`w:ins`. Word opens it and keeps the bookmark, but Accept All extends the
bookmark over the new text and `endpoints()` no longer match the source
(#827's contract). The code name in the brief is wrong (section 0.1); the
shape is real.

**Design.** The rule `insertion_split_boundary` in `src/edit/tracked.rs`
already has: opening markers travel with the words to their right, closing
markers with the words to their left, body-level markers stay at body
level. Apply it to the edge-insert path for `bookmarkStart/End`,
`commentRangeStart/End` and `permStart/End` (`RANGE_ENDS` in
`src/edit/controls.rs`). Choice A of #1156 (source position wins): the
bookmark does not grow over the new text; a `REF` to it still points at the
original words, which is the conservative reading for a cross-reference
target.

**Verification.** Every row of #1156's table in
`tests/edit_bookmark_preservation.rs`; property: the multiset of marker
positions relative to source text is unchanged by any insert.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** markers beside, source position wins | 0.90 | 0.95 | 1.00 |

- **A cons:** after Accept All the bookmark does not cover the added
  sentence; a human typing in Word *would* have extended it if they typed
  inside the range. Dummy: `<w:p>S r E<w:ins>new</w:ins></w:p>`.


### 8. Plan schema errors that teach, and three aliases

**Problem.** `INVALID_PLAN unknown field "comment"` on `delete`/`rewrite`;
`comment` instead of `text` on comment ops; missing `schema_version`;
trailing characters. The error lists no valid fields. `EditPlan` and every
op use `deny_unknown_fields` (`src/edit.rs:64`), which is right; the
message is the defect.

**Design.**

- Every `INVALID_PLAN` names the op index, the kind, the offending field,
  the valid fields for that kind, and the nearest valid field by edit
  distance: `INVALID_PLAN (op-4, reply_comment): unknown field "comment";
  did you mean "text"? fields: comment_id, text`.
- Missing `schema_version`: `INVALID_PLAN: schema_version is required
  (1 for this build)`; do not default it, a plan without a version is not
  portable.
- Trailing characters: `INVALID_PLAN: the plan must be exactly one JSON
  object; characters follow it at line 6 column 58`.
- Aliases, accepted with a `note:` line in the report, never silently:
  1. `comment` on `delete`: a real feature, a comment anchored to the
     deleted text (Word allows comments on deleted text). Same on
     `rewrite` (anchored to the paragraph) and `strike`/`restore`.
  2. `comment` → `text` on `comment` and `reply_comment`: unambiguous,
     the op has no other string field.
  3. `anchor` → `find` on `replace`, `delete`, `comment`: the CLI word.
- Stay errors: `text` on `replace` (replacement or anchor?), `position`
  together with `after`, a numeric `paragraph`, an unknown `kind` (list the
  kinds), `ids` as a string.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** strict + teaching errors + the three aliases with notes | 0.90 | 0.85 | 0.95 |


- **A cons:** three documented spellings for one thing. Dummy:
  `{"kind":"delete","paragraph":"p5","find":"x","comment":"why"}` → applies,
  comment anchored to the struck "x".


### 9. Summary-first reading

**Problem.** Agents page `read` five to ten times on turn 2–4 documents
because every pending revision is inlined; median first edit at 21–25
minutes. `read --changed --by` exists but still prints whole paragraphs
with marks.

**Design.** Three read-only views, all deterministic, all `--json`:

- `changes --summary`: counts by author × kind, then by section (nearest
  heading above, with its paragraph range): `§8 Limitation of Liability
  (p150-p168): LargeCo 12 ins, 4 del, 3 comments; ids r401-r428`.
- `read --outline`: headings with paragraph ranges and per-section counts
  of pending changes and open threads, nothing else. Two screens for a
  1,000-revision contract.
- `changes --brief [--by A] [--section "8"]`: one line per logical change
  (change 11's grouping): `r401+402 ins p152 §8.1 "and each of their
  respective parents…" (62 chars)`.
- `read -p p150-p168 --track-changes accepted` already gives the compact
  "their draft" text; the skill should say so.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** summary, outline, brief | 0.85 | 0.90 | 0.90 |


- **A cons:** section detection needs heading styles or outline levels;
  documents without them fall back to "body". Dummy above.


### 10. Report what a settle removes; allow replies in tracked-deleted text (#1159)

**Problem.** Accepting r13 (a deletion holding only c4's reference mark)
deletes c4 silently; its reply c5 survives with a dangling parent. A reply
to a comment whose paragraph the plan deletes is refused as
`OVERLAPPING_EDITS` (`src/edit.rs:4050-4056`), the wrong code.

**Design.**

- Every settle path reports each removed thread: `removed c4 (Trainer 1
  - AgentCo) and its reply c5: their anchor was text the acceptance
  removed`; JSON `removed_comments: [{"id":"c4","replies":["c5"]}]`. Replies
  are removed with the parent (Word does this, #1159), so
  `commentsExtended` stays consistent.
- `changes` lists `r13 deletion (holds comment c4's mark)` and the view
  shows the mark inside the deletion.
- `UNKNOWN_COMMENT` after a settle in the same plan says: `rejecting r1
  removes c0 and 2 replies; to keep the thread, strike r1 instead, or reply
  in a plan before this one`.
- Reply to a comment in a paragraph the plan deletes: allowed when the
  deletion is tracked (the reference stays in deleted text and Word shows
  the thread); refused with `COMMENT_ANCHOR_DELETED` only in editing mode.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** report, remove replies with the parent, allow tracked case | 0.90 | 0.90 | 0.95 |


### 11. One notion of a change

**Problem.** `read` shows `r7+8+9+10`; `changes` lists four; rejecting r7
alone leaves "main in effect…" pending. `-p r87+r88` is refused with a
message that does not show `r87+88`.

**Design.** `changes` groups adjacent elements of one author, date and
kind into one logical change with the composite id, listing members in
`--json`; `-p r87+r88` is accepted as a spelling of `r87+88`; the refusal
shows the composite form it saw in the file. After change 1 splits an
insertion, the new pieces join the same group.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** group in `changes`, accept both spellings | 0.85 | 0.90 | 0.95 |


- **A cons:** a group spanning a paragraph mark must still be one id.
  Dummy: `changes` → `r7+8+9+10 insertion p23 by Trainer 1 … (4 elements)`.

### 12. `replace` keeps the formatting it keeps; lenient anchors keep the document's characters

**Problem.** A long `find` with one word changed re-emits the kept
characters in the first run's formatting with a `w:rPrChange`; five
defined terms lost bold in four trials. A straight quote in `--anchor`
matches a curly one and the straight quote in `--content` then replaces it
(p6: seven changes for a one-word edit).

**Design.** The word-level diff inside `replace`/`rewrite` runs on the
normalised text, but emission takes equal segments from the **source
runs** (split at the diff boundaries, original `rPr`, original characters),
and only inserted segments are new runs, formatted like the character
before them. `whole: true` keeps today's behaviour for callers that want
the span re-typed. `anchor_read_as` already records the normalisation;
the replacement is mapped through the same table so a character that
matched leniently is never part of the diff.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** emit equal segments from source runs; map `content` through the match table | 0.90 | 0.95 | 0.90 |
|
- **A cons:** the diff boundary inside a run splits it; more runs. Dummy:
  `--anchor '"Affiliates" means any' --content '"Affiliates" means each'` →
  one `{~~any~>each~~}`, bold untouched, curly quotes untouched.

### 13. Insert into an empty paragraph

`check_insert_position` refuses when the projection is empty
(`src/edit.rs:3971`). Word types into an empty paragraph by creating a run
with the mark's run properties. Do the same: new `w:r` with a copy of
`w:pPr/w:rPr` minus revision marks, wrapped in our `w:ins`. Small. Property:
the paragraph's `pPr` is byte-identical before and after.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** run from the mark's rPr | 0.90 | 0.95 | 1.00 |


### 14. Stable comment ids; no editing another author's comment

**Problem.** A bare `accept`/`reject` renumbers comment ids because
`revision_processor.rs:3493-3544` renumbers annotation ids when no revision
is left; a targeted settle does not. `edit -p c106 --content …` rewrites a
counterparty comment under their name with no warning.

**Design.** Stop renumbering on settle (ids are identifiers; renumber only
in `validate --repair` for duplicates). `edit_comment` on a comment whose
`w:author` differs from the plan's author is refused with
`COMMENT_NOT_YOURS: c106 is by "Trainer 1 - LargeCo"; reply instead
(reply_comment), or pass "rewrite_others_comments": true to change their
words under their name`. The flag exists because the brief allows it when
the user explicitly asks; it is never the default. Word 365's modern
comments only let you edit your own; classic Word let you edit any
(experiment E5 confirms the current build).

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** stable ids; refuse with explicit override | 0.90 | 0.95 | 1.00 |


### 15. `insert_paragraph` with `style` copies nothing else; structural ops stay structural

**Problem.** `style: "Normal"` after a numbered heading still inherits
`numPr` and bold (`src/edit/structural.rs:423-459` copies the anchor's
pPr), so the new paragraph becomes "4." and every later article renumbers.
On the comparer path (no `keep`), `delete_paragraph` and `insert_paragraph`
are diffed character by character across the paragraph boundary, so a
leading “ hops between paragraphs; Accept All and Reject All are right but
a partial settle is not.

**Design.** When `style` is given: `pPr = {pStyle}` only, runs take the
style's formatting; `like` copies from `like`; neither copies the anchor
(today). Structural ops (`insert_paragraph`, `delete_paragraph`,
`merge`) are emitted by the direct emitter in every mode; only
`rewrite`/`replace` text diffs go through the comparer. The emitter exists
(`src/edit/tracked.rs`), so this is routing, not a new engine.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** style means style only; structural ops emitted directly | 0.85 | 0.90 | 0.95 |
| B keep the comparer, pin paragraph boundaries as hard anchors | 0.55 | 0.65 | 0.80 |

### 16. Numbering side effects

**Problem.** An edit adds `w:num` entries with duplicate
`w16cid:durableId` (three delivered documents); after an accept-all on
s2-t3-g06a a single `replace` re-pointed 17 untouched list paragraphs from
`numId` 13 to 25, each as a tracked formatting change by our author, plus
eight duplicate durableIds.

**Design.** A cloned `w:num` gets a fresh durableId (random 32-bit, unique
in the part) or none. An untouched paragraph never changes `numId`: find
the path that re-points them (the comparer's numbering reconciliation on
the accept-all base, `B11` in the repro) and make it leave paragraphs the
plan did not name. Experiment E6 settles what Word does with duplicate
durableIds (I expect it to open and regenerate them, but a duplicate may
make Word treat two lists as one).

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** fresh durableId; untouched paragraphs untouched | 0.85 | 0.95 | 0.95 |

### 17. View attribution

Never pair a `w:del` and a `w:ins` by different authors into a
substitution; always print ` on rN` for a change nested in another. Both
are in `src/markdown/from_docx/critic.rs`; `fix/nested-on-1111` is already
touching the nesting. Small; fixtures from s2-t2-g03c p47 and s1-t3-g03a p8.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** pair same author only; always print nesting | 0.85 | 0.95 | 1.00 |

### 18. Settled-id messages

`PARAGRAPH_RESOLVED: p5 merged into p5` names the source id and the
settled id with the same text. Print both with their view: `p5 (as read)
merges into p4 (after the plan's revisions settle); run read
--track-changes accepted to see settled ids, or give a find`. Better:
resolve whole-paragraph ops by paragraph identity across the settle when
the mapping is one-to-one (B11's `paragraph_ids()` already computes it) and
only refuse the many-to-one case. Document the code in `jubarte help
errors`.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** resolve through the mapping; refuse only ambiguity, with both ids | 0.85 | 0.90 | 0.95 |


### 19. A new paragraph beside a pending-deleted mark

Place the new paragraph where Word's Enter would put it (experiment E3)
and report: `p3's paragraph mark is a pending deletion by Trainer 1
(r88); if it is accepted, p3 and the new paragraph merge; restore r88 to
keep them apart`. Do not restore the mark unasked. Small.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** Word's placement + report | 0.80 | 0.90 | 1.00 |


### 20. Small fixes from the repro

`changes --json` keeps `\t` in revision text (B12). `validate --original`
treats a revision present in the original as not foreign (#1192). Both
small, both with a one-line test.

### 21. Revision-aware compare (three-way)

**Problem.** Two-way `compare` reads the input with the counterparty's
changes accepted, so their 442 revisions flatten, every unstruck change
becomes implicitly accepted, and a comment anchored to their deletion loses
its anchor (c106). The brief asks whether a three-way compare should be the
primary workflow.

**My view.** Not primary. Changes 1 and 2 remove the `UNSUPPORTED_STRUCTURE`
class in place, and in-place editing is what Word does. A three-way
compare is worth building as a *second* workflow, for an agent or a human
who edits plain text (the Markdown round trip) and wants the redline
regenerated, and it should be built as **diff → plan → `keep` emission**,
not as a new comparer mode:

1. `accepted = accept_all(input)`; `diff(accepted, candidate)` with the
   existing word-level comparer gives our edits as spans in accepted-text
   coordinates.
2. Each span becomes a plan op (`replace`/`insert`/`delete`,
   `insert_paragraph`/`delete_paragraph`) addressed by settled paragraph id
   and `find` text.
3. The plan is applied to the **original input** under `keep`, which, after
   change 1, nests our deletions inside their insertions and lays our
   insertions beside their deletions. Their pending changes keep their
   author; nothing is accepted implicitly.
4. Comments in the candidate that are not in the input become `comment`
   ops anchored by text; comments in both are kept by id.

The C4 memo's option C ("full merge semantics in the comparer") is the
expensive road; this path reuses three shipped pieces.

**Hard cases, honestly.** Their pending deletion whose text we retyped
(ambiguous: restore or new insertion; choose `restore` when the text is
identical). Moves (their `moveFrom`/`moveTo` pending while we edit inside:
refuse those spans with the span named). Paragraph splits and merges across
their deleted marks. Table rows they inserted that we removed (row-level
`w:del`). Fields and `pPrChange`. Text that equals their insertion but
sits elsewhere (LCS anchors handle most; a probe set is needed). Comments
anchored entirely in text we deleted (keep them on the deleted text, as
Word does; the c118 loss disappears).

**Verification.** Property on every case folder:
`changes_by(other)(threeway(in, cand)) ≡ changes_by(other)(in)` as XML, and
`accept_all(threeway(in, cand))` text ≡ `cand` text. Word opens each.

| alt | desirability | best practices | handrolling |
|---|---|---|---|
| **A (R)** diff → plan → `keep` emission | 0.70 | 0.85 | 0.85 |


- **A cons:** the synthesized plan can hit the same refusals as a hand
  plan (moves, fields); the output is only as good as the `keep` emitter.
  Dummy: `compare in.docx cand.docx --keep-revisions -o out.docx`.


### 22. Help that fits a clock

`jubarte help plan` prints the plan schema (from `capabilities`) with one
example per kind, under 6 KB. `jubarte help errors` lists every code with
one line and the usual fix. The 63 KB help-all stays as the reference and
the skill stops telling the agent to read it first. Small.

### 23. Skill template (bench side)

Not an engine change, listed because the brief asks. The template in
`~/temp/T/jubarte-syntax/skill-tools/SKILL.template.md` should say: apply
each section's plan as soon as it dry-runs clean (`--skip-failed` when the
clock is short); `strike`/`restore` for the counterparty's changes you
reject, `resolve_revisions` only for changes you accept or want gone; one
line on `validate --baseline`; delete "one or two passes" and "no
findings"; state what to do with ten minutes left (apply what you have).

---

## B. The design questions

**B1. Visible reject: default or explicit?** Explicit, as `strike` and
`restore` operations (change 2), with `reject --visible` as a spelling.
`reject` must keep Word's meaning: a lawyer who asks for Reject expects
Word's Reject. What practising lawyers expect by round: in rounds 2 to 4
the convention is "mark up their last draft", which in Word with tracking
on produces exactly the nested shapes, so visible is the natural form for
every change we do not accept; silent settles belong to the clean-up before
signature, or to changes we accept. The engine should make the visible
form the one the hint offers first (change 4) and the skill the one it
teaches, without changing the verb.

**B2. Partial application.** Yes, opt-in (`--skip-failed`, change 3),
with the failed ops written back as a runnable plan and an optional `group`
for ops that must land together. Coherence at the document level is
already guaranteed by `check_conflicts`; coherence at the legal level is the
agent's and the report names what did not land. Yes, print the apply
command after a clean dry-run, and print the new sha256 after every write.
Do not make apply the default of `--dry-run`.

**B3. Three-way compare.** Buildable as diff → plan → `keep` emission
(change 21); the hard cases are moves, deleted paragraph marks, rows, and
identical text that is both their insertion and our retyping. Worth it as
a second workflow after changes 1 and 2, not instead of them: fixing in-place
editing is a medium change that makes the file identical to a Word session;
the three-way is a large change whose output is bounded by the same emitter.

**B4. `validate --baseline`** yes (change 6), by finding identity, not
count. The bookmark fix (change 7) is independent of it and should land
first; note that the code name in the brief is not produced by 0.12.0
(section 0.1).

**B5. Summary-first `read`.** `changes --summary` (author × kind × section
with id ranges), `read --outline` (headings, ranges, counts), `changes
--brief` (one line per logical change), and point the skill at
`read -p RANGE --track-changes accepted` for the compact text (change 9).
Grouping by section needs heading styles; fall back to "body" without them.

**B6. Aliases.** Accept with a note: `comment` on `delete`, `rewrite`,
`strike`, `restore` (a real feature, not an alias); `comment` → `text` on
`comment`/`reply_comment`; `anchor` → `find`. Keep as errors, with the valid
fields and the nearest one named: `text` on `replace`, `position` with
`after`, numeric `paragraph`, unknown `kind`, missing `schema_version`.

---

## C. What not to do

- **Do not fake a tracked change to satisfy a validity gate.** A round
  whose answer is "accept all" has no change by us. Fix the gate.
- **Do not make `reject` visible by default**, and do not add a mode that
  strikes everything the counterparty did. A lawyer decides per change.
- **Do not auto-settle the containing revision** to make an edit fit
  (change 1, alternative C): it re-attributes their words to the base.
- **Do not re-anchor another author's comment** to text they did not
  comment on (change 10, alternative C), and do not edit their comment
  text without an explicit flag.
- **Do not suppress validator codes globally** because the inputs carry
  them; diff against a baseline instead.
- **Do not add a `--force` that overrides `STALE_SOURCE`** or conflicts.
- **Do not auto-apply after a dry-run**, and do not restore a deleted
  paragraph mark unasked.
- **Do not let a lenient parser drop unknown plan fields**; a typo in
  `replacement` would become a silent no-op.
- **Do not tune the word-level diff to the judge's annotated view.** The
  shapes to match are Word's; the fixtures are Word's files.
- **Do not generate a comment for every change** to "explain" it; the
  agent asked for a comment where it wanted one.

---

## D. Word experiments that settle the open shapes

Run each through `word_pdf.py` normal, `--accept-all` and `--reject-all`,
on Word 16, and keep the files as fixtures beside `Step_2`.

- **E1.** Author A inserts a paragraph (mark included); author B deletes the
  whole paragraph with tracking on. Expected: `w:rPr` of the mark holds both
  `w:ins` (A) and `w:del` (B).
- **E2.** A moves a sentence; B deletes two words inside the `moveTo` with
  tracking on. Expected: a nested `w:del` inside `w:moveTo`.
- **E3.** A deletes p3's paragraph mark (p3 and p4 pending merge); B puts
  the cursor at the end of p3 and presses Enter, then types. Where does the
  new mark go, and what does Accept All read?
- **E4.** A deletes a paragraph mark; B retypes the paragraph break. Shape
  of the restored mark.
- **E5.** In the current Word build, can B edit A's comment text, and what
  does the XML carry afterwards (`w:author`, `w:date`)?
- **E6.** Two `w:num` entries with the same `w16cid:durableId`: does Word
  open, repair, or merge the lists?
- **E7.** Word's Compare of `Step_1` against `Step_2`, both carrying
  pending changes: whether Compare keeps A's history (the C4 memo's
  question), as a reference for change 21.

---

## E. Order and stacking

PR over PR, each stacked on the previous one, none merged until you say
so:

1. `fix/edit-inside-ins-1136` (change 1) with #1108's hint (change 4).
2. `strike`/`restore` (change 2).
3. `--skip-failed`, `next:` line, `sha256:` after write (change 3).
4. Option conflicts and JSON errors (change 5); `--baseline` and
   warnings once (change 6); markers beside insertions (change 7).
5. Schema errors and aliases (change 8); settled-id messages (change 18).
6. Comment loss reporting, reply in deleted text, empty paragraph, stable
   comment ids, comment author guard (changes 10, 13, 14).
7. One notion of a change (change 11); view attribution (change 17).
8. Formatting-preserving replace and lenient anchors (change 12).
9. Structural ops and `style` (change 15); numbering (change 16); the
   pending-mark placement (change 19); small fixes (change 20).
10. Summary reads and help (changes 9, 22); skill template (change 23).
11. Three-way compare (change 21), after E7.

Each PR: a Ring-1 or edit test with a red probe first, the fixture from the
case folder it cites, `cargo clippy --all-targets -- -D warnings`, and a
Word open of every new shape before the PR is marked ready.
