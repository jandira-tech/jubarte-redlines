# Question: what should change in jubarte so it is genuinely useful for redlining contracts?

## Your role and the goal

You are reviewing **jubarte**: a CLI that lets an AI agent read, edit, compare and validate Word (`.docx`) contracts with native tracked changes and comments.

We measured it inside a contract-redlining benchmark. **We do not want changes that raise the benchmark score.** We want jubarte to be the tool a careful lawyer, or an agent working for one, would actually want.

Judge every proposal against two users:

1. **The lawyer who opens the result in Microsoft Word.** The document must open without any warning or repair prompt. Every party's changes must be attributed to the right author. The other side must be able to see what we proposed, what we accepted, and what we rejected.
2. **The agent driving the CLI.** It works under a time budget, often on documents carrying hundreds to over a thousand tracked changes by other authors. It needs operations that succeed on the first try, errors that say exactly how to fix the call, and work that is never lost when time runs out.

If a change would help the benchmark but make the tool worse or less honest for a real user, say so and reject it. If our analysis below is wrong, say so plainly.

## What jubarte is (version 0.12.0)

**Commands:**
- `read`: the agent view, with `<!-- pN -->` paragraph ids, inline CriticMarkup for revisions, and comments with ids
- `changes`, `accept`, `reject`: list and settle tracked changes
- `edit` and `add`: single operations via `-p WHERE --anchor/--content/--delete/...`, or a JSON plan
- `compare` / `redline`, `diff`, `visual-diff`, `comments`, `validate`, `scrub`, `convert`, `append`

**The edit plan** (the main write path):

```json
{
  "schema_version": 1,
  "source_sha256": "<from read's header>",
  "author": "Reviewing Counsel (LargeCo)",
  "existing_revisions": "keep",
  "resolve_revisions": {"accept": {"ids": ["r46+45"]}, "reject": {"ids": ["r50"]}},
  "operations": [
    {"kind": "replace", "paragraph": "p160", "find": "three (3) months", "replacement": "ninety (90) days", "comment": "Standard notice period."},
    {"kind": "insert", "paragraph": "p158", "after": "indemnities", "text": ", confidentiality obligations"},
    {"kind": "delete", "paragraph": "p158", "find": " completion of such Services"},
    {"kind": "rewrite", "paragraph": "p12", "text": "<whole new paragraph text>"},
    {"kind": "comment", "paragraph": "p159", "find": "Termination", "text": "..."},
    {"kind": "insert_paragraph", "paragraph": "p160", "position": "after", "runs": [{"text": "..."}]},
    {"kind": "delete_paragraph", "paragraph": "p155"},
    {"kind": "reply_comment", "comment_id": "c53", "text": "..."},
    {"kind": "resolve_comment", "comment_id": "c42"}
  ]
}
```

**How a plan is checked and applied:**
- The plan is validated as a whole: `--dry-run` reports per-op status, and the real run writes the file only if every op succeeds. This is "all-or-nothing".
- `resolve_revisions` runs before the operations.
- `existing_revisions` controls what happens to other authors' pending revisions: keep, accept first, reject first, or refuse.

## How it is used in the benchmark (context only)

- **The task:** an agent (Claude Code harness) represents one party across four negotiation turns of a commercial contract.
  - Turn 1: mark up a clean template.
  - Turns 2–4: respond to the counterparty's redline, which carries their pending insertions, deletions and comments.
- **The constraints:** the agent has 60 minutes and must deliver `/app/contract.docx` with its own tracked changes and comments.
- **The grading:** an LLM grades it against rubric items written by practicing attorneys, e.g. "Rejects in Section 8.1 the insertion of '…'" or "Replaces 75% with 100% in Exhibit A".

## Data: three full runs of the same model, same tasks, same judge

All runs: model `muse-spark-1.3-contributor`, reasoning effort max, 60-minute budget, graded by Claude Haiku 5.5. All three runs completed all 140 tasks.

| per trial, all 140 tasks | original skill (python-docx scripts, no jubarte) | jubarte, older build | jubarte 0.12.0 |
|---|---|---|---|
| mean score | 0.313 | 0.351 | 0.347 |
| empty deliverable (no own tracked change) | 38 | 32 | 26 |
| hit the 60-min limit | 78 | 33 | 33 |
| delivered something despite timing out | 50 | 5 | 9 |
| agent minutes | 48.8 | 36.1 | 35.0 |
| input / output tokens | 3.98M / 226k | 2.93M / 181k | 3.08M / 176k |
| assistant turns | 47.7 | 34.5 | 35.9 |
| tool calls | 58.1 | 39.1 | 42.7 |
| edit/add calls refused or failed | n/a | 19.0% | 13.2% |

