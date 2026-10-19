# redline-s3-t4-g02a (old jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total/redline-s3-t4-g02a__FF93f3W`
- Haiku 5.5 score: 0.4634
- agent exception: None
- first edit attempt: 5.0 min
- successful writes to /app/contract.docx at: [5.0, 10.7, 16.5]

## Why jubarte

1 rubric item(s), weight 9 of 82, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=9** (silent reject of their insertion) — Rejects in Article 7, Section 7.5 AgentCo’s insertion of “at mutual locations”
  - in task input: `current:ins:Trainer 1 - AgentCo`; in agent output: `absent`
  - judge: The phrase 'at mutual locations' does not appear anywhere in Section 7.5 (p-100), so no rejection is present.

## jubarte edit/add timeline (minutes from agent start)

- 5.0 write ok
- 10.7 write ok
- 13.5 write FAILED UNKNOWN_CHANGE
  - `error: plan refused: UNKNOWN_CHANGE: resolve_revisions: no tracked change r11`
- 16.5 dry-run ok
- 16.5 write ok
