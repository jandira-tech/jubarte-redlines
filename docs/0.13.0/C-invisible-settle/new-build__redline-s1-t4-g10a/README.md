# redline-s1-t4-g10a (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s1-t4-g10a__HW8X4uE`
- Haiku 5.5 score: 0.5652
- agent exception: None
- first edit attempt: 29.8 min
- successful writes to /app/contract.docx at: [None, None, None, None, None, None, None, None]

## Why jubarte

1 rubric item(s), weight 8 of 69, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=8** (silent reject of their insertion) — Rejects in Section 3.2 AgentCo’s insertion of “Conditioned on LargeCo’s compliance with the terms of this agreement”
  - in task input: `current:ins:Trainer 1 - AgentCo`; in agent output: `absent`
  - judge: The 'Conditioned on LargeCo's compliance' insertion does not appear anywhere in Section 3.2 (p-028), so there is no rejection.

## jubarte edit/add timeline (minutes from agent start)

- 29.8 dry-run FAILED INVALID_PLAN
  - `error: INVALID_PLAN: operations[16] (comment): unknown field "comment"`
- 30.0 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-1): the text sits inside r54, a tracked insertion by "Trainer 1 - AgentCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r54"]}}, or reject), or comment on it`
- 34.5 dry-run ok
- 38.8 dry-run ok
- 39.4 dry-run ok
- None write ok
- None write ok
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
- None dry-run ok
- None dry-run ok
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
- None dry-run ok
- None dry-run ok
- None write ok
- None write ok
