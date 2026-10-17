# Reassessment: implement the editing contract before expanding the template

## Scope and conclusion

This is an implementation and experimental-design review of four newly supplied reports, read in full. It revises the earlier recommendation where the new evidence or existing code contradicts it. It does not reproduce the underlying agent runs, modify the repository, or treat report quotations as independently verified raw traces.

**The highest-priority change is an executable, discoverable transaction that combines selected revision decisions, body edits, and comment actions, with serialized-result checks before publication.** A template should generate that same transaction. It should not replace an already capable engine with a smaller public model that cannot express the work the traces required.

Keep `jubarte edit template` as the canonical name; keep one desired-text field and remove `run_operation`. But those are prospective interface improvements, not demonstrated causes or remedies for these runs. The reports contain no tested comparison involving `run_operation`. The immediate observed obstacles are discovering actual operation contracts, handling existing revisions, late detection of structural problems, managing large output, and distinguishing an applied plan from a completed task.

The other substantial correction is that **the inspected main source already executes the plan during dry run**. The missing capability is not simply “simulate the edit.” It is a shared, explicit check of the serialized candidate’s required invariants and requested effects. Adding separate public `plan check`, `plan simulate`, and `plan apply` ceremonies would make the workflow longer without fixing that implementation gap.

## Evidence references and build boundaries

Local report references below identify line ranges in the supplied files:

| Reference | Material and evidentiary role |
|---|---|
| F1 | `upload/final_turn_001.md`, 160 lines. First 75-run audit report. |
| F2 | `upload/final_turn_002.md`, 364 lines. Expanded version of the same 75-run audit, not another independent sample. |
| R17 | `upload/deep-research-report(17).md`, 1,078 lines. Report on three specific trajectories, two graded artifacts, and a separate generation-level telemetry sample. |
| FU | `upload/2026-10-09-agent-edit-follow-ups.md`, 440 lines. Implementation proposal with a later status audit. Its unchecked tasks are not evidence that the proposed code shipped. |
| B | `deliverables/jubarte-cli-proposal-b-revised.md`. Previous revised contract, especially §§4 and 9. |
| E | `deliverables/jubarte-cli-evidence-review.md`. Previous evidence review; its released-binary probes remain a separate evidence source. |

Code observations in this memo use the local checkout at **`0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3`**, unless explicitly labeled otherwise. F2’s quoted help identifies **`0.12.0-main.d003d8e8`** (F2:342); those experiment calls must not be equated with our earlier released-0.12.0 probes. FU’s header reports an audit at `aba48bd5`; its October 18/19 editorial dates do not establish a current installed runtime or the execution date of the experiments. The parent audit has separately verified those commit identities. This memo does not infer missing build behavior from a shared `0.12.0` version prefix.

Useful pinned source references:

- [Edit plan, revision settlement, execution, and preview](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit.rs): `resolve_selected` at 1122–1192; `settle` at 1221–1262; `apply_plan` at 1565–1650; `preview_plan` at 1686–1697; `Transaction::start` at 1951–2040; rewrite resolution at 3410–3517; range/opaque-structure diagnostics at 3845–3978.
- [Existing-revision tracked emitter](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit/tracked.rs): `check`, especially 84–167.
- [Original-paragraph correspondence](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit/origin.rs).
- [CLI orchestration](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs): `run_edit` at 946–1135; `run_validate` at 2320–2358.
- [Validation APIs and current failure predicate](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/validate.rs): `validate_with`, `fails`, and the documented finding taxonomy.
- Existing tests to extend: `tests/edit_plan.rs`, `tests/edit_resolved_ids.rs`, `tests/edit_opaque_refusals.rs`, `tests/edit_keep_revisions.rs`, `tests/edit_comment_text.rs`, and `tests/cli_plan_record.rs`.

## 1. What the new evidence establishes—and what needs correction

### Concrete mechanisms worth designing against

