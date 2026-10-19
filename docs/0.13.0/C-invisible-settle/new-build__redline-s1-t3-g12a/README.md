# redline-s1-t3-g12a (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s1-t3-g12a__2cadi9r`
- Haiku 5.5 score: 0.4925
- agent exception: AgentTimeoutError
- first edit attempt: 25.3 min
- successful writes to /app/contract.docx at: [25.3, 58.5, 59.6]

## Why jubarte

1 rubric item(s), weight 2 of 67, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=2** (silent restore (reject of their deletion)) — Rejects in Section 13.3 LargeCo's deletion of "PROVIDER MAKES NO WARRANTY OF ANY KIND THAT THE SERVICE WILL MEET LARGECO’S OR ANY OTHER PERSON'S REQUIREMENTS, OPERATE WITHOUT INTERRUPTION, ACHIEVE ANY INTENDED RESULT, BE COMPATIBLE WITH ANY SYSTEM, OR BE SECURE, ACCURATE, COMPLETE, FREE OF HARMFUL CODE, OR ERROR FREE"
  - in task input: `original:del:Trainer 1 - LargeCo`; in agent output: `current:plain | original:plain`
  - judge: The disclaimer tail at p-093 appears as untracked base text with comment cmt-722 claiming restoration, but no tracked change shows the rejection.

## jubarte edit/add timeline (minutes from agent start)

- 25.3 write ok
- 34.2 dry-run ok
- 43.4 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-2): the text sits inside r437, a tracked insertion by "Trainer 1 - AgentCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r437"]}}, or reject), or comment on it`
- 50.6 dry-run ok
- 52.6 dry-run ok
- 54.8 dry-run ok
- 56.3 dry-run ok
- 58.5 write ok
- 59.6 write ok
