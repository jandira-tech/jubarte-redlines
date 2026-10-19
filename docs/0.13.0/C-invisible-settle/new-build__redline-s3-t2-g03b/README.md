# redline-s3-t2-g03b (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s3-t2-g03b__6hm3YWq`
- Haiku 5.5 score: 0.6136
- agent exception: RewardFileNotFoundError
- first edit attempt: 22.4 min
- successful writes to /app/contract.docx at: [25.6]

## Why jubarte

1 rubric item(s), weight 7 of 88, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=7** (silent restore (reject of their deletion)) — Restores in Section 7.7  "Consultant will not" use the trade name of GiantCo without permission.
  - in task input: `original:del:Trainer 1 - AgentCo`; in agent output: `current:plain | original:plain`
  - judge: The trade-name restriction at p-108 was never deleted and no tracked restoration appears, so nothing is restored.

## jubarte edit/add timeline (minutes from agent start)

- 22.4 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-5): the text sits inside r105, a tracked insertion by "Trainer 1 - AgentCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r105"]}}, or reject), or comment on it`
- 24.7 dry-run ok
- 25.6 write ok
