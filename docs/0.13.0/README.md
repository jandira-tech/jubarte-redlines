# When jubarte caused the lost score: muse-spark-1.3-contributor, two jubarte builds

`QUESTION_FOR_LARGE_MODEL.md` is a self-contained brief asking a large model which jubarte changes would make the tool useful (not benchmark-tuned); send it together with this folder.

Built 2026-10-18 from:
- old build: `reproduce_out/jobs/muse-1-3-contributor-total` (140 trials)
- new build: `reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18` (113 trials finished)

The new run's other 27 tasks (17 never started, 10 cancelled) are being re-run now and are not included.

Both runs were rejudged with the same judge: Claude Haiku 5.5 through the Kilo gateway (`reproduce_out/rejudge/`).
Every case below was checked against the agent's actual `.docx` deliverable.

## Folders

| folder | meaning | old | new |
|---|---|---|---|
| `A-blocked-with-time-left/` | started editing before 40 min; jubarte refused every attempt; never got a clean dry-run; nothing landed (score 0) | 1 | 0 |
| `B-final-blow-late-start/` | first edit attempt came at 44–59 min and jubarte refused it; nothing landed (score 0). The late start is the agent's fault; the refusal finished the trial off | 4 | 5 |
| `C-invisible-settle/` | deliverable landed, but rubric items failed even though the text outcome was right. jubarte's reject/accept removes the counterparty's change with no tracked trace, so the judge sees no rejection | 3 trials, 29 weight | 10 trials, 120 weight |

Each case folder contains:
- `agent_output_redline.docx`: the agent's deliverable
- `task_input.docx`: the document the agent started from
- `haiku_grade.json`: Haiku 5.5's per-rubric verdicts
- `README.md`: the jubarte edit timeline with the exact refusal messages; for C cases, also each lost rubric item, where its text sits in the input vs the output, and the judge's words

`cases.json` is the machine-readable list.

## How the cases were picked

- **A/B** (`jubarte_cause_cases.py`, `jubarte_blame.py`):
  - The deliverable is empty (validity gate: no tracked change by the agent's author).
  - At least one jubarte `edit`/`add` call was refused.
  - Either no dry-run ever came back clean, or the last attempt was the refusal and nothing was written.
  - Not counted: trials that got a clean dry-run and never applied it. Those are model-side.
- **C** (`silent_settle_scan.py`):
  - Covers every FAILed rubric whose criterion quotes contract text.
  - The quoted text was a counterparty tracked insertion (or deletion) in the input and is gone (or silently restored) in the output.
  - The rubric verb asks for exactly that outcome: Rejects/Deletes/Removes/Restores.
  - Not counted: "Accepts X" rubrics where the agent rejected X. That is a wrong legal call, not jubarte.
  - No trial used python-docx or a raw-XML bypass; every settle went through jubarte.

## What jubarte cost

**Empty deliverables.** jubarte was the primary cause of 1 of 32 old-build empties (`s2-t2-g01a`: 6 refusals from 26 min on, the last one `PARAGRAPH_RESOLVED` at 58.9 min). It was the primary cause of 0 of 25 new-build empties. It was the final blow on 4 + 5 late starts.
- Two of the five new-build final blows (`s1-t2-g04b`, `s1-t3-g14a`) are tasks the old build had landed. They are among the 14 regressions.
- Every new-build refusal here is the same one: `UNSUPPORTED_STRUCTURE: the text sits inside rN, a tracked insertion by "Trainer 1 - …"; settle it first`.

**Invisible settles.** The new build lost 120 rubric weight in 10 landed trials, against 29 weight in 3 trials for the old build. If those items had passed, the new build's mean score would rise by up to +0.014, the old build's by up to +0.004.
- The new build hits this more because its agents settle more (they reject the counterparty's insertion instead of being refused and giving up).
- Worst cases: `s1-t2-g02c` (28 of 87 weight), `s2-t2-g03c` (23 of 96), `s1-t2-g02a` (18 of 74).

**Fix** (also in `reproduce_out/babysit/jubarte-usage-report.md`):
- A visible reject: nest the reviewer's `w:del` inside their `w:ins`, as Word does when you delete inside someone's insertion. Re-insert their deleted text as our tracked insertion.
- Allow `replace` inside a counterparty insertion instead of refusing it.
- Change the refusal hint so it no longer says "settle it first".

## The two runs side by side (same Haiku 5.5 judge)

The paired columns cover the 113 tasks finished in both runs. Cost is Claude Code's own estimate, not the gateway's bill.

| metric | old (paired) | new (paired) |
|---|---|---|
| mean score | 0.3241 | 0.3224 |
| empty deliverables | 30 | 25 |
| never attempted an edit | 15 | 8 |
| 60-min timeouts | 31 | 32 |
| agent minutes, mean | 38.9 | 37.3 |
| input tokens/trial (M), cache reads (M) | 3.07, 2.83 | 3.30, 3.12 |
| output tokens/trial mean, median (k) | 195, 188 | 189, 170 |
| cost/trial (USD, est.) | 6.57 | 6.94 |
| assistant turns/trial | 35.6 | 37.6 |
| turns at the 32k output cap/trial (all thinking-only) | 0.64 | 0.42 |
| tool calls/trial | 40.6 | 45.3 |
| jubarte calls/trial | 36.7 | 45.5 |
| jubarte runtime/trial (min) | 0.25 | 0.16 |
| edit/add attempts/trial | 6.6 | 9.6 |
| edit/add refused or failed | 19.0% | 13.2% |
| trials with >=1 refusal | 67 | 78 |
| first edit attempt, median (min) | 25.4 | 20.7 |

**Score changes:**
- 46 wins, 39 losses, 28 ties (sign test p = 0.52).
- 19 rescues (old empty → new landed, mean 0.371) and 14 regressions (old landed → new empty; old mean 0.490).
- On the 69 tasks both runs landed, the new build is −0.005.