The score differences are not statistically significant (sign test p = 0.15–0.49). Quality when something is delivered is about the same in all three (0.43–0.46).

**What jubarte clearly buys:**
- Speed: half the timeouts.
- About 25% fewer turns, tool calls and tokens.

**What it costs:**
- Delivered work is lost at the time limit. The python-docx scripts saved after every call; a jubarte plan is all-or-nothing, so a timeout usually means an empty document.

## Observed problems, with evidence

Case folders sit next to this file. Each has the task input, the agent's output `.docx`, the grade, and the jubarte call timeline with the exact error messages. `README.md` and `cases.json` index them.

1. **Edits inside another author's pending insertion are refused.** This is the most frequent error: about three quarters of all refusal codes in 0.12.0 (111 of 146 at one count).

   ```
   error: plan refused: UNSUPPORTED_STRUCTURE (op-2): the text sits inside r366, a tracked insertion by
   "Trainer 1 - LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r366"]}}, or reject),
   or comment on it
   ```

   - Word itself lets a reviewer delete or replace text inside someone else's insertion: it nests the reviewer's `w:del` inside the other author's `w:ins`.
   - In 5 of 0.12.0's trials (4 in the older build), the agent's first edit attempt came at 44–59 minutes, was refused this way, and the trial ended with nothing delivered.
   - The same family also blocks "delete a paragraph that holds tracked changes" under `keep`.
   - **It also refuses edits to the agent's own pending insertions.** Example: s3-t4-g13a, 0.12.0. At 10.0 min jubarte refused an edit "inside r242, a tracked insertion by 'Reviewing Counsel (GiantCo)'". That is the agent's own author, and the input held no revisions by that author, so r242 came from the agent's own write at 8.8 min.
     - Word lets an author edit their own pending insertion freely: the inserted text simply changes, and no nested revision is needed.
     - This happened in 13 old-build and 8 new-build trials, about 3–5% of the inside-revision refusal messages.

2. **Accept and reject leave no trace.**
   - `resolve_revisions.reject` does what Word's Reject button does: the counterparty's insertion simply disappears. The other side then cannot see that we rejected it.
   - In multi-round negotiations, lawyers often prefer the walk-back to stay visible: their insertion struck through by us, or their deleted text re-inserted by us.
   - Because problem 1 refuses the visible route, and its hint steers toward settling, agents reject silently.
   - Measured: in 13 delivered documents (10 in 0.12.0), rubric items whose required outcome was achieved still failed, because nothing in the document showed the rejection. That is 120 rubric weight in 0.12.0.
   - Verified by locating the quoted text: it was a counterparty `w:ins` in the input and is absent from the output, with no tracked deletion.
   - Separately, settling only (accept/reject with no own edits) produces a document with no tracked change by our author.

3. **All-or-nothing plans.**
   - One bad op discards the whole batch. With the clock running, this became zero output.
   - Agents also composed one giant plan and kept extending it. In several trials the plan dry-ran clean many times and was never applied before the timeout.
   - `SKILL.md` encourages this: "execute in one or two jubarte passes".
   - It says "Only the failed one needs fixing; don't reapply the ones that already succeeded", while also saying nothing was written. Agents read these two statements differently.

4. **Regenerating a redline with `compare` erases other parties' revisions.**
   - We tested a "copy, edit freely, then regenerate the redline" workflow: `jubarte compare task_input.docx agent_final.docx -a "Reviewing Counsel (AgentCo)" -o fresh.docx`.
   - The rejections became visible as our deletions. But all 442 pending revisions by "Trainer 1 - LargeCo" in the input disappeared, flattened into plain text or into our revisions. The compare reads the input with LargeCo's changes accepted, as Word's Compare does.
   - LargeCo's own comment survived. One of our 19 comments was lost: c118 ("We removed this definition…"), anchored entirely to the 2.24 "User Information" definition we had deleted. The output regenerates 2.24 as our deletion but does not re-attach the comment to it.
   - This matches the usual convention of marking a redline against the last draft received: the base reads as LargeCo's draft, so the output is a legitimate deliverable. What it drops is the round's provenance:
     - a strike of LargeCo's new text looks the same as a strike of previously agreed text;
     - every LargeCo change not struck is implicitly accepted, and none can stay pending;
     - comments anchored to LargeCo's deletions lose their anchor (their c106 ends up attached to nothing).
   - So `compare` works as a "decide the text, then render the redline" back end only if those losses are acceptable. Keeping the provenance needs a revision-aware compare.