The reports show credible examples of agents searching a large help dump for fields the dump did not contain; making several incorrect guesses about comment-operation fields; reaching a real refusal after authoring a large plan; targeting p15 when the intended literal text was at p20; and interpreting a structurally valid artifact as a successful negotiation. These are useful reproduction targets even without treating the aggregate report statistics as authoritative.

R17’s bookmark case is particularly valuable: the report describes a successful dry run, then an applied scratch artifact with `BOOKMARK_CROSSES_REVISION`, followed by an insertion-position workaround and a successful recheck. That supports a serialized-output preservation regression test. It does **not** establish that every bookmark crossing is fatal to Word, that every warning should block publication, or that adding another named validation command would fix the underlying range-placement bug.

F2 describes a pipeline that mixes stderr into stdout and reports the exit status of `tail`, not Jubarte (F2:245–257). It also describes a claimed delivery path different from the paths actually used. Those support subprocess instrumentation and artifact identity checks. They do not show that Jubarte can make its process status survive arbitrary shell transformations.

### Material problems in the reports

| Claim or presentation | Necessary correction |
|---|---|
| F1 says 8 of 16 gate-failed runs never invoked edit/add; F2 says 14, then lists nine first-edit entries. | The classifications conflict. Several names in F2’s “never edited” list also appear in its first-edit table. Do not reuse the no-edit rate or its claimed median until recomputed from raw events. F2’s displayed nine percentages have a median of 64%, not its stated 89%. |
| F2 labels 37 positive-score runs with mean .319, then says all 40 gate-passing runs average .319 despite three zeros. | Those quantities cannot both be correct as printed. Do not quote .319 as a dependable conditional mean. |
| A failed expected-author gate proves the document was “untouched” or “empty.” | It proves only the recorded gate condition failed for the artifact the judge inspected. F2 explicitly says the inspected file path is missing. Wrong attribution, wrong destination, no artifact, and no substantive change are distinct possibilities. The source itself contains hundreds of existing revisions in the examples. |
| Reading `SKILL.md` “does not help,” based on readers’ and nonreaders’ means. | This is observational and confounded by task difficulty and agents seeking help after trouble. The 58-reader/12-nonreader breakdown also leaves five of 75 unspecified. It is not a causal test of discoverability. |
| Missing a pending-insertion rule explains a fixed count of “unblocked” runs. | Runs with a refusal identify exposure, not the number an intervention would rescue. Earlier discovery might help, but a legally appropriate revision decision, adequate remaining budget, and correct final content are still necessary. |
| F2’s refusal is about “another author’s” insertion. | The quoted example also includes a pending insertion by the agent’s own expected author. The relevant boundary is the tracked container and supported edit semantics, not simply author inequality. |
| Zero confirmed false-failure cases justify leaving the error channel “exactly as it is.” | Non-observation does not validate every error/recovery path. Bounded target context, structured repair choices, and committed-state reporting remain valuable. |
| R17’s incomplete 93-second trace proves a whole run was spent reading. | The visible trace is right-censored. It establishes no edit in the observed segment, not why or whether the full task terminated. Likewise, terminal tool/thinking events are not proof of budget exhaustion without termination metadata. |
| More operations, revisions, comments, or validation calls mean a more complete redline. | These are activity or artifact measures. The reports themselves show many operations and weak substantive grades. Counts must also distinguish plan operations, logical revisions, XML entries, baseline comments, new comments, and threads. |

R17 appropriately states that its telemetry groups have disjoint model support and no reliable task/session join. Its 12.75× prompt-length association is not a Jubarte effect. Its description of one run as “thorough” is an informal description of activity, not a validated measure of legal quality. The two graded artifacts have different denominators and task contexts; comparing their percentages is not an intervention estimate.

