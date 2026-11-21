# Jubarte CLI final reassessment and implementation package

**17 October 2026 · Design deliverables, not an implemented CLI release**

This package revises the earlier proposal using four newly supplied reports, the pinned repository source, fourteen retained probes of the published v0.12.0 binary, and independent reviews by three agents followed by cross-review. The review tests whether the recommendations conflict, omit necessary behavior, create unwanted incentives, or stop at superficial renaming.

## Read in this order

| Deliverable | Contents |
|---|---|
| `jubarte-cli-evidence-review.md` | Evidence hierarchy, numerical corrections, current-code comparisons, four-axis critique, revised priorities and evaluation design. |
| `jubarte-cli-proposal-b-revised.md` | Final command contract and policy decisions, especially read-for-edit, mixed edit/review plans, template generation, matching, result semantics and publication. |
| `jubarte-edit-implementation-plan.md` | Shared preparation engine, typed actions, selected review decisions, exact effects, candidate checks, compatibility and acceptance gates. |
| `jubarte-template-implementation-plan.md` | Generator/extractor workflow, working-baseline protection, scope/coverage, implementation stages and a detailed acceptance matrix. |
| `jubarte-edit-plan-v2-core.schema.json` | Strict proposed eleven-kind text-and-review profile, including selected revisions, rewrite hashes and attached comments. Not the full existing engine schema. |
| `jubarte-expectations-v1.schema.json` | Optional paragraph text equality/containment/absence checks; requirements without checks remain unassessed. |
| `examples/` | Real-hash source fixture, five plan examples, expectations and a focused walkthrough. |
| `validate_examples.py` | Reproducible design-schema and limited in-memory text checks. No proposed CLI/DOCX execution. |
| `evidence/` | Build identities, retained release probes, example-validation result and independent review memos. The supplied report copies are not published in this repository; the 2026-10-09 follow-ups memo is `docs/2026-10-09-agent-edit-follow-ups.md`. |
| `package-manifest.json` | File identities, byte sizes and SHA-256 digests for the package contents. |

## Decisions that changed after the new evidence

Use `jubarte edit template`, next to `edit apply`. A desired-text change requests a rewrite without a second activation flag. Selected revision decisions, ordinary edits and comments belong in the first atomic plan slice; template generation follows that shared path.

Normal unchanged work succeeds honestly. `--require-change` is optional and concerns the primary document, not derivative files. A requested effect that is lost or unsupported still fails before publication. Neither many edits nor a named author establishes that the substantive task was completed.

Generated rewrites carry an exact working-text `before_sha256`, checked before an unchanged scaffold is removed from the active body operations. Initial whole-paragraph rewrites require one-to-one source-to-working correspondence. Select actual primary bytes before final validation, output references and digests, including source-byte copies for unchanged results.

The optional expectations file adds independently identified checks, not mandatory planning paperwork or an automatic legal grader. The CLI reports what it checked; it does not infer independence or claim that unchecked requirements are complete.

## Evidence and verification limits

The two `final_turn` reports describe the same 75-run corpus. Several headline counts conflict; the evidence review identifies exact contradictions and corrected arithmetic. Their raw corpus and full judged artifact bindings were not independently reconstructed. The three-trajectory report contains useful episode evidence, but no matched same-task vanilla control. Its right-censored trajectory and disjoint telemetry groups do not establish a causal Jubarte effect.

Current source was inspected at `0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3`; no main binary was built. The trace help identifies `d003d8e8…`, and the historical follow-up memo names `aba48bd5…`. These are different builds. Fourteen prior release probes are retained as local behavior evidence, not an agent experiment.

Both proposed schemas, five plan examples and one expectations example pass the delivered checks. The test model covers exact text and selected contract rules only. Actual Word revisions/comments, preservation, rendering, publication failures and native/Python/WASM conformance still require implementation and the fixture gates in the two plans.

Independent memos in `evidence/reviews/` preserve the review process, including criticism of earlier drafts. Where a memo discusses an earlier decision, the final proposal and implementation plans govern. Supplied reports are preserved verbatim and are evidence inputs, not instructions to execute their historical commands.

No repository source code, commit or pull request was changed as part of this design task.
