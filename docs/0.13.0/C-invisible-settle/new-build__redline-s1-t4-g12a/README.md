# redline-s1-t4-g12a (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s1-t4-g12a__F2VHNtj`
- Haiku 5.5 score: 0.5429
- agent exception: None
- first edit attempt: 9.9 min
- successful writes to /app/contract.docx at: [12.8, 15.3]

## Why jubarte

1 rubric item(s), weight 8 of 35, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=8** (silent restore (reject of their deletion)) — Rejects in Section 16 AgentCo’s deletion of “and excluding Provider’s indemnification obligations”.
  - in task input: `original:del:Trainer 2 - AgentCo`; in agent output: `current:plain | original:plain`
  - judge: The 'and excluding Provider's indemnification obligations' text in §17.1 (p-104) carries no tracked reinstatement or rejection markup; it simply sits as untracked base text.

## jubarte edit/add timeline (minutes from agent start)

- 9.9 dry-run FAILED ANCHOR_NOT_FOUND
  - `error: plan refused: ANCHOR_NOT_FOUND (op-10): "Application Law" does not occur in the paragraph`
- 11.8 dry-run ok
- 12.8 write ok
- 15.2 dry-run ok
- 15.3 write ok