The reports’ literature gloss should also be checked rather than inherited: their roughly four-in-ten undetected-corruption observation belongs to the Loud/Quiet study’s quiet-failure detection figure, not the cited ToolUniverse audit. None of the supplied papers demonstrates that this particular template format, no-effect rule, or author gate improves contract-redlining success.

## 2. Revisions needed to the previous proposal

### Retain these decisions

1. **One canonical `edit template` route.** It belongs next to `edit apply`; `create --template` remains new-document creation from a reusable DOCX. Additional root aliases and a separate “agent” grammar increase discovery work.
2. **Remove `run_operation`.** An edited desired value should not be silently ignored because a second field was not toggled. Retain explicit operations for structural deletion, insertion, review, and formatting.
3. **Preflight inside every apply.** A separate dry-run invocation is optional. Do not require a planning document or several preparatory CLI calls for a simple, authorized replacement.
4. **Source binding, exact literal matching, explicit occurrence, and transactional publication.** The p15/p20 incident is evidence for preserving these protections, not for silently repairing a wrong target.
5. **Report actual effects and artifact identity.** “Plan prepared,” “candidate verified,” “artifact committed,” and “user’s requirements satisfied” are different assertions.

### Change the implementation order and the small schema

The previous four-operation core omits the very operation combination the new traces require. Main already has `resolve_revisions`, accepts/rejects selected logical changes before editing, maps original paragraph IDs through settlement, and supports comment actions in the same plan. `tests/edit_opaque_refusals.rs` even demonstrates that accepting the named r7 makes the intended replacement possible.

It is misleading to classify all revision correspondence as a later, absent capability. New paired-ID serialization and template-baseline semantics still require adapter work and tests, but the underlying source mapping is an asset to reuse. The initial production plan adapter should retain supported selective revision decisions and essential comment operations. A simplified tutorial schema can show a small subset; it must not become the only parser users can reach.

Keep separate `review` commands for standalone review tasks. Also permit an atomic plan to say, in effect, “resolve these explicitly selected revisions; preserve the rest; make these body edits; add/reply to these comments.” Do not make the caller create a new intermediate DOCX and rebind every paragraph solely to cross a CLI taxonomy boundary.

### Fix the template’s baseline ambiguity

The previous template compares desired text with a working baseline derived after the declared revision policy. If an author later changes the selected revision decisions, that baseline may change. Naively comparing old scaffolds with the new working text can turn an untouched row into an accidental request to restore rejected text.

A practical initial guard is a generated per-row `before_sha256` over the exact logical baseline text, alongside the source-byte digest and target. This adds no duplicated editable paragraph text. Apply verifies the expected working text before executing a changed row; a mismatch reports `WORKING_BASE_CHANGED`, the affected target, and bounded current context. It never silently rebases desired text. The codec must be versioned: Unicode characters, tabs, soft breaks, and excluded structural marks must mean the same thing at generation and application.

This guard has a usability cost that must be designed explicitly. If every row in an all-document template is checked after revision settlement, a deliberate rejection may invalidate untouched scaffolds or remove their target paragraphs. Either classify unchanged rows against a verifiable generation baseline before settlement, or require refreshing/removing only affected rows. The latter is simpler for an initial implementation, but the diagnostic must identify those rows and preserve the author’s actual desired changes. Do not tell an agent to regenerate the whole plan and lose its work.

Selected revision-only transactions are real mutations. They must not be refused merely because the body-operation list is empty. An unchanged **template scaffold** needs protection against being presented as implemented edits; that rule must not redefine all valid no-op review/state-setting operations as errors. The no-effect check should also count structural and review effects, not just differing text or a nonempty `operations` array.

### Do not make full-document templates the compulsory path

A named, complete template file is useful. It need not be printed back to the agent, and it should not be the recommended first step for every edit. Full-document desired-state editing asks the model to preserve substantial unchanged text and can encourage extra cosmetic rewriting. The reports’ excessive planning and output persistence make that risk more relevant, not less.