5. **Option conflicts that the docs themselves teach.**
   - `read --no-page-markers` with `--json`: at least 165 refusals in 78 of 140 trials in 0.12.0 (43 in 33 trials in the older build). The quick start teaches one, the exact-text recipe the other.
   - Agents usually pipe `--json` output into a parser with `2>&1`, so the refusal shows up only as a JSON parse error, and the agent retries blind.
   - `--tables` with `--paragraphs`: 9 times; `--track-changes` with `--json`: 7.

6. **Validation noise.**
   - `BOOKMARK_CROSSES_REVISION`: jubarte's own edit puts an existing `_Hlk…` bookmark end inside its `w:ins`. The input validated clean.
   - `COMMENT_METADATA_MISMATCH`: the `commentsExtensible` durableId is missing, already present in the inputs.
   - `SKILL.md` requires "no findings", so agents spend 3–4 minutes in raw XML chasing findings they did not cause.
   - `read`, `changes` and `comments` also repeat these input-level warnings on every call, in 23 of 0.12.0's trials.

7. **Plan schema friction.**
   - Errors seen: `INVALID_PLAN unknown field "comment"` on `delete`/`rewrite`, `comment` instead of `text` on comment ops, missing `schema_version`, trailing JSON characters.
   - The error does not list the valid fields.

8. **Unclear messages.**
   - `PARAGRAPH_RESOLVED`: "p5 merged into p5 when the plan's revisions were settled; an operation on a whole paragraph needs its settled id". The same id appears twice, and the code is undocumented.
   - `UNKNOWN_COMMENT` after settling a revision that a comment was anchored to.

9. **Reading cost.**
   - Agents first read the 63 KB `jubarte-help-all.md`.
   - On turn 2–4 documents they page through `read` output 5–10 times, because every pending revision is inlined.
   - Median time to the first edit attempt was 21–25 minutes of a 60-minute budget.

10. **Accepting a deletion that holds a comment's reference mark deletes the comment silently and orphans its replies.**
    - Reproduce on the s1-t3-g15a task input: `jubarte edit contract.docx -a "Reviewing Counsel (AgentCo)" -p r13 --accept -o out.docx`.
    - r13 is a counterparty deletion whose only content is the run carrying comment c4's `w:commentReference`.
      - `changes` lists it as an empty deletion (`r13 deletion text ""`), and the accept reports `accepted r13 in p23: "{----}" > ""`.
      - Nothing says that c4 goes with it.
    - After the accept:
      - Comment c4 ("Term is not defined in this agreement…") is gone from `comments.xml`, `commentsIds.xml` and the document, although its anchor text is still there.
      - Its reply c5 survives. `commentsExtended.xml` keeps `w15:paraIdParent="706324C5"` pointing at the deleted parent, and `validate` then reports `commentsExtended paraIdParent '706324C5' does not resolve`.
    - It happened in a delivered document (0.12.0, s1-t3-g15a). The agent had earlier added its own reply in that thread; that reply ended up orphaned too.
    - Open questions:
      - What Word does when the deletion holding a comment's reference mark is accepted: we believe it deletes the whole thread, but this needs a Word experiment.
      - Whether jubarte should then delete the replies, keep the comment re-anchored, or refuse with a message that names the comment.

11. **Replying to a comment in a deleted paragraph is refused, under the wrong code.**
    - `OVERLAPPING_EDITS (op-29): replies to a comment whose reference is in a deleted paragraph`: 7 refusals across both builds. Nothing overlaps.
    - Word lets a reviewer reply to a comment whose anchor sits in deleted text. Replying there is a natural way to explain why we struck the paragraph.

12. **Text cannot be inserted into an empty paragraph.**
    - `UNSUPPORTED_STRUCTURE: cannot insert text into an empty paragraph; use insert_paragraph`: 5 trials in 0.12.0, 4 in the older build.
    - In Word, typing into an empty paragraph is ordinary.

13. **`read` and `changes` disagree on what one revision is.**
    - In s1-t3-g15a, one counterparty insertion is four `w:ins` elements, split at run boundaries, once mid-word ("…and shall re" | "main in effect…").
    - `read` shows one change, `r7+8+9+10`, and settling that composite id works. `changes` lists r7, r8, r9 and r10 separately.
    - An agent working from `changes` that rejects r7 leaves the fragment "main in effect until terminated…" behind as a pending insertion.
    - The CLI refuses the natural variant `-p r87+r88` with "--accept and --reject take a revision (-p r12)". The message does not show the accepted composite form.

