# redline-s2-t4-g09a (old jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total/redline-s2-t4-g09a__7hS5PjS`
- Haiku 5.5 score: 0.0000
- agent exception: None
- first edit attempt: 46.7 min
- successful writes to /app/contract.docx at: [53.9, None, None, 57.1, None, None]

## Why jubarte

3 rubric item(s), weight 15 of 43, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=5** (silent reject of their insertion) — Deletes from Section 9.3 ", representations, covenants, indemnities, Service Level Agreement obligations, privacy/data security obligations, no-training obligations, malware commitments, and DPA obligations".
  - in task input: `current:ins:LargeCo`; in agent output: `absent`
  - judge: Section 9.3 (p-071) contains no tracked deletion of the listed representation, covenant, SLA, privacy, and DPA language.
- **w=5** (silent reject of their insertion) — Deletes from Section 8.1 "; (b) AgentCo's breach of its representations, warranties, covenants, or obligations under this Agreement;".
  - in task input: `current:ins:LargeCo`; in agent output: `absent`
  - judge: Section 8.1 (p-047) has no tracked deletion of the '(b) AgentCo's breach of its representations...' clause.
- **w=5** (silent reject of their insertion) — Deletes from Section 8.1 "(c) AgentCo's breach of applicable law;".
  - in task input: `current:ins:LargeCo`; in agent output: `absent`
  - judge: Section 8.1 (p-047) has no tracked deletion of '(c) AgentCo's breach of applicable law;'.

## jubarte edit/add timeline (minutes from agent start)

- 46.7 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-2): the text sits inside r1749, a tracked insertion by "LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r1749"]}}, or reject), or comment on it`
- 50.1 dry-run ok
- 52.1 dry-run ok
- 53.9 write ok
- 55.9 dry-run ok
- None write ok
- None write ok
- 57.1 write ok
- None write ok
- None write ok