Use a short inline action when the target and change are already known. Use a sparse typed plan for a known batch, particularly one containing review decisions. Use a selected template when the agent wants to author desired clause text without inventing a schema. Continue offering an explicit full-scope export, with honest coverage. These are adapters to one transaction model, not separate semantics or a requirement to progress through all three.

Do not mark an entire paragraph uneditable merely because it contains a pending insertion or hyperlink. Current `resolve_rewrite` computes changed spans and preserves unchanged opaque spans; `tests/edit_opaque_refusals.rs` tests this behavior. Eligibility is conditional on the edited range. The template can expose the restriction without discarding valid changes elsewhere in the paragraph.

## 3. Responsibility: CLI, harness, and model

| Observed problem | CLI/engine responsibility | Harness responsibility | Model/task responsibility |
|---|---|---|---|
| Missing operation fields and repeated schema guesses | Actual operation schema, examples, constraints, consistent errors; do not silently drop unknown fields. | Provide compact discovery access; preserve the full result outside a bounded display. | Choose the operation corresponding to the intended edit; inspect a relevant schema when needed. |
| Pending insertion blocks a late plan | Surface range-specific restrictions; support explicit selected settlement and report its consequences. | Budget/measure time to useful execution and recovery without forcing premature writes. | Decide whether accepting/rejecting that revision is appropriate; a comment is not automatically equivalent to the requested body change. |
| Bookmark/range regression after dry run | Preserve ranges; check actual serialized candidates before publishing. | Record diagnostic and artifact identities. | Respond to supported corrective choices; do not improvise OOXML in ordinary workflows. |
| Exit status lost through `tail`, combined stderr, or shell status overwrite | Stable process status, parseable result, clean stdout/stderr; quiet preserves explicit data. | Capture subprocess return code directly and expose stdout/stderr separately. | Avoid treating a pipeline’s last-command status as Jubarte’s status. |
| Wrong output path or an uncertain write | Explicit artifact path, digest, role, committed state. | Grade the exact committed path/digest associated with the run. | Report the returned artifact, not an assumed default path. |
| Expected author missing | Allow explicit attribution; validate a declared attribution policy. | Inject the task’s known author or pass a job policy; verify only newly authored effects against it. | Use the specified author; do not impersonate a person inferred from unrelated configuration. |
| Required clause absent despite many edits | Verify declared deterministic effects and report their limits. | Independent requirement rubric and artifact inspection. | Select and implement the requested substantive changes. |
| Long reasoning/no final delivery | Cheap discovery, targeted output, direct actions, recoverable errors. | Termination reason, budgets, timeout/cancellation behavior, artifact collection; do not inject ambiguous “no visible output” messages. | Prioritize required changes and reserve time to apply/check/report; disclose genuine blockers. |

“The CLI cannot solve reasoning” is true but incomplete. It can reduce avoidable interface work and make wrong completion claims harder to support. Conversely, a CLI cannot infer an unexpressed legal requirement, repair a shell harness that discards its result, or make a chosen revision decision appropriate by labeling it a recovery step.

## 4. Concrete implementation sequence

### Work package A: expose the real request model and restrictions

Start with `EditPlan::from_json`, `OperationKind`, the flag adapter, and existing dispatcher support. Generate the schema and minimal examples from an authoritative normalized request model, including cross-field constraints the current parser enforces manually. A list of kind names or a serde-shaped schema lacking those constraints is insufficient.

Expose focused discovery through the already proposed `capabilities --operation`, `--schema`, and document-specific capability query. Do not introduce a new `agent-info` command that returns the whole schema, a full document inventory, and a mandatory five-step workflow in one giant response. The bootstrap result should fit a small budget and point to relevant detail.

Reuse projection segments and existing `check_range`, `check_insert_position`, `opaque_structure`, and tracked-emitter constraints for read-side diagnostics. Return, for a selected paragraph/range, facts such as “this span lies in r7; replacing it under keep requires a revision decision” and “this other span is direct editable text.” Do not encode a paragraph-wide `editable:false` where only one range is conditional. Unknown/unsupported coverage must remain explicit.

