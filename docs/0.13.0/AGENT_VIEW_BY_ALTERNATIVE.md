# The agent view under each alternative

Companion to `PROPOSALS_PLAN.md`. For every item there and every alternative
(A, B, C, D as numbered there), this shows what the agent sees. "Agent view"
means the output of `jubarte read`. Where an alternative changes no line of
the view, that is said, and the command output the agent sees instead is
shown (stderr, the report, a listing).

**Dialect.** The examples use the settled `re` dialect (#1135, migration B):
`{++x++}{r1 @A}`, a nested change `{--x--}{r4 @B re r3 @A}`, a paragraph
mark `¶{r0 @A}`, comments `{==x==}{>>c5 @A: text<<}`, replies
`{>>c6 @B re c5: text<<}`. On today's `main` the same lines print with
`{>>r1 @A<<}` and ` on ` instead of ` re `; nothing else differs.

**Handles.** `@TL` is "Trainer 1 - LargeCo" (the counterparty), `@RC` is
"Reviewing Counsel (AgentCo)" (us). Handles are illustrative; jubarte derives
them from the names. Revision ids above r900 are the ones our edits create.

## The running document (turn 2, before any edit)

```
---
source: contract.docx
sha256: 3f9c0b2e…9e1
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: on                  # w:trackRevisions set; edits are tracked
revisions: 442                     # 486 marks shown as 442 changes: 301 insertions, 128 deletions, 9 substitutions, 4 paragraph marks
comments: 62 threads open, 3 resolved
authors:
  document_owner: Trainer 1 - LargeCo   # dc:creator
  TL: Trainer 1 - LargeCo          # 442 revisions, 2026-10-02T09:00:00Z
body: p0-p310, 6 tables, 41 pages  # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, justify  # default; unannotated paragraphs use it
  "#": Heading1, Calibri bold 14pt, before 12pt, keep-next
---

<!-- p23 -->
This Agreement commences on the Effective Date {++and shall remain in effect until terminated++}{r7+8+9+10 @TL} in accordance with Section 12.

<!-- p24 -->
The {==Term==}{>>c4 @TL: Term is not defined in this agreement.<<}{>>c5 @RC re c4: Agreed; we will define it in 1.1.<<} begins on the Effective Date.

<!-- p155 empty -->

<!-- p158 -->
AgentCo may {--use, store, copy, analyze, monitor, create Derivative Works from--}{r12 @TL} access the Customer Data solely to provide the Services.

<!-- p160 -->
Neither party shall be liable to the other party {++and each of their respective parents, subsidiaries and affiliates++}{r366 @TL} for any indirect damages.

<!-- p162 -->
Either party may terminate for convenience on {++ninety (90) days'++}{r242 @RC} notice.
```

(p162 holds our own insertion from a write earlier in the session. In p24,
comment c4's reference mark sits inside LargeCo's deletion r13, which holds
nothing else; today the view prints that deletion empty, see item 10.)

---

## 1. Edits inside another author's insertion

Command: `jubarte edit contract.docx -p p160 --anchor "parents, subsidiaries and affiliates" --content "subsidiaries" -a "Reviewing Counsel (AgentCo)" -o contract.docx --force`

**A (R), Word's nested shape.** The view after the edit:

```
<!-- p160 -->
Neither party shall be liable to the other party {++and each of their respective ++}{r366 @TL}{~~parents, subsidiaries and affiliates~>subsidiaries~~}{r901+902 @RC re r366 @TL}{++ ++}{r903 @TL} for any indirect damages.
```

- our deletion is nested in their insertion (`re r366 @TL`); our new word
  is a sibling insertion; the remainder of their insertion carries a new
  id, r903, with their name and date. `jubarte changes` lists r366 and r903
  as one logical change once item 11 lands: `r366+903 insertion … @TL`.
- the header moves to `revisions: 444 … by TL, RC`, `authors:` gains
  `RC: Reviewing Counsel (AgentCo) # 2 revisions`.

The own-insertion case, `edit -p p162 --anchor "ninety (90)" --content "sixty (60)"`:

```
<!-- p162 -->
Either party may terminate for convenience on {++sixty (60) days'++}{r242 @RC} notice.
```

In place, same id, no nested mark: the view reads as if we had typed it the
first time, which is what Word records.

**B, refuse with a teaching hint.** The view is unchanged (p160 as in the
running document). stderr:

```
error: plan refused: UNSUPPORTED_STRUCTURE (op-1): the text is inside r366, text "Trainer 1 - LargeCo" inserted.
  strike it visibly:    {"kind":"strike","revision":"r366"}
  accept it, then edit: "resolve_revisions":{"accept":{"ids":["r366"]}}   (their words become plain text)
  or comment on it:     {"kind":"comment","paragraph":"p160","find":"parents","text":"…"}
```

**C, auto-settle the containing revision.** The view loses their
attribution:

```
<!-- p160 -->
Neither party shall be liable to the other party and each of their respective {~~parents, subsidiaries and affiliates~>subsidiaries~~}{r901+902 @RC} for any indirect damages.
```

`revisions: 441`, and `TL` drops to 441 in `authors:`. Nothing in the view
says that "and each of their respective" was LargeCo's proposal. That is
why C is rejected.

---

## 2. `strike` and `restore`

**A (R), operations.** Plan:

```json
{"kind": "strike",  "revision": "r366", "comment": "Parents and affiliates are not parties to this Agreement."}
{"kind": "restore", "revision": "r12",  "comment": "This carve-out was agreed in round 1."}
```

View:

```
<!-- p158 -->
AgentCo may {--use, store, copy, analyze, monitor, create Derivative Works from--}{r12 @TL}{++use, store, copy, analyze, monitor, create Derivative Works from++}{r904 @RC}{>>c120 @RC: This carve-out was agreed in round 1.<<} access the Customer Data solely to provide the Services.

<!-- p160 -->
Neither party shall be liable to the other party {--and each of their respective parents, subsidiaries and affiliates--}{r905 @RC re r366 @TL}{>>c121 @RC: Parents and affiliates are not parties to this Agreement.<<} for any indirect damages.
```

- p160: a deletion that is another change's whole content prints as the
  deletion with `re r366 @TL` (the dialect's "change that was another's
  only content" form). The agent sees whose words it struck.
- p158: their deletion stays, our insertion follows it, different authors
  are never fused into one substitution (item 17).
- `read --track-changes accepted` of the same file:

```
<!-- p158 rev r12 @TL; r904 @RC -->
AgentCo may use, store, copy, analyze, monitor, create Derivative Works from access the Customer Data solely to provide the Services.

<!-- p160 rev r366 @TL; r905 @RC -->
Neither party shall be liable to the other party  for any indirect damages.
```

- `read --track-changes rejected`: p158 reads "…may use, store, copy, …
  access…" once; p160 reads "…to the other party  for any…".

**B, `resolve_revisions.reject` with `visible: true`.** The view is
identical to A's. The difference is in the plan and the report: because
`resolve_revisions` runs before the operations, an operation later in the
same plan still sees the struck text:

```
error: plan refused: UNSUPPORTED_STRUCTURE (op-7): the text is inside r905, text "Reviewing Counsel (AgentCo)" deleted
```

for a `replace` on p160 whose `find` touches "parents". Under A the same
plan is refused by `check_conflicts` as two operations on one range, with
both op ids named.

**C, `existing_revisions: "layer"`.** The struck items print as in A. Every
other reject in the plan also becomes visible, including the ones the agent
wanted silent. A typo fix by LargeCo (r370, "teh" → "the") that the agent
rejected because it was wrong becomes:

```
<!-- p201 -->
Supplier shall deliver {~~teh~>the~~}{r370+371 @TL}{--the--}{r906 @RC re r371 @TL}{++teh++}{r907 @RC} Services…
```

Noise a lawyer would not produce; the agent cannot mix one silent reject
with one strike.

**D, visible is the new meaning of `reject`.** As C, for every plan and
for `jubarte reject --id r366` on the command line. The lawyer who asked
for Word's Reject gets a strikethrough instead; the view never shows a
clean removal again.

---

## 3. Partial application, the apply command, the new digest

The view is unchanged by the mechanism; what changes is what the agent
sees around the write.

**A (R), `--skip-failed`, `P.failed.json`, `next:`, `sha256:`.** A clean
dry-run:

```
$ jubarte edit contract.docx --plan plan.json --dry-run
{"ev":"plan","sha256":"3f9c0b2e…9e1","author":"Reviewing Counsel (AgentCo)","ops":9}
{"ev":"op","id":"op-1","op":"strike","status":"ok","at":"p160"}
…
{"ev":"summary","status":"ok","ops":9,"ok":9}
next: jubarte edit contract.docx --plan plan.json -o contract.docx --force
```

A write with one bad op:

```
$ jubarte edit contract.docx --plan plan.json -o contract.docx --force --skip-failed
{"ev":"op","id":"op-4","op":"replace","status":"skipped","code":"ANCHOR_NOT_FOUND","message":"\"three (3) month\" does not occur in p161; the paragraph reads: \"…three (3) months…\""}
{"ev":"summary","status":"partial","ops":9,"ok":8,"skipped":1,"sha256":"a71d…0c4"}
wrote contract.docx (docx, 412,118 bytes)
sha256: a71d…0c4
skipped 1 of 9 operations; the fixes are in plan.failed.json
```

`plan.failed.json`:

```json
{"schema_version": 1, "source_sha256": "a71d…0c4", "author": "Reviewing Counsel (AgentCo)", "existing_revisions": "keep",
 "operations": [
  {"id": "op-4", "kind": "replace", "paragraph": "p161", "find": "three (3) month", "replacement": "ninety (90) days",
   "why": "ANCHOR_NOT_FOUND: \"three (3) month\" does not occur in p161; the paragraph reads: \"…three (3) months…\""}
 ]}
```

The view after the write holds the eight landed changes; its header's
`sha256:` equals the line printed.

**B, `--batch N`.** stderr after each batch:

```
wrote contract.docx after operations 1-5 (sha256 9b2e…)
error: plan refused: ANCHOR_NOT_FOUND (op-7): …
operations 6-9 not written; fix op-7 and rerun --plan plan.json --from op-6
```

The view holds five changes; the agent must know `--from`.

**C, atomic, skill says "small plans".** Nothing new to see; on the bad op
the agent sees today's `error: plan refused: … nothing was written` and a
view without any of the nine changes.

**D, apply is the default after a clean check.** `jubarte edit … --check`
prints the report and, when clean, immediately `wrote contract.docx`. The
agent that wanted to look first has already written.

---

## 4. Refusal hints

No change to the view. After item 1 the hint fires only for deletions,
moves and fields.

**A (R), one message, three routes, the cost of each:**

```
error: plan refused: UNSUPPORTED_STRUCTURE (op-2): the text is inside r12, text "Trainer 1 - LargeCo" deleted.
  put it back visibly:  {"kind":"restore","revision":"r12"}             (their deletion stays, your insertion shows)
  let it go:            "resolve_revisions":{"accept":{"ids":["r12"]}}  (their words leave the document for good)
  put it back silently: "resolve_revisions":{"reject":{"ids":["r12"]}}  (their deletion disappears, no trace)
  or comment on it:     {"kind":"comment","paragraph":"p158","find":"access the Customer Data","text":"…"}
```

**B, short hint plus a `read --for-edit` continuation:**

```
error: plan refused: UNSUPPORTED_STRUCTURE (op-2): the text is inside r12 (a deletion by "Trainer 1 - LargeCo")
  decide it first: jubarte read contract.docx -p p158 --for-edit
```

and the continuation prints the decisions:

```
<!-- p158 decisions: r12 @TL deletion "use, store, copy, …" → restore | accept | reject | comment -->
```

One more call under the clock.

---

## 5. Option conflicts and JSON-shaped usage errors

No change to the tracked view. What the agent sees:

**A (R):**

```
$ jubarte read contract.docx --no-page-markers --json | jq -r .source_sha256
3f9c0b2e…9e1

$ jubarte read contract.docx --track-changes accepted --json | jq '.paragraphs[160].text, .summary.track_changes'
"Neither party shall be liable to the other party and each of their respective parents, subsidiaries and affiliates for any indirect damages."
"accepted"

$ jubarte read contract.docx --tables -p t0
table 0: 4x3 header_rows=1 widths=2.1in,2.4in,2.0in
p40=Service	p41=Uptime Commitment	p42=Credit
…

$ jubarte read contract.docx --json --head 3 --tail 3 2>&1 | jq .
{"error": {"code": "USAGE", "message": "--head cannot be used with --tail", "usage": "jubarte read [OPTIONS] <FILE>"}}
```

**B, keep the refusals, teach better:**

```
$ jubarte read contract.docx --no-page-markers --json
error: --no-page-markers shapes the view; a .json snapshot has none. Drop it:
  jubarte read contract.docx --json
```

Still a non-JSON line on stderr; with `2>&1 | jq` the agent sees
`jq: parse error`.

---

## 6. `validate --baseline`, warnings once

No change to the tracked view's body.

**A (R):**

```
$ jubarte validate contract.docx --baseline task_input.docx
new: none
already in the input: 1 (not yours)
  COMMENT_METADATA_MISMATCH  word/commentsExtensible.xml  c17 has no durableId
$ echo $?
0

$ jubarte read contract.docx -p p160 >/tmp/v.md
input has 1 validity warning; jubarte validate contract.docx lists it
```

(the second line is stderr, once per call, dropped by `--quiet`). After an
edit that does introduce a defect:

```
$ jubarte validate contract.docx --baseline task_input.docx
new: 1
  COMMENT_PARENT_CYCLE  word/commentsExtended.xml  c122's paraIdParent names c122
already in the input: 1 (not yours)
$ echo $?
2
```

**B, suppress noisy codes globally.** `validate` prints `no findings` for
both files above, including the one with the new cycle.

**C, edit fills missing durableIds.** `validate` prints `no findings` on
the output because the edit rewrote `commentsExtensible.xml`; `jubarte
debug task_input.docx contract.docx` shows a part changed that the plan did
not name.

---

## 7. Markers beside a new insertion

Command: `edit -p p33 --anchor "" --content " AI Outputs are drafts for human review." --position end` (plan `insert`, `position: end`).

The view hides `_Hlk` bookmarks, so **A and B print the same line**:

```
<!-- p33 -->
Supplier will provide the Outputs described in the Order Form.{++ AI Outputs are drafts for human review.++}{r908 @RC}
```

The difference is only in the file and in the settled views. For a `_Ref`
bookmark that a cross-reference targets, `convert -t md` (plain Markdown)
shows where the anchor sits:

- **A (R), source position wins:** `<a id="_Ref12"></a>Supplier will provide … Order Form.{++ AI Outputs …++}` and after Accept All the `REF` still reads "Supplier will provide … Order Form."
- **B, range grows:** the same tracked view; after Accept All the `REF`
  reads "… Order Form. AI Outputs are drafts for human review."
- **C, refuse:** the view is unchanged and stderr says
  `CANDIDATE_INVALID (op-1): the insertion would move bookmark _Hlk8812's end`.

---

## 8. Plan schema errors and aliases

Plan line: `{"kind": "delete", "paragraph": "p158", "find": "solely to provide the Services", "comment": "Too narrow for the support scope in Exhibit B."}`

**A (R), accepted as a feature, with a note.** stderr: `note: op-3: comment
on delete is anchored to the deleted text`. View:

```
<!-- p158 -->
AgentCo may {--use, store, copy, analyze, monitor, create Derivative Works from--}{r12 @TL} access the Customer Data {--solely to provide the Services--}{r909 @RC}{>>c123 @RC: Too narrow for the support scope in Exhibit B.<<}.
```

And for `{"kind": "reply_comment", "comment_id": "c4", "comment": "We will define Term in 1.1."}`:
`note: op-5: read "comment" as "text"`; view in p24 gains
`{>>c124 @RC re c4: We will define Term in 1.1.<<}`.

**B, strict, teaching error.** The view is unchanged; stderr:

```
error: INVALID_PLAN (op-3, delete): unknown field "comment"; fields: paragraph, find, occurrence
error: INVALID_PLAN (op-5, reply_comment): unknown field "comment"; did you mean "text"? fields: comment_id, text
```

**C, lenient parser.** No error, no note. View:

```
<!-- p158 -->
AgentCo may {--…--}{r12 @TL} access the Customer Data {--solely to provide the Services--}{r909 @RC}.
```

The comment the agent wrote is gone and nothing said so.

---

## 9. Summary-first reading

**A (R).** Three new outputs, nothing changes in the tracked view.

```
$ jubarte changes contract.docx --summary
by author
  TL  Trainer 1 - LargeCo   301 ins  128 del  9 subst  4 marks   62 comments
by section
  §1 Definitions (p5-p48)                 TL 41 ins 12 del 2 subst   r1-r61      comments c1-c9
  §8 Limitation of Liability (p150-p168)  TL 12 ins  4 del 1 subst   r401-r428   comments c40-c43
  §9 Data (p169-p190)                     TL 22 ins 19 del           r429-r470   comments c44-c51
  Exhibit C (p260-p310)                   TL 30 ins  6 del           r500-r536   comments c58-c62
442 change(s) in 23 sections

$ jubarte read contract.docx --outline
<!-- p0 # -->   MASTER SERVICES AGREEMENT
<!-- p5 # -->   1. DEFINITIONS                      p5-p48    55 changes  9 threads
<!-- p49 # -->  2. SERVICES                         p49-p80   31 changes  4 threads
…
<!-- p150 # --> 8. LIMITATION OF LIABILITY          p150-p168 17 changes  4 threads
…

$ jubarte changes contract.docx --brief --section 8
r401+402  ins   p152  §8.1  "and each of their respective parents, subsidiaries and affiliates" (62)
r403      del   p152  §8.1  "direct"
r404+405  subst p153  §8.1  "the fees paid" → "the fees paid or payable in the twelve (12) months"
…
17 change(s)
```

Then `read -p p150-p168` for the section itself, as today.

**B, `read --changed --marks-only`.** The tracked view with everything
between marks dropped:

```
<!-- p152 -->
… {++and each of their respective parents, subsidiaries and affiliates++}{r401+402 @TL} … {--direct--}{r403 @TL} …
```

Shorter, but the sentence the mark sits in is gone, so the agent cannot
judge it without another read.

**C, v2 `read --for-edit --find`.**

```
$ jubarte read contract.docx --find "liable" --context 1 --for-edit --json
{"cursor":"c9f1…","matches":[{"id":"p160","text":"Neither party shall be liable to the other party and each of their respective parents, subsidiaries and affiliates for any indirect damages.","before_sha256":"…","revisions":[{"short_id":"r366","long_id":"document.w:ins.w:id.366","author":"Trainer 1 - LargeCo","kind":"insertion"}]}]}
```

Working text (their changes accepted) with the revision inventory beside
it; a cursor to continue. Precise, but a larger contract and JSON only.

---

## 10. What a settle removes; replies in deleted text

**A (R).** The view shows the reference mark inside the deletion so the
agent can see that c4 rides on r13:

```
<!-- p24 -->
The {==Term==}{--{>>c4 @TL: Term is not defined in this agreement.<<}--}{r13 @TL}{>>c5 @RC re c4: Agreed; we will define it in 1.1.<<} begins on the Effective Date.
```

`jubarte changes` lists `r13	deletion	run	Trainer 1 - LargeCo	p24	""	holds comment c4`.
Accepting r13:

```
$ jubarte edit contract.docx -a "Reviewing Counsel (AgentCo)" -p r13 --accept -o out.docx
accepted r13 in p24: "{----}" > ""
removed c4 (Trainer 1 - LargeCo) and its reply c5: their anchor was the mark the acceptance removed
```

and the view of `out.docx` has p24 with no comment at all, consistent with
`comments.xml`, `commentsIds.xml` and `commentsExtended.xml`. A plan that
rejects r1 and replies to the comment anchored in it, in one plan:

```
error: plan refused: UNKNOWN_COMMENT (op-3): rejecting r1 removes c0 and 2 replies (their anchor is r1's text); to keep the thread, strike r1 instead ({"kind":"strike","revision":"r1"}), or reply in a plan before this one
```

A reply to a comment in a paragraph the same plan tracked-deletes is
allowed; the view:

```
<!-- p211 break-del r910 @RC -->
{--{==Customer grants AgentCo the right to identify Customer as a customer of the Service==}{>>c33 @TL: Standard in our forms.<<}{>>c125 @RC re c33: Not for a regulated entity; struck.<<} and to use Customer's logo for that purpose.--}{r910 @RC}
```

**B, refuse the settle unless `--allow-comment-loss`.** The view is
unchanged; stderr: `error: accepting r13 removes c4 and its reply c5; pass
--allow-comment-loss to proceed`. One more refusal; Word itself does not ask.

**C, re-anchor the comment.** View of `out.docx`:

```
<!-- p24 -->
The {==Term==}{>>c4 @TL: Term is not defined in this agreement.<<}{>>c5 @RC re c4: Agreed…<<} begins on the Effective Date.
```

LargeCo's comment now sits on "Term" as if they had placed it there. They
did not; their reference mark was in text they themselves deleted.

---

## 11. One notion of a change

**A (R).** The tracked view is unchanged (`{++and shall remain in effect
until terminated++}{r7+8+9+10 @TL}`); `changes` agrees with it:

```
$ jubarte changes contract.docx
r7+8+9+10	insertion	run	Trainer 1 - LargeCo	p23	"and shall remain in effect until terminated"	4 elements
…
$ jubarte edit contract.docx -p r7+r8+r9+r10 --reject -a "Reviewing Counsel (AgentCo)" -o out.docx
rejected r7+8+9+10 in p23: "{++and shall remain in effect until terminated++}" > ""
```

(`-p r7+r8+r9+r10` is read as `r7+8+9+10`; the receipt prints the
canonical form.) After item 1 splits an insertion, the remainder joins the
group: `r366+903 insertion … 2 elements`.

**B, make `read` list the members instead.** The view fragments:

```
<!-- p23 -->
This Agreement commences on the Effective Date {++and shall re++}{r7 @TL}{++main in effect++}{r8 @TL}{++ until++}{r9 @TL}{++ terminated++}{r10 @TL} in accordance with Section 12.
```

A mid-word split shown as four changes is worse for both readers.

---

## 12. `replace` keeps formatting; lenient anchors keep the document's characters

Command: `edit -p p6 --anchor '"Affiliates" means any' --content '"Affiliates" means each'` (straight quotes typed; the document has curly quotes and a bold defined term).

**A (R).**

```
<!-- p6 -->
“**Affiliates**” means {~~any~>each~~}{r911+912 @RC} person that controls, is controlled by…
```

One substitution. The quotes and the bold are the document's. The report
says `anchor_read_as: "“Affiliates” means any"`.

**Today, and B (refuse lenient matches unless `--exact` is off):**

```
<!-- p6 fmt r913 @RC -->
{~~“Affiliates” means any~>"Affiliates" means each~~}{r911+912 @RC} person that controls…
```

Seven changes in the file: the whole span re-typed, both quotes
straightened, and `fmt r913 @RC` on the id line is the un-bolding of
"Affiliates" as a tracked formatting change. B only differs in that the
command is refused first (`ANCHOR_NOT_FOUND: the anchor matches only with
curly quotes read as straight; retype it or pass --lenient`).

**C, normalise the document to the agent's characters.** The whole
paragraph churns:

```
<!-- p6 -->
{~~“~>"~~}{r914+915 @RC}**Affiliates**{~~”~>"~~}{r916+917 @RC} means {~~any~>each~~}{r911+912 @RC} person that controls…
```

---

## 13. Insert into an empty paragraph

Command: `edit -p p155 --content "Reserved." --position end`

**A (R):**

```
<!-- p155 -->
{++Reserved.++}{r918 @RC}
```

The id line loses `empty`; the paragraph keeps its style and alignment.

**B, keep the refusal:** `<!-- p155 empty -->` unchanged; stderr
`UNSUPPORTED_STRUCTURE: cannot insert text into an empty paragraph; use
{"kind":"insert_paragraph","paragraph":"p154","position":"after","runs":[{"text":"Reserved."}]} and delete_paragraph p155`.
Two operations for one typed word.

---

## 14. Stable comment ids; no editing another author's comment

**A (R).** After `jubarte reject contract.docx -o out.docx` (reject all),
the view of `out.docx` keeps every comment id:

```
<!-- p24 -->
The {==Term==}{>>c4 @TL: Term is not defined in this agreement.<<}{>>c5 @RC re c4: Agreed; we will define it in 1.1.<<} begins on the Effective Date.
```

and `jubarte comments out.docx` lists `c4 … c5 reply to c4` as before, so a
`-p c5 --reply` noted earlier still lands in the right thread. An attempt
to rewrite their comment:

```
$ jubarte edit contract.docx -a "Reviewing Counsel (AgentCo)" -p c106 --content "We will share PHI with AgentCo."
error: plan refused: COMMENT_NOT_YOURS (op-1): c106 is by "Trainer 1 - LargeCo"; reply instead ({"kind":"reply_comment","comment_id":"c106","text":"…"}), or pass "rewrite_others_comments": true to change their words under their name
```

View unchanged: `{>>c106 @TL: We do not intend to share any PHI with AgentCo.<<}`.

**Today, and B (allow with a new date):**

```
<!-- p190 -->
{==PHI==}{>>c106 @TL: We will share PHI with AgentCo.<<}
```

LargeCo's name on our sentence. B adds only `{>>c106 @TL 2026-10-18T10:00:00Z: …<<}` under `--dates`, which no reader would notice.

And after today's bare reject, the renumbering the agent does not see
coming: `comments` lists `c6 … c7 reply to c6` where `c7` and `c9` were,
so `-p c9 --reply` replies into a different thread.

---

## 15. `insert_paragraph` with `style`; structural ops stay structural

Plan: `{"kind":"insert_paragraph","paragraph":"p25","position":"after","style":"Normal","runs":[{"text":"Unnumbered para after heading."}]}` after heading "3. SERVICES AND SUPPORT".

**A (R):**

```
<!-- p25 # num "3." -->
SERVICES AND SUPPORT

<!-- p26 break-ins r919 @RC -->
{++Unnumbered para after heading.++}{r919 @RC}¶{r919 @RC}

<!-- p27 # num "4." -->
FEES
```

**Today, and B (comparer with pinned boundaries, which fixes the quote
case but not this one):**

```
<!-- p26 # num "4." break-ins r919 @RC -->
{++**Unnumbered para after heading.**++}{r919 @RC}¶{r919 @RC}

<!-- p27 # num "5." -->
FEES
```

Every later article renumbers; a `REF` to "Section 4" now reads "Section 5".

The structural delete, `delete_paragraph p14` on a clean document:

**A (R):**

```
<!-- p14 break-del r920 @RC -->
{--“Confidential Information” means all non-public information…--}{r920 @RC}¶{r920 @RC}

<!-- p15 -->
“Customer” means the entity identified in the Order Form.
```

**Today (comparer path):**

```
<!-- p14 break-del r920 @RC -->
{--“Confidential Information” means all non-public information…--}{r920 @RC}{++“++}{r921 @RC}

<!-- p15 -->
{--“--}{r922 @RC}Customer” means the entity identified in the Order Form.
```

The opening quote hops from p15 into p14 as two extra changes; a partial
settle leaves `““Customer”`.

---

## 16. Numbering side effects

After `resolve_revisions: accept all` plus one `replace` in p212:

**Today:**

```
<!-- p200 num "1." level 0, fmt r930 @RC -->
Provider shall…
<!-- p201 num "2." level 0, fmt r931 @RC -->
Provider shall…
… (17 such lines)
<!-- p212 num "13." level 0 -->
Provider shall {~~notify~>notify in writing~~}{r940+941 @RC} …
```

Seventeen `fmt` tags by us on paragraphs the plan never named; `jubarte
changes` lists 17 `formatting` changes by RC.

**A (R):**

```
<!-- p200 num "1." level 0 -->
Provider shall…
…
<!-- p212 num "13." level 0 -->
Provider shall {~~notify~>notify in writing~~}{r940+941 @RC} …
```

`changes` lists 2 changes by RC. `validate` has no duplicate `durableId`
to report.

**B, strip `w16cid` from the numbering part.** Same view as A; `jubarte
debug task_input.docx out.docx` shows `word/numbering.xml: 38 durableId
attributes removed`, a change to a part the plan did not name.

---

## 17. View attribution

Two view-only fixes; the file is the same under every alternative.

Wrong pairing, s2-t2-g03c p47 (LargeCo inserted "limited, ", we replaced
"non-exclusive" right after it). **Today:**

```
<!-- p47 -->
AgentCo grants Customer a {~~non-exclusive~>limited, ~~}{r524 @AL}{r141 @TX}{++nonexclusive++}{r524 @AL} license…
```

**A (R):**

```
<!-- p47 -->
AgentCo grants Customer a {++limited, ++}{r141 @TX}{~~non-exclusive~>nonexclusive~~}{r524+525 @AL} license…
```

Lost nesting, s1-t3-g03a p8 (their deletion of "applicable" inside their
own earlier multi-run insertion r13). **Today:**

```
<!-- p8 -->
… the {--applicable--}{r16 @TX2} {++ ++}{r13 @TX} law…
```

**A (R):**

```
<!-- p8 -->
… the {++ ++}{r13 @TX}{--applicable--}{r16 @TX2 re r13 @TX}{++ ++}{r13 @TX} law…
```

The agent can see that "applicable" was itself inserted text, so striking
it is a nested decision, not a change to the base.

---

## 18. Settled-id messages

Plan with `resolve_revisions: {"accept": {"ids": ["r88"]}}` (LargeCo
deleted p3's paragraph mark, so p3 and p4 merge) and `{"kind":"rewrite","paragraph":"p4","text":"…"}`.

**Today:**

```
error: plan refused: PARAGRAPH_RESOLVED (op-2): p4 merged into p4 when the plan's revisions were settled; an operation on a whole paragraph needs its settled id
```

**A (R), resolve through the mapping; refuse only ambiguity.** The plan
applies: the rewrite targets the merged paragraph, and the report says
`{"ev":"op","id":"op-2","op":"rewrite","status":"ok","at":"p3","note":"p4 (as read) is p3 after r88 settles"}`.
View of the output:

```
<!-- p3 -->
The parties agree as follows:{~~The Services are described in Exhibit A.~>The Services and Deliverables are described in Exhibit A and each Order Form.~~}{r942+943 @RC}
```

When two read paragraphs would both map to one settled paragraph the
refusal names both views:

```
error: plan refused: PARAGRAPH_RESOLVED (op-2): p4 (as read) merges into p3 (after the plan's revisions settle) together with p5; name the text with "find", or settle the revisions in a plan of their own and read the document again (jubarte read --track-changes accepted shows the settled ids)
```

**B, fix the message only.** The second message above, every time, even
in the one-to-one case.

---

## 19. A new paragraph beside a pending-deleted mark

Command: `add -p p3 --content "NEW PARAGRAPH AFTER P3."` where LargeCo
deleted p3's mark (r88).

**A (R), Word's placement and a report line** (placement pending
experiment E3; shown as the shape the question observed):

```
<!-- p3 break-del r88 @TL -->
The parties agree as follows:¶{r88 @TL}

<!-- p4 break-ins r944 @RC -->
{++NEW PARAGRAPH AFTER P3.++}{r944 @RC}¶{r944 @RC}

<!-- p5 -->
1. Services.
```

stderr: `note: p3's paragraph mark is a pending deletion by "Trainer 1 -
LargeCo" (r88); if it is accepted, p3 and the new paragraph merge; restore
r88 to keep them apart ({"kind":"restore","revision":"r88"})`.

`read --track-changes accepted` then shows what the note warns of:

```
<!-- p3 rev r88 @TL; r944 @RC -->
The parties agree as follows:NEW PARAGRAPH AFTER P3.
```

**B, restore the mark automatically:**

```
<!-- p3 break-del r88 @TL, break-ins r945 @RC -->
The parties agree as follows:¶{r88 @TL}¶{r945 @RC}

<!-- p4 break-ins r944 @RC -->
{++NEW PARAGRAPH AFTER P3.++}{r944 @RC}¶{r944 @RC}
```

A decision about LargeCo's deletion the agent did not make, written under
our name.

---

## 20. Small fixes

`changes --json` keeps tabs:

```
{"ref":"r62","kind":"insertion","paragraph":"p70","text":"Position: \t\tPosition:"}
```

(today `"Position: Position:"`). And `validate --original task_input.docx
--author "Reviewing Counsel (AgentCo)"` on the untouched turn-2 input
prints `no findings` instead of 518 `FOREIGN_AUTHOR` lines.

---

## 21. Revision-aware compare

Command: `jubarte compare task_input.docx working.docx -a "Reviewing Counsel (AgentCo)" --keep-revisions -o contract.docx` where `working.docx` is the agent's plain-text edit of the accepted view.

**A (R), diff → plan → `keep` emission, and B (full three-way) print the
same view** when they succeed:

```
---
revisions: 451                     # … by TL, RC
authors:
  TL: Trainer 1 - LargeCo          # 442 revisions, 2026-10-02T09:00:00Z
  RC: Reviewing Counsel (AgentCo)  # 9 revisions, 2026-10-18T10:00:00Z
---
<!-- p158 -->
AgentCo may {--use, store, copy, analyze, monitor, create Derivative Works from--}{r12 @TL}{++use, store, copy, analyze, monitor, create Derivative Works from++}{r904 @RC} access the Customer Data…

<!-- p160 -->
Neither party shall be liable to the other party {--and each of their respective parents, subsidiaries and affiliates--}{r905 @RC re r366 @TL} for any indirect damages.
```

Every LargeCo change not touched by the working copy is still pending
under their name. Where A and B differ is the failure mode: A refuses the
spans it cannot replay and names them in the report
(`{"ev":"op","id":"span-14","status":"failed","code":"UNSUPPORTED_STRUCTURE","message":"the text is inside r410, a move by \"Trainer 1 - LargeCo\""}`),
B would have to invent a shape for them inside the comparer.

**C, two-way compare (today):**

```
---
revisions: 61                      # 61 marks shown as 58 changes … by RC
authors:
  RC: Reviewing Counsel (AgentCo)  # 58 revisions, 2026-10-18T10:00:00Z
---
<!-- p158 -->
AgentCo may access the Customer Data…

<!-- p160 -->
Neither party shall be liable to the other party {--and each of their respective parents, subsidiaries and affiliates--}{r905 @RC} for any indirect damages.
```

`TL` is not in `authors:` at all: their 442 changes read as the base. In
p158 the deletion we let stand has simply vanished, and in p160 our strike
of their new words looks like a strike of agreed text.

---

## 22. Help

```
$ jubarte help plan
A plan is one JSON object:
  {"schema_version": 1, "source_sha256": "<sha256: line of read>", "author": "…", "existing_revisions": "keep",
   "resolve_revisions": {"accept": {"ids": ["r12"]}, "reject": {"ids": ["r13"]}},
   "operations": [ … ]}
Operation kinds (fields; * required):
  replace          paragraph*, find*, replacement*, occurrence, format, comment, whole
  insert           paragraph*, text*, one of after | before | position, occurrence, format, comment
  delete           paragraph*, find*, occurrence, comment
  strike           revision*, comment
  restore          revision*, comment
  …
Example, one of each kind: jubarte help plan --examples
```

Under 6 KB; the 63 KB `--help` collection stays as the reference.

---

## 23. Skill template (bench side)

Not a view. The paragraph that replaces "execute in one or two jubarte
passes":

```
Apply each section's plan as soon as it dry-runs clean; do not grow one plan for the whole document.
Use strike / restore for the counterparty's changes you reject; resolve_revisions only for changes you accept or want gone without trace.
Every write prints sha256:; put it in the next plan's source_sha256.
With ten minutes left: jubarte edit … --skip-failed and stop.
jubarte validate FILE --baseline ORIGINAL must print "new: none"; findings already in the input are not yours.
```
