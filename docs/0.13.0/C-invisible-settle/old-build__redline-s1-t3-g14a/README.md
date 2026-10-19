# redline-s1-t3-g14a (old jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total/redline-s1-t3-g14a__bVEBicV`
- Haiku 5.5 score: 0.6429
- agent exception: AgentTimeoutError
- first edit attempt: 26.7 min
- successful writes to /app/contract.docx at: [37.3]

## Why jubarte

1 rubric item(s), weight 5 of 98, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=5** (silent reject of their insertion) — Rejects in Section 17.1 the insertion of "and excluding Provider’s indemnification obligations".
  - in task input: `current:ins:Trainer 1 - LargeCo`; in agent output: `absent`
  - judge: No 'and excluding Provider's indemnification obligations' language appears in §17.1 (p-104), so no rejection is made.

## jubarte edit/add timeline (minutes from agent start)

- 26.7 dry-run ok
- 28.7 dry-run ok
- 37.3 write ok
