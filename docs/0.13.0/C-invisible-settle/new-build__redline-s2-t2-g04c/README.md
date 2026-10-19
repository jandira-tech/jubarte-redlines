# redline-s2-t2-g04c (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s2-t2-g04c__h9yxBcY`
- Haiku 5.5 score: 0.4455
- agent exception: None
- first edit attempt: 14.0 min
- successful writes to /app/contract.docx at: [33.7]

## Why jubarte

1 rubric item(s), weight 9 of 101, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=9** (silent reject of their insertion) — Rejects in Section 8.1 the insertion of "or (ii) a breach by AgentCo of any term of this Agreement,".
  - in task input: `current:ins:Trainer 1 - LargeCo`; in agent output: `absent`
  - judge: No tracked change in Section 8.1 (p-053) strikes the 'or (ii) a breach by AgentCo' insertion; the clause is absent from the redline.

## jubarte edit/add timeline (minutes from agent start)

- 14.0 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-1): the text sits inside r87, a tracked insertion by "Trainer 1 - LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r87"]}}, or reject), or comment on it`
- 19.1 dry-run ok
- 32.0 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-4): cannot insert text into an empty paragraph; use insert_paragraph`
- 33.2 dry-run ok
- 33.7 write ok