Add the missing simple flag adapters for occurrence and source digest to the versioned action grammar; they already have plan-level mechanisms. Use one spelling consistently (`--occurrence`, `--expect-sha256`) rather than adding `--nth` and `--digest` as new competing public terms. Retain legacy spellings only where actually shipped.

**Acceptance:** a user can obtain the exact fields for replacement and comment reply without reading the full help dump; a selected range exposes the same pending-revision boundary that apply later enforces; duplicated literals require an explicit occurrence.

### Work package B: preserve atomic review-plus-edit support

Adapt the existing `resolve_selected → settle → Origin::build → resolve → tracked::check` path into the new request model. Explicit accept/reject selections resolve against the original source’s logical revision IDs. The remaining revisions follow `keep` unless another policy is deliberately declared. No recommendation should default to accepting every revision to free an anchor.

Report selected revisions separately from changes removed as dependencies. The current `resolve_selected` comments acknowledge that accepting one change can take another with it; the production receipt must not describe every disappearance as an independently rejected change. Treat move halves together and reject conflicting accept/reject selections. Keep unknown IDs and source-stale errors distinct.

Expose comment additions/replies/resolution through the same compiled transaction where supported. Retain the engine’s refusal when selected revisions remove the anchor of a referenced comment. If the only supported sequence requires handling the comment first, report that exact dependency; do not silently delete a thread or turn a requested reply into a new unanchored comment.

For body edits, preserve minimal span resolution. A full paragraph rewrite should not touch an unchanged tracked insertion. Exact replacement and desired-state rewrite remain complementary entry points. Do not trade the existing boundary checks for approximate anchor repair.

**Acceptance:** one plan explicitly resolves a pending insertion, edits its now-direct text, changes a separate ordinary clause, and adds/replies to a comment; all effects publish together or none do. Original IDs still locate the intended paragraphs after settlement, including an earlier deleted/merged paragraph and table movement. A review-only plan works and produces a truthful receipt.

### Work package C: prepare once, validate actual candidates, publish once

At the inspected main snapshot, `run_edit` calls `preview_plan` for dry run, and `preview_plan` calls `apply_plan` in memory to obtain counts. `apply_plan` resolves, mutates, packages, and creates the tracked result. It does not invoke the general `validate_with` pipeline as a publication gate. Its successful result is not a general final-artifact validity proof.

Refactor toward one internal preparation result containing serialized candidate bytes, resolved effects, source/output correspondence, and diagnostics. Dry run discards those bytes and reports “prepared”; normal apply publishes them after destination checks. This avoids duplicate resolution now performed by `preview_plan`, reduces divergence, and does not add public workflow stages.

Before publication:

1. Reopen each requested DOCX candidate and run the required package/relationship/revision/range invariants.
2. Verify operation postconditions on the reopened result using correspondence, not old pN indices: exact replacement/rewrite text; deliberate paragraph insertion/deletion; selected revision state; comment/thread effect where requested.
3. Compare protected structure and meaningful source-derived invariants outside the intended edit footprint. Exact text success alone cannot detect a misplaced bookmark or vanished comment.
4. Evaluate the selected publication policy, then commit the artifact or new bundle once. A refused candidate remains unpublished; a reporting failure after commit must disclose committed state.

Do not simply wire the current `validate::fails` into all writes and call it done. At this snapshot `fails` treats any `word` or `edits` finding as failing, even if labeled warning. Existing documents may carry nonfatal findings. Define a narrow mandatory set for input admission and output correctness, block newly introduced or worsened designated preservation regressions, and report baseline findings. Optional quality/layout policies use explicit thresholds. Matching only aggregate counts is inadequate: one disappeared baseline warning plus one new violation can leave the count unchanged. Compare code, source-correspondent entity/range, and relevant details.

