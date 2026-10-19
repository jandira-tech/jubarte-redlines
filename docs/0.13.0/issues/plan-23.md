@coderabbitai plan against the bench skill template (`skill-tools/SKILL.template.md` in jubarte-syntax) and the bench `instruction.md` lines 117 and 183, against the 0.13 CLI the other plans define; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 23: skill template (bench side)

**Stack.** Step E10 of the jubarte 0.13 contract-redlining plan; branch `(bench side: jubarte-syntax / neurotic_docx_bench)`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 23.

**Problem.** The skill teaches "one or two passes", "no findings", reading
the help-all first, and `resolve_revisions` as the answer to the
refusal (brief §C, `instruction.md:117,183`, bench
`.tasks/<task>/environment/skills/contract-redliner/SKILL.md`).

**Design (§A-detail 23).** The template at
`~/temp/T/jubarte-syntax/skill-tools/SKILL.template.md` says: read
`changes --summary` and `read --outline` first; plan a section, dry-run,
apply (`--skip-failed` when the clock is short), then the next section;
`strike`/`restore` for their changes you reject, `resolve_revisions`
only for changes you accept or want gone; `validate OUT --baseline IN`
once; with ten minutes left, apply what you have. Delete "one or two
passes" and "no findings". Not an engine change; tracked here so the
engine and the skill ship together.

**Files.** `~/temp/T/jubarte-syntax/skill-tools/SKILL.template.md`,
the bench's `instruction.md` (117, 183) and the generated SKILL.md; the
bench's verifier tools (memory: `redline-bench-jubarte-deploy`).

**Steps.**

- [ ] Rewrite the template against the 0.13 CLI (plans 2, 3, 6, 9, 22);
      every command in it is run once against a 0.13 build.
- [ ] Regenerate the bench SKILL.md; diff against the old one; the words
      "one or two passes", "no findings" and "help-all" are gone.
- [ ] One bench trial on a turn-2 document to confirm the first edit
      lands before minute 10.

**Depends on.** Plans 2, 3, 6, 9, 22 released.

**Related.** brief §C.

---

## Closing notes

- Three plans change settled text or behaviour that Arthur may want to
  keep: plan 17 stops printing the dialect's two-author substitution;
  plan 15 widens direct emission to every mode and re-pins comparer
  outputs; plan 14 stops renumbering comment ids on a bare settle. Each
  is flagged in its plan and its issue.
- Word experiments gate: E1 (plans 1, 2), E2 (1, 2), E3 (19), E4 (2, 19),
  E5 (14), E6 (16), E7 (21). Run them before the plan that needs them;
  ship the text-only part first when an experiment is late.
- Existing issues this plan references rather than duplicates: #1136,
  #1108, #1158, #1161, #1173, #1156, #827, #1159, #1192, #1150, #1134,
  #1111, #1135.
