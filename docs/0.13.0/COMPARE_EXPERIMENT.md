# Experiment: regenerate the 13 invisible-settle deliverables with `jubarte compare`

Run 2026-10-18. Question: if the agent's final document is turned into a fresh redline against the task input, do the rejects that jubarte made invisible become visible, and what does that cost?

## Method

- For each trial in `C-invisible-settle/`: `jubarte compare task_input.docx agent_output_redline.docx -a "<agent author>" -o compare_redline.docx --force` (jubarte 0.12.0).
- The compare output was rendered with the verifier's own `annotated_view.py`. It was graded by the same judge (Claude Haiku 5.5 via Kilo) against the same rubrics.
- **Control:** the original annotated view was sent to the same judge a second time, to measure judge noise.
- **The judge sees only the delivered document's annotated view, never the task input.**
- Per case folder: `compare_redline.docx`, `compare_haiku_grade.json`, `control_haiku_grade.json`.
- Scripts: `scripts/compare_rejudge.py`, `scripts/compare_rejudge_report.py`, `scripts/run_judge.sh`. Full output: `scripts/compare_experiment_output.txt`.

## Result

| build | task | original | control rerun | compare | invisible-settle items passing (orig → compare) |
|---|---|---|---|---|---|
| old | s1-t3-g14a | 0.643 | 0.643 | 0.622 | 0/1 → 1/1 |
| old | s2-t4-g09a | 0.000 | 0.000 | 0.349 | 0/3 → 3/3 |
| old | s3-t4-g02a | 0.463 | 0.463 | 0.512 | 0/1 → 1/1 |
| new | s1-t2-g02a | 0.000 | 0.000 | 0.243 | 0/2 → 2/2 |
| new | s1-t2-g02c | 0.080 | 0.080 | 0.506 | 0/4 → 3/4 |
| new | s1-t3-g12a | 0.493 | 0.597 | 0.627 | 0/1 → 1/1 |
| new | s1-t4-g10a | 0.565 | 0.565 | 0.681 | 0/1 → 1/1 |
| new | s1-t4-g12a | 0.543 | 0.543 | 0.771 | 0/1 → 1/1 |
| new | s2-t2-g02a | 0.407 | 0.496 | 0.504 | 0/1 → 1/1 |
| new | s2-t2-g03c | 0.094 | 0.094 | 0.333 | 0/3 → 3/3 |
| new | s2-t2-g04c | 0.446 | 0.446 | 0.535 | 0/1 → 1/1 |
| new | s2-t4-g06a | 0.104 | 0.493 | 0.493 | 0/1 → 1/1 |
| new | s3-t2-g03b | 0.614 | 0.693 | 0.557 | 0/1 → 1/1 |
| | **mean** | **0.342** | **0.393** | **0.518** | **0/21 → 20/21** (control 3/21) |

- **Judge noise:** the control moved four trials on its own. The worst was s2-t4-g06a, which went from 0.104 to 0.493 with the identical view. Compare against the control, not the original: +0.125 on these 13.
- **Selection bias:** these 13 were picked because they lost points to invisible rejects. This is the best case for compare, not an estimate for all trials.

## What the compare fixed

- 20 of the 21 invisible-settle rubric items now pass. Examples:
  - "Rejects in Section 8.1 the insertion of 'and each of their respective parents…'": the judge now sees the text struck by us.
  - Silently restored text now shows as our insertion.
- **It also exposed one judge error.** In old s3-t4-g02a the agent had silently removed AgentCo's insertion "applicable". The original and control grades both passed "Accepts … 'applicable'". With the removal visible, the judge correctly failed it.

## What the compare cost

- **Accepted deletions vanish.** The compare reads the input with the counterparty's changes accepted, so every pending counterparty deletion the agent let stand simply disappears. Six rubric items (37 weight) phrased "Accepts the deletion of …" went PASS → FAIL, because the judge finds no deletion:
  - s2-t2-g02a: PHI definition, w8
  - s2-t2-g04c: PHI definition, w10
  - s3-t2-g03b: 4.8 damages for delay (w3); 9.1 hourly fee schedule (w9)
  - old s1-t3-g14a: 1.12 (w5); 2.3.1 (w2)
- This is the mirror image of the invisible-reject problem. Two-way compare makes our rejects visible and our acceptances invisible.
- A lawyer reading a "redline against your last draft" would read the missing text as accepted, by convention. A reader without the previous draft cannot.

## Takeaway for jubarte

Neither "edit in place with invisible settles" nor "two-way compare" shows both what we rejected and what we accepted. A revision-aware compare (or in-place editing with visible rejects) that keeps the counterparty's pending changes, with their authorship, and lays our changes on top would show both.
