# redline-s2-t2-g03c (new jubarte build)

- category: C-invisible-settle
- trial dir: `/Users/arthrod/temp/T/redline-bench/reproduce_out/jobs/muse-1-3-contributor-total-jubarte-oct-18/redline-s2-t2-g03c__Ak8z43j`
- Haiku 5.5 score: 0.0938
- agent exception: AgentTimeoutError
- first edit attempt: 21.9 min
- successful writes to /app/contract.docx at: [59.1]

## Why jubarte

3 rubric item(s), weight 23 of 96, failed although the text outcome was right: jubarte's reject/accept removed the counterparty change with no tracked trace.

## Rubric items lost to invisible settles

- **w=9** (silent reject of their insertion) — Rejects in Section 8.1 the insertion of "and each of their respective parents, subsidiaries, affiliates, sponsors, advertising agencies".
  - in task input: `current:ins:Trainer 1 - LargeCo`; in agent output: `absent`
  - judge: No tracked change rejects the "parents, subsidiaries, affiliates..." language; the insertion is absent from Section 8.1 (p-060).
- **w=5** (silent reject of their insertion) — Rejects in Section 8.1 the insertion of "except as otherwise expressly stated in this Agreement or to the extent such combination is necessary or reasonably contemplated for the use of the Service".
  - in task input: `current:ins:Trainer 1 - LargeCo`; in agent output: `absent`
  - judge: The "except as otherwise expressly stated" combination language is absent from Section 8.1 and no rejecting tracked change exists.
- **w=9** (silent reject of their insertion) — Rejects in Section 10.2 the insertion of "AgentCo’s breach of its confidentiality obligations, AgentCo’s breach of its data and information security obligations, ".
  - in task input: `current:ins:Trainer 1 - LargeCo`; in agent output: `absent`
  - judge: AgentCo keeps its own 10.2 proviso covering confidentiality and data security breaches; no rejection of the insertion appears.

## jubarte edit/add timeline (minutes from agent start)

- 21.9 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-2): the text sits inside r366, a tracked insertion by "Trainer 1 - LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["r366"]}}, or reject), or comment on it`
- 26.4 dry-run ok
- 49.1 dry-run ok
- 53.7 dry-run FAILED UNSUPPORTED_STRUCTURE
  - `error: plan refused: UNSUPPORTED_STRUCTURE (op-12): under keep, a paragraph that holds tracked changes cannot be deleted; resolve them first with resolve_revisions`
- 56.2 dry-run ok
- 59.1 write ok
