# redline-s2-t2-g02a (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s2-t2-g02a__YpQNo88`
- Haiku 5.5 score: 0.4071
- agent exception: None
- first edit attempt: 8.3 min
- successful writes to /app/contract.docx at: [19.3, None, None, None, None, None, None, None, None]

## Why jubarte

1 rubric item(s), weight 9 of 113, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=9** (silent reject of their insertion) — Rejects in Section 9.1 the change of language from "warrants" to "represents, to the best of Customer's knowledge."
  - in task input: `current:ins:Trainer 1 - LargeCo`; in agent output: `absent`
  - judge: §9.1 (p-053) has no tracked change rejecting the knowledge-qualified 'represents' language; the flat warranty appears only as unmarked text.

## jubarte edit/add timeline (minutes from agent start)

- 8.3 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-1): the text sits inside r93, a tracked insertion by "Trainer 1 - LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r93"]}}, or reject), or comment on it`
- 10.5 dry-run ok
- 14.4 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-8): the insertion point sits inside r77, a tracked insertion by "Trainer 1 - LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r77"]}}, or reject), or comment on it`
- 16.1 dry-run ok
- 17.3 dry-run ok
- 19.3 write ok
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
- 27.5 dry-run FAILED STALE_SOURCE
  - `error: plan refused: STALE_SOURCE: source bytes do not match the plan's source_sha256`
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