For the reported bookmark case, repair the insertion/range placement logic and add a preservation invariant for the bookmark endpoints. A generic warning suppression, blanket bookmark removal, or automatically moving the operation from “after p33” to “before p34” is not the fix. Those positions need not be equivalent in every table, section, revision, or bookmark context.

**Acceptance:** the actual candidate that would be written fails before commit when the reproduced bookmark boundary regresses; an unrelated pre-existing nonfatal finding is reported without arbitrary blockage; dry run and apply agree on the same candidate’s substantive checks. Publication tests cover output collision and reporting failure, not only engine return values.

### Work package D: implement the template as an adapter, with a small first surface

Generate the same plan model used by work packages A–C. The first template may expose only supported text rows and read-only context, but the plan parser must still permit the production review/body/comment operations. Keep the original source digest, explicit policy, canonical target, generated baseline-text hash, one desired text, and declared scope/coverage. Do not add an editable copy of every property merely because it exists in the document graph.

Make changed text sufficient to request the rewrite. Removing a row leaves that source object unchanged; an empty paragraph text is not structural deletion. Remove unchanged rows from overlap checks and output postconditions, subject to the explicit baseline-classification rule discussed above. Never let an old scaffold restore text after an explicit replacement or revision decision.

Return a compact receipt for a named template, with coverage and a few actionable restrictions. Default tutorials should select the relevant clauses or show a sparse batch. A complete all-stories file must inventory unreadable/uneditable regions honestly; it cannot claim a complete editable representation of every Word construct. A schema cannot turn unsupported textbox, drawing, field, or comment-story mutation into a supported operation.

Freeze attribution/date and revision policies in the plan. Keep policies editable as request fields with validation; do not silently override them from flags when applying. If a changed policy invalidates an authored baseline, expose the affected rows rather than treating it as a reason to overwrite all desired text. Source paths are hints; matching source bytes may be supplied under another path unless a separately stated file-identity rule requires otherwise.

**Acceptance:** generate→change desired text→apply produces the exact change; an untouched template cannot be reported as a changed artifact; legitimate review-only effects remain possible; rejection cannot accidentally reinsert rejected text; changed and unchanged rows coexist with explicit operations; restricted spans are preserved or diagnosed without making unrelated spans unavailable.

### Work package E: make receipts useful without claiming task completion

The receipt should identify source, normalized request, candidate/committed artifact, exact path, digest, phase, and effect counts. It should distinguish requested/changed/unchanged/refused effects, new revisions, selected existing revisions resolved, dependent effects, and comment changes. A generic `ok:true` or checksum alone is not enough.

Supply bounded semantic change records and links/arguments to targeted reads. A new root `stats` command is unnecessary if `review changes --json`, `review comments --json`, and read summary already expose native counts. Define whether counts include baseline content and whether a move is one logical change or multiple XML containers.

Optional deterministic expectations may verify a declared clause’s text, absence of a removed range, expected occurrence count, or required attribution on new changes. They should be independently expressible from the edit operations. If generated merely by copying each operation’s requested result, they are postconditions, not independent evidence that the user’s requirements were covered. The CLI should say what was checked and leave unassessed natural-language requirements unassessed.

Do not make `--author` globally mandatory to pass a benchmark gate. If a task or job policy specifies the author, enforce that known value; otherwise preserve an explicit neutral default. Also do not force an extra change merely to satisfy a no-effect guard when the user’s desired state is already present. Return that state honestly under the appropriate explicit unchanged contract.

## 5. Recommendations to reject or narrow