Items 14–23 were found by running jubarte 0.12.0 directly on the task inputs. The commands are in `repro/repro.sh`, the outputs in `repro/repro_output.txt`. Each was checked in the XML and against the text that Accept All / Reject All gives.

14. **`replace` changes the formatting of text it keeps.**
    - The agent restates the anchor with one word changed. jubarte keeps the unchanged characters but gives them the first run's formatting, as a tracked formatting change by our author.
    - Repro (s1-t1-g01a): `edit -p p6 --anchor '“Affiliates” means any' --content '“Affiliates” means each'`. The result carries a `w:rPrChange` on "Affiliates" whose old `w:rPr` has `<w:b/>`: the defined term is un-bolded.
    - Seen in delivered documents: 5 defined terms in 4 trials of 0.12.0 lost bold this way ("CPRA", "PCI", "PCI DSS", "Uptime Commitment"). They came from plain `replace` ops with long `find` spans; the agents asked for no formatting change.
    - If the replaced span mixes formatting ("Interpretation**.** If any"), the new text takes the first run's formatting only.

15. **A counterparty's comment can be rewritten under their name.**
    - `edit contract.docx -a "Reviewing Counsel (AgentCo)" -p c106 --content "We will share PHI with AgentCo."` (s2-t2-g03c) turns LargeCo's "We do not intend to share any PHI with AgentCo." into its opposite.
    - The comment keeps `w:author="Trainer 1 - LargeCo"` and its original date. There is no warning, and `validate --original` does not notice.
    - Editing comment text is a documented feature (`-p c5 --content "Agreed."`), with no check that the comment belongs to `--author`. We believe Word's modern comments only let you edit your own; this needs checking.

16. **A bare `accept` or `reject` renumbers every comment id.**
    - `jubarte reject` on s3-t4-g13a turns c7 into c6, c9 into c7 and c12 into c9 (`w:id` rewritten in `comments.xml`). Targeted `edit -p rN --accept` keeps the ids.
    - An agent that noted ids before the settle then replies with `-p c9 --reply` into a different thread.
    - Plans are protected by `source_sha256`; single CLI operations are not.

17. **`read` misattributes adjacent changes.**
    - **Wrong pairing (s2-t2-g03c p47):** after we replace "non-exclusive" right after LargeCo's insertion "limited, ", the view prints `{~~non-exclusive~>limited, ~~}{>>r524 @AL<<}{>>r141 @TX<<}{++nonexclusive++}`. The XML order is LargeCo ins "limited, " → our del "non-exclusive" → our ins "nonexclusive". The view fuses our deletion with their insertion into one substitution and reverses the order.
    - **Lost nesting (s1-t3-g03a p8):** a counterparty deletion inside a multi-run insertion is shown as `{--applicable--}{>>r16 @TX2<<}`, without the `on r13` that `changes` reports ("inside r13"). r13 itself shows only its leading space, so the agent cannot see that "applicable" was AgentCo's insertion.

18. **`insert_paragraph` with `style` still inherits the neighbour's numbering and bold.**
    - Repro (s1-t1-g01a): `{"kind":"insert_paragraph","paragraph":"p25","position":"after","style":"Normal","runs":[{"text":"Unnumbered para after heading."}]}` after heading "3. SERVICES AND SUPPORT".
    - The new paragraph gets `numPr` (numId 1, ilvl 0) and bold runs. It becomes "4.", and every later article renumbers, which breaks cross-references.
    - `SKILL.md` documents `style` and `like` as the way to set the format.

19. **Paragraph-level insert and delete are diffed character by character across the paragraph boundary.**
    - **Delete p14 (s1-t1-g01a):** besides the deletions, jubarte writes an insertion of "“" in p14 and a deletion of "“" in p15.
    - **`insert_paragraph` before a definition:** the new paragraph starts with the old paragraph's untracked "“", and the next paragraph gets our inserted "“".
    - Accept All and Reject All give the right text. A partial settle does not: if the counterparty rejects only our new paragraph, the next definition reads "““User” means". Not seen in the delivered documents.

20. **Numbering side effects.**
    - An edit can add `w:num` entries with duplicate `w16cid:durableId` values. This happened in 3 delivered documents (s2-t1-g01a/b/d: 0 → 3 duplicates).
    - After an accept-all on s2-t3-g06a, a single `replace` in p212 re-pointed 17 untouched list paragraphs to new numbering instances (`numId` 13 → 25). Each became a tracked formatting change by our author, and 8 duplicate durableIds were added.
    - How Word treats duplicate durableIds needs testing.