- **Automatic anchor-to-paragraph correction:** return bounded candidate targets and exact context; require the next request to name the chosen target. Identical literal text can appear in multiple clauses with different legal meaning. Similarity 1.0 is not authority to retarget.
- **“Accept all to unblock”:** selected settlement is already available; automatic global acceptance destroys a review choice. Accepting/rejecting must follow the task’s intended treatment of that revision.
- **Two new checking commands:** internal candidate validation is the needed engineering. `--dry-run` already has an established role; another “simulate” verb increases planning pressure.
- **One plan-builder subprocess per operation as the primary path:** incremental recording exists and is useful to some humans, but 95 separate shell calls may cost more than a schema-guided batch. Optimize the batch path and retain recording as an optional adapter.
- **Blanket refusal of paragraphs containing revisions:** editability depends on the changed span. The engine already preserves unchanged opaque structures during rewrite.
- **“No findings” as completion:** replace it with the check’s actual scope, not more reassuring phrasing. An integrity pass cannot certify negotiation requirements.
- **Public `pid:` identity or stamping all paragraphs to make it available:** FU explicitly withdraws this. Internal origin tracking does not make `w14:paraId` a durable user address. Reuse the agreed address contract and re-read after published mutations.
- **Blindly implement FU Task 5’s `allow_detach`:** its suggested rewrite check—whether new text contains the old anchor string—is an acknowledged approximation and can match the wrong occurrence. Range correspondence, replacement anchoring, and thread preservation need defined semantics; a plan-wide bypass is too broad.
- **Prioritize paragraph moves because FU has a detailed task:** the operation was not shipped, and the reported dominant failures do not establish it as a first priority. A compare engine’s heuristic move detection is not a guarantee of a requested Word move representation.
- **Treat all omitted or unfinished work as a model failure:** retain that possibility, but missing stop reasons, artifact paths, and report inconsistencies prevent assigning the entire residual to the model. Conversely, renaming commands cannot be credited with fixing all non-delivery.

## 6. Focused regression and evaluation plan

### Deterministic regression suite first

Use existing fixture helpers and extend the existing edit tests. The important assertions are artifact effects and preserved structure, not snapshots that mirror the implementation.

| Case | Assertion that matters |
|---|---|
| Pending insertion plus normal text in one paragraph | Editing the normal span succeeds under keep; editing the insertion needs explicit supported settlement; other marks remain. |
| Selected settlement shifts paragraph positions | Original targets still bind correctly; removed/merged targets are explained; no adjacent paragraph is edited by accident. |
| Mixed review/body/comment transaction | Valid combination publishes once; failure leaves no primary artifact; dependent revision/comment effects are accounted for. |
| Repeated literal and wrong paragraph | Ambiguity/wrong target refuses with bounded context; explicit occurrence succeeds; suggestions do not execute themselves. |
| Bookmark insertion regression | Reopened serialized candidate preserves the intended endpoint/range; introduced violation blocks publication without stripping the bookmark. |
| Baseline warning vs introduced violation | Existing unrelated nonfatal finding is distinguishable from a new violation, even with equal total finding counts. |
| Template baseline and review decision change | Rejecting a revision cannot turn an unchanged scaffold into restoration; affected authored rows receive a recoverable baseline mismatch. |
| Untouched/empty/structural/review-only requests | No-op classification respects operation semantics; explicit paragraph deletion is not confused with empty text; review-only effects count. |
| Exact whitespace/soft break | Requested logical text is realized exactly or refused; normalized-away changes cannot be counted as implemented. |
| Artifact/result contract | Structured mode is parseable on errors and empty results; quiet preserves requested data; committed path/digest and post-commit reporting failures are accurate. |

Add a small compatibility set for recorded v1 plans, old default revision behavior, current inapplicable/unknown fields, and each supported runtime’s actual dispatcher. Do not require rendering or Word parity tests for every text-wrapper change; reserve those for the concrete structural/rendering claims being made.

### Then a paired workflow experiment

Start with a small held-out set spanning clean paragraphs, received redlines, comments, repeated clauses, large documents, and one known unsupported requirement. Compare the existing interface and the targeted new interface on the **same tasks, same input hashes, same fixed model/version, same task instructions, same budget, and same corrected subprocess/artifact harness**. Randomize order and repeat each pair. A pilot such as 12 tasks × 2 interfaces × 2 runs is useful for finding large failures, not for claiming a precise general effect. Expand only where uncertainty affects the release decision.