21. **A paragraph added after one whose mark is a pending deletion merges into it on Accept All.**
    - Repro: `add -p p3 --content "NEW PARAGRAPH AFTER P3."` on s2-t2-g03c, where LargeCo deleted p3's paragraph mark. After Accept All, p3 reads "…agree as follows:NEW PARAGRAPH AFTER P3.".
    - What Word does when you press Enter at that spot needs checking. Either way, the agent asked for a separate paragraph.

22. **Lenient anchor matching rewrites typography.**
    - Straight quotes or plain spaces in `--anchor` match curly quotes or a non-breaking space. The straight characters in `--content` then replace them as tracked changes.
    - s1-t1-g01a p6: a one-word edit produced 7 changes, including swapping both curly quotes and un-bolding the term.
    - p16: `(1) any officer` → `(1) every officer` replaced the non-breaking space.
    - Characters that matched leniently should be kept as they are in the document.

23. **Smaller issues.**
    - `changes --json` drops tabs from revision text: "Position: \t\tPosition:" is reported as "Position: Position".
    - `validate --original INPUT --author US` on an untouched turn-2 input reports 518 `FOREIGN_AUTHOR` errors, one per counterparty revision. That makes the edits check useless on negotiation documents.

**Out of jubarte's scope, but relevant to designing for agents:**
- About 0.4–0.8 turns per trial were 32k-token thinking-only turns, with no text and no tool call.
- The task instructions never state the time limit.

## What we want from you

**A. A prioritized list of changes to jubarte.** Cover the engine, CLI, plan schema, errors, `validate`, and `SKILL.md` / help text. For each change give:

1. **Problem:** what goes wrong today, citing the evidence above or in the case folders.
2. **Who benefits and how:** the lawyer in Word, the agent, or both. Keep this concrete.
3. **Design:**
   - The exact CLI flags or plan fields, with defaults and backward compatibility.
   - The OOXML produced. For example: how a reviewer's deletion inside another author's insertion is written; how a visible reject of their insertion and a visible restore of their deletion look; and how moves, paragraph marks, numbering, tables, fields, bookmarks and comments anchored in the affected range are handled.
4. **Word fidelity:** confirm the output matches what Word itself produces for the same manual action. It must open in Word with no repair prompt, and Accept All / Reject All must still give the right text for every author.
5. **Edge cases and failure modes:** what still has to be refused, and what the error should say, including the exact command to run instead.
6. **How to verify without this benchmark:** fixtures, round-trip tests through Word, and property tests (e.g. "Reject All on our author restores the counterparty's version").
7. **Effort and risk:** small, medium or large, and what could break.

**B. Your view on these specific design questions:**
- Should a visible reject be the default when responding to a counterparty's pending change, or an explicit choice (`reject --visible`, a `strike` op, `existing_revisions: "layer"`)? What do practicing lawyers expect in each negotiation round?
- Should plans support partial application? For example: `--apply-ok` writes the ops that passed and reports the rest, or a checkpoint per batch. How do we keep this from producing half-coherent documents? Should `edit` print the exact apply command after a clean dry-run?
- Can `compare` become revision-aware (three-way)? The target behaviour: keep every existing revision attributed to its author, and lay our differences on top. That would make "edit a clean working copy, then regenerate `contract.docx` after each batch" a safe primary workflow, removing the whole `UNSUPPORTED_STRUCTURE` class. What are the hard cases, and is it worth it compared with fixing in-place editing?
- `validate --baseline input.docx`: report only findings introduced by our edits. Should jubarte also stop creating `BOOKMARK_CROSSES_REVISION` by keeping existing bookmark markers outside generated runs?
- What should a summary-first `read` look like for documents with over 1,000 pending revisions? For example `read --changed`, or grouping by author, section and kind with ids, so an agent can plan its response without paging the whole document.
- Which plan-schema guesses should be accepted as aliases (e.g. `comment` on `delete`/`rewrite`, `comment` → `text`), and which should stay errors with a better message?

**C. What not to do.** List any change you would reject because it optimizes for an LLM judge or for these rubrics rather than for real use. Examples: making settles look like edits to satisfy a validity gate, or inventing tracked changes that no lawyer would make.

**Constraints:**
- Deterministic output.
- Never rewrite or re-attribute other authors' revisions unless the user explicitly asks.
- Never produce a file Word wants to repair.
- Prefer fixing the engine over adding instructions the agent must remember.
- When you are unsure how Word behaves, say so and describe the experiment that would settle it.

Answer in Markdown. Put the prioritized change list first, as a table with one row per change: rank, change, benefit, effort. Then give the detailed sections.