The corrected harness must be common to both interface arms. Otherwise a simultaneous status-capture fix, author injection, model change, and CLI redesign produce an uninterpretable comparison. If harness behavior itself is under test, give it a separate paired experiment. Testing several frontier models can follow the first fixed-model pilot; it is not necessary to begin with a large factorial study.

Predefine three distinct outcomes:

1. **Tool correctness:** the committed artifact realizes declared operations and explicit review decisions, with required preservation/integrity checks. Refusal of an unsupported request can be correct tool behavior.
2. **Task completion:** an independent assessment of the final artifact against the user’s requirements, including appropriate handling of blocked work. A valid plan or an expected-author revision is not a substitute. Score actual clause requirements, not activity volume.
3. **Claimed completion:** whether the agent’s final claims match the artifact and the verified scope. Measure false completion among incorrect/incomplete artifacts, and false failure among correctly realized outputs, with explicit denominators.

Record missing/wrong artifact, attribution mismatch, unfinished run, legitimate unsupported task, and substantive omission separately. Preserve the actual artifact path/digest examined by the grader. Do not discard unjudged runs from the delivery endpoint merely because their grade is absent; grade availability is itself an outcome requiring a reason.

Useful secondary measurements are time to first **required** effect, time spent in discovery, bytes returned/persisted, actual return codes, recoveries from specific refusals, total resource use, and time remaining at first blocked call. An early cosmetic edit is not useful progress, so “time to first edit” alone should not become the optimization target. Nor should higher revision counts or fewer refusals be unconditionally rewarded: fewer refusals can mean silent wrong edits.

Log model/provider/version, CLI commit and binary digest, task/input/prompt hashes, start/end and tool timestamps, structured argv, direct process return codes, separate stdout/stderr, explicit stop reason, requested output, committed artifact identity, and independent grade. Keep cumulative generation-token sums separate from unique context size and per-run totals. Mark genuinely right-censored traces as such.

For contract-content grades, have a small adjudicated sample checked against the document, with graders blinded to interface arm where practical. An LLM rubric can help, but neither it nor a “domain-calibrated detector” is automatically ground truth. Avoid computing confidence as though rubric items nested within the same task were independent runs.

## 7. Migration and release gates

Ship targeted discovery, boundary diagnostics, adapters to existing selected settlement, and internal candidate checks before expanding authoring formats. Do not let the work turn into a comprehensive rewrite of every command, Markdown projection, renderer, and document schema.

Preserve plan-v1 parsing and its historical semantics. Introduce source-bound desired-state rows, baseline hashes, paired targets, and any changed no-op semantics under a new version. Preserve existing plan-owned policy; do not silently reinterpret missing fields as the new defaults. Validate and document the normalizer from both old and new inputs into the shared engine representation.

An `edit <action>` grammar can collide with a legacy positional filename that happens to equal an action. It is not automatically a completely additive rename. Define the migration boundary or an exact legacy routing rule, and test real ambiguous operands rather than assuming extensions disambiguate everything. New aliases must not silently change output destinations, revision treatment, JSON/JSONL meaning, or attribution.

Do not require the new template path to wait for every proposed story/feature. Advertise its actual supported subset and carry unsupported regions as context/coverage. But do require the complete target workflow that justified it: received redline → explicit selected review decisions → body and comment edits → verified committed DOCX → truthful receipt. A simpler template that cannot complete that workflow is an attractive demonstration, not the needed product improvement.

**Release decision:** prefer demonstrably fewer schema guesses, less avoidable late recovery, correct committed effects, and fewer false completion claims. Do not declare success from fewer commands, more edits, more author-gate passes, or a prettier generated plan alone.
