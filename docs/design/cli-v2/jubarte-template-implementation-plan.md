# Jubarte edit template: implementation plan

**Status:** proposed implementation plan. It describes work to build and evaluate, not behavior demonstrated by the current binary. The executor changes come first; the template command is a generator over those shared capabilities.

## 1. Outcome and scope

An agent or person should be able to obtain a source-bound editing request, change the desired text in a few relevant paragraphs, make explicit decisions about blocking existing revisions, and publish a verified document in one apply transaction. The interface must distinguish saving that request from changing the DOCX.

The initial complete workflow includes the eleven-kind text-and-review profile: rewrite, replace, delete, insert, insert_paragraph, delete_paragraph, comment, reply_comment, resolve_comment, edit_comment, and delete_comment. Selected revision acceptance/rejection is a separate explicit plan field. The workflow must support several non-overlapping text edits in one paragraph. An illustrative four-operation schema is useful for explanation, but must not become a separate production dialect that excludes normal mixed redlining batches.

This plan does not require arbitrary OOXML editing, a move-paragraph operation, a generalized operation-result reference language, a DOCX creation spec, or an editable Markdown importer. Those features have separate preservation and usability requirements.

### Design decisions

| Decision | Contract |
|---|---|
| Canonical command | jubarte edit template |
| Plan consumer | jubarte edit apply --plan |
| Output | Require -o FILE or explicit -o -; a dry-run needs neither |
| Default selection | Entire document, all stories, in the saved file; examples begin with small selections |
| Editable row | kind, paired target, before_sha256, one desired text value |
| Activation | No run_operation, enabled, or equivalent Boolean |
| Unchanged row | No text edit after the baseline precondition passes |
| Removed row | Requests no rewrite for that paragraph; explicit decisions and other operations still apply |
| Structural deletion | An explicit delete_paragraph operation |
| Existing revisions | Keep unselected revisions; selected accept/reject decisions are explicit |
| Revision choices at generation | Repeatable --accept-change ID and --reject-change ID |
| Readiness | Per operation and affected range, with exact known revision blockers |
| Apply preparation | Always resolve, prepare, serialize, reopen, and check before publication |
| No-effect apply | status ok with data.effect unchanged; optional --require-change requires a primary document change |
| Task expectations | Optional external --expect FILE on apply, separate from template generation |

## 2. CLI contract to implement

~~~text
Prepare a source-bound edit plan from the document's exact logical text.

Usage:
  jubarte edit template [OPTIONS] -o PLAN.json SOURCE.docx
  jubarte edit template [OPTIONS] -o - SOURCE.docx
  jubarte edit template [OPTIONS] --dry-run SOURCE.docx

Common options:
  -o, --output FILE       Write one versioned JSON plan; - explicitly writes stdout.
      --select IDS        Paragraphs or supported containers; canonicalized and deduplicated.
      --story ID          body, a printed story ID, or all. Default: all.
      --find TEXT         Literal search of original-source final logical text.
      --regex PATTERN     Explicit regex search; conflicts with --find.
      --ignore-case       Search only; editing remains exact.
      --changed           Select blocks intersecting original revisions/comments.
      --by NAME           Existing-mark author filter with --changed; repeatable.
      --context N         Read-only original-source context; never extra editable rows.
      --accept-change ID  Accept this original logical revision before editing; repeatable.
      --reject-change ID  Reject this original logical revision before editing; repeatable.
      --author NAME       Attribution for new revisions/comments. Default: Jubarte.
      --datetime VALUE    Record the plan's timestamp, using the common attribution option.
      --track MODE        on or off. Default: on.
      --existing-revisions POLICY
                          keep, accept, reject, or refuse for remaining revisions.
                          Default: keep; applied after the selected decisions.
      --dry-run           Report scope and readiness; write no files.
      --json              One result object for a file output or dry-run.
      --jsonl             Result records with a terminal result; not a plan encoding.

Change a row's text to request a rewrite. Keep or remove unchanged rows.
Removing a row requests no rewrite; explicit decisions/other operations still apply.
Use explicit operations for insertions, deletions, comments, and replies.
Source IDs and before_sha256 are generated preconditions, not editable content.
~~~

The datetime spelling should follow the shared CLI's chosen attribution option. The wire field remains date. This command must not introduce a second spelling that other authoring actions do not use.

First examples:

~~~sh
jubarte edit template --select p12,p20-p25 -o plan.json contract.docx
jubarte edit template --select p12 --accept-change r7 -o plan.json contract.docx
jubarte edit apply --plan plan.json -o review.docx contract.docx --json
~~~

An optional separate check is:

~~~sh
jubarte edit apply --plan plan.json --dry-run contract.docx --json
~~~

The first apply form already performs that preparation internally. A separate dry-run is an inspection choice, not a required step in every workflow.

### Output rules

- With -o FILE, save raw PlanV2 JSON to that file and print a compact receipt. Do not also print the full plan or document.
- With -o -, stdout contains only the raw plan. --json and --jsonl conflict with this form, because a result envelope and a plan document are different products.
- --json and --jsonl control the receipt for a file output or dry-run. The JSONL terminal result determines completeness. Neither result protocol is accepted as an edit plan.
- A dry-run writes no plan, document, report file, render, or persistent cache. It reports the scope and readiness without dumping a document-sized scaffold; --dry-run conflicts with the raw-stdout -o - form.
- --force replaces only an explicitly named existing output file. It does not replace the source, override a stale hash, resolve a revision, or bypass a structural check.

The saved file covers the complete selected scope within admission limits. Selection and presentation are different: a small stdout receipt does not mean the file is truncated. Failure to finish the selected traversal must fail generation rather than publish a seemingly complete partial plan.

A valid filter matching no source entities returns EMPTY_SELECTION with zero matches and no plan file. This does not redefine valid unchanged application as failure. A nonempty scope containing only read-only/opaque entities can produce a complete inventory with zero rewrite rows and an explicit zero-editable-row receipt. Ordinary reads retain normal empty-result success.

### Selection is frozen on the original source

In template and read --for-edit, --select, --find/--regex, and --changed/--by select entities in the original source S before any revision settlement. Text search uses S's final logical text; mark filters use S's original revision/comment inventory. Freeze that selection, then display exact working text from W for the selected entities. A rejection may therefore make the displayed W text differ from the original matched phrase; label the original match and its context as original.

--context adds read-only adjacent context from S. It never adds rewrite rows or direct editing scope. Read cursors bind the source digest, frozen selection, selected decisions, tracking mode, and remaining-revision policy. Changing any of those requires a new query rather than continuing an old cursor.

read --for-edit exposes --accept-change, --reject-change, --existing-revisions, and --track so its working projection and readiness use the same inputs as generation and apply. Its result cannot authorize broader changes than those inputs declare.

## 3. Shared request and observation model

PlanV2 is the public request model. Inline actions, templates, and apply compile to one internal representation. Generated JSON Schema, examples, argument help, and capability reports come from that model. PlanV1 remains readable through an explicit version adapter with its historical semantics.

The essential fields are source.sha256, policy, author, date, scope, resolve_revisions, and operations. Optional source_context contains immutable observations of the original source. Every document reference in JSON is exactly the established short_id/long_id pair. Operation JSON pointers can locate a request entry in an error; they are not document IDs.

A generated paragraph row is:

~~~json
{
  "kind": "rewrite",
  "target": {
    "short_id": "p0",
    "long_id": "document.w:p.j:short.p0"
  },
  "before_sha256": "63275eebdca0d61b331ef6408059f5a0df710dfcec01c23efa89331bf4284120",
  "text": "Payment is due within 30 days."
}
~~~

The example uses the existing synthetic fixture without native paragraph IDs. The hash is SHA-256 of the exact UTF-8 text shown. Real target pairs and hashes are copied from the generator or reader.

Changing text to “Payment is due within 45 days.” requests that exact new paragraph text. Leaving text equal to the verified baseline requests no text edit. An attached_comment remains an explicit comment request even when the text is unchanged. An empty string requests an empty paragraph; it does not remove the paragraph mark. Removing the row requests no rewrite; selected revision decisions and other explicit operations can still change or remove that paragraph.

### Original source and effective working text

Let S be the original input bytes. Its digest and ID registry remain authoritative throughout the transaction.

1. Resolve and validate all original paired IDs and explicit revision selections in S.
2. Apply the selected acceptance/rejection decisions transiently, retaining original-to-working correspondence.
3. Apply the declared policy to any remaining revisions, using the documented ordering.
4. Derive the final logical working projection W.
5. Resolve operations against W using their original source references and verified correspondence.
6. Prepare a provisional candidate and determine the actual primary bytes to publish. A truly unchanged primary uses the original bytes S.
7. Reopen those actual primary bytes, validate them, evaluate expectations, and derive output IDs/digests from them before publication.

Keep is the normal remaining-revision policy. Apply explicit selected decisions first, then apply keep, accept, reject, or refuse to the remaining revisions. Refuse rejects any remaining revision anywhere in the source, not merely in the selected editing footprint; unknown revision forms cannot establish revision-free status. An otherwise unchanged request may therefore refuse under that explicit policy. Global acceptance/rejection applies to remaining history even when operations is empty. A revision selected for both acceptance and rejection is a conflict. Both halves of a move form one logical decision; dependent effects are reported separately.

Accepting or rejecting an old revision is an explicit effect even when there are no new text edits. Do not require a dummy replacement to make a legitimate resolution-only request publish.

Scope controls direct content and review-operation targets separately from resolve_revisions. Naming a revision authorizes that whole logical change and its reported dependencies; it never authorizes unrelated body edits in paragraphs touched by that revision. A comment range ending at through must include every intervening paragraph in the permitted direct scope, not just its endpoints.

Initial rewrites require exact one-to-one original-to-working paragraph identity. Paragraphs merged or split by settlement become read-only unsupported rows until a separate range-mapping adapter is implemented and tested. Existing Origin/paragraph correspondence can identify movement or a shared destination, but is not proof of exact source-range provenance. Other operations must not infer that missing provenance from numeric IDs.

### before_sha256 prevents baseline drift

For every supplied rewrite row, verify before_sha256 against the exact logical working paragraph text before deciding whether the row is unchanged. A mismatch is WORKING_BASE_CHANGED; return the current text/hash and the relevant revision decisions in bounded recovery information. A removed or unmappable paragraph is a separate target-correspondence refusal.

This catches a serious scaffold hazard. If a pending revision changes 30 to 45, a generated final-view row initially contains 45. Adding a rejection later makes W contain 30. Without the hash, the old scaffold would silently request 45 again. The hash forces the affected row to be refreshed.

The precondition is per paragraph. Resolving an unrelated revision should not require regenerating all rows when their working text is unchanged. The tool calculates these hashes; examples must not ask an agent to invent or manually calculate them.

The hash is not the entire preservation check. The original source digest, identity pair, actual structure, operation support, and output checks still apply. Identical text in two paragraphs yields the same text hash, so it cannot prove that the caller chose the intended clause.

### Logical text and context

The versioned text codec must preserve exact Unicode and whitespace. Define tabs, soft breaks, and paragraph boundaries explicitly. A newline within the supported paragraph text is a soft break; structural paragraph insertion is a typed operation. Do not put CriticMarkup delimiters, field placeholders, image markers, or comment balloons into editable plain text as if they were literal source characters.

A paragraph with pending revisions is not automatically excluded. If it has the required one-to-one correspondence, its logical text is representable, and supported edits outside blocked ranges are possible, it may have a rewrite row with conditional readiness in the generation result. Once desired text changes, test its actual diff against those ranges. A paragraph with no faithful supported rewrite representation belongs in read-only context with reasons.

Read-only inventory records must distinguish represented text, unsupported constructs, and opaque regions. Objects without public IDs are inventoried beneath a real paired ancestor; do not invent IDs for runs or bare boxes. Existing comments are context unless explicitly addressed by a typed review operation.

source_context records immutable original-S observations, using S's final logical projection for paragraph text and original comment bodies for comment observations. Changing selected decisions does not invalidate those original observations. Working text, before_sha256, and readiness are separately derived from W and recomputed by the engine. Readiness belongs in generation/read results; client-supplied plan “editable” or readiness Booleans are never authority. Editing supplied source_context text must refuse rather than silently ignore the change. Removing an observation does not request a content mutation.

### Attached and standalone comments

The initial profile includes optional attached_comment:{"text":"..."} on replace, rewrite, insert, and insert_paragraph. Attribution comes from the plan. The anchor is respectively the nonempty resulting replacement, the complete nonempty resulting paragraph, the inserted text, or the new paragraph. An empty resulting anchor with an attachment refuses; the engine does not choose a nearby point. Standalone comment operations resolve their ranges against W before content mutation and require supported range correspondence into the result.

An unchanged rewrite or a replace whose find equals text may still request an attached comment. Discard only the unchanged text component; retain and verify the comment effect. Conversely, removing the entire operation also removes its attached-comment request.

Review conflicts are explicit. delete_comment conflicts with every mutation, reply, or state operation on its affected comment/thread. Two edit_comment operations on the same target conflict. At most one resolve_comment state operation is permitted per thread. Replies can coexist with that one explicit state operation, which is applied after the replies; without it, preserve the original surviving thread state. Reply array order defines sibling order only and does not create an implicit reply-to-previous-result chain.

## 4. Executor stages and release gates

### Stage A — Freeze the model and compatibility boundary

**Work**

- Define strict PlanV2 parsing, required source binding, paired references, the logical text codec, rewrite preconditions, selected revision choices, and the common mixed operation set.
- Keep existing PlanV1 parsing and defaults in a versioned adapter.
- Generate schema, focused examples, and capabilities from the actual dispatch model.
- Distinguish syntax validation from source resolution and from full candidate preparation in result fields.

**Likely integration points**

The inspected source already has EditPlan, ResolveRevisions, resolve_selected, settle, Transaction, preview_plan, and apply_plan in src/edit.rs. Use those as integration points rather than adding a parallel edit engine. CLI parsing and adapters currently live in src/cli.rs and src/bin/jubarte.rs. Logical text and addressing have existing reader components under src/markdown/from_docx.

These are code locations to investigate and change, not a claim that their present behavior satisfies PlanV2. Current main's preview_plan already calls apply_plan. The gap is the shared final-candidate postchecks, not an assumption that preview executes only textual validation.

**Gate**

All shipped examples validate through the exact production schema and enter the corresponding dispatcher. Unknown fields, incompatible flags, unsupported versions, and third ID representations fail predictably. Legacy conformance fixtures still exercise their old contract.

### Stage B — Establish the effective baseline and precise readiness

**Work**

- Resolve original pairs once and verify that both IDs identify the same entity.
- Freeze direct selection and original-match context on S before settlement; keep explicit revision authorizations separate.
- Validate selected accept/reject choices against the original revision inventory before modifying a transient working copy.
- Retain original-to-working paragraph and comment correspondence through settlement, requiring one-to-one paragraph identity for initial rewrites.
- Calculate exact logical text and before_sha256 from W.
- Add readiness records for operation/range combinations: available, conditional, unavailable, or unknown.
- For conditional cases, include the exact known blocking revision pairs and their affected ranges. Describe choices without choosing acceptance or rejection.
- Share this extractor between read --for-edit, template, inline actions, and apply.

**Gate**

A selected existing insertion can be accepted and edited in one transaction. Rejecting it changes the baseline, and an old rewrite hash refuses before any output. Under keep, unselected revisions and their attribution remain intact. Under refuse, a remaining revision outside the selected footprint still blocks. Editable ranges outside a pending insertion remain available. Merged/split rewrite targets are read-only unsupported until the separate adapter passes its tests; shifted pN indices or Origin alone cannot substitute for range provenance.

Do not claim that readiness predicts an unwritten rewrite. It reports present structure and support. Full candidate preparation decides whether the actual desired change can execute.

### Stage C — Compose complete mixed requests

**Work**

- Resolve all literal anchors and occurrence values against W, before mutation.
- Count every exact literal start position, including overlaps, for uniqueness and occurrence. The anchor `aa` in `aaa` has two candidates; an unqualified edit is ambiguous. A supported all-match adapter refuses overlapping candidate ranges instead of greedily selecting a subset. Keep legacy matching semantics versioned.
- Compose disjoint edits to one paragraph as intervals over the same baseline.
- Verify one final paragraph result reflecting all composed operations.
- Remove only verified unchanged text components before overlap checks and final text postconditions; retain explicit attached comments.
- Give paragraph insertions at the same edge and text insertions at the same source position deterministic array order. Other ambiguous boundary ownership refuses.
- Support common body edits plus new comments/replies in one request.
- Implement attached_comment on replace/rewrite/insert/insert_paragraph with the exact nonempty result-anchor rules, without another public result-ID family.
- Preserve comment anchors through defined range correspondence; report removals or unsupported relationships accurately.
- Enforce the specified comment-edit, deletion, reply, and thread-state conflict rules.

**Gate**

Two disjoint replacements in the same paragraph succeed, including repeated anchors where each occurrence is explicit. The first replacement cannot change which occurrence the second uses. True overlap or unsupported boundary ownership refuses. An unchanged scaffold plus an explicit replacement does not restore old text. An unchanged text operation with an attached comment still creates that comment. All four attachment kinds have tested nonempty result anchors, and empty anchors refuse. A comment target removed by selected resolution reports that cause. Conflicting review actions fail before any publication.

In the initial profile, a changed full rewrite conflicts with other active text edits within the same paragraph. The caller can express one final rewrite or separate disjoint edits. Compatible comment effects and paragraph insertions beside the target may still coexist. This is not a blanket one-operation-per-paragraph rule.

### Stage D — Prepare and validate before publishing

**Work**

- Make every apply use one candidate-preparation pipeline.
- Serialize a provisional candidate, classify the primary transaction's effective changes, and select the actual primary bytes. Use S verbatim when the primary is truly unchanged.
- Reopen and check those actual primary bytes for structural/package validity, exact text and structural postconditions, supported tracking/attribution, and preservation constraints.
- Define the blocking invariants explicitly. Report baseline, introduced, and unresolved findings separately; do not promote general quality warnings to publication failures. Refuse a pre-existing condition only when it prevents establishing a required invariant.
- Evaluate supplied initial-profile expectations against the final projection of those actual bytes: paragraph text_equals, text_contains, and text_absent only. Require exact one-to-one target correspondence; a missing target refuses even for text_absent.
- Validate optional derivative artifacts under their requested policies before committing the bundle.
- Derive actual primary effect counts, output IDs, and digests from the selected and validated primary bytes. Report derivative effects separately.
- Enforce --require-change against the primary transaction. Producing a render, clean derivative, plan, patch, or report cannot satisfy it.
- Return data.phase validated, data.effect not_applied, and predicted_effect changed or unchanged for --dry-run without writing files. Apply stages those actual validated primary bytes on the destination filesystem and rechecks source/output conditions at commit.

**Gate**

The bookmark-crossing scenario from the supplied trajectory is diagnosed by shared final-candidate checks before primary publication. A candidate that normalizes away a requested exact change fails POSTCONDITION_FAILED. Missing expected text cannot become a successful unchanged result. A dry-run and actual apply agree on request validity and predicted effects given identical source/options; publication still rechecks the source and destination. For unchanged output, all final checks, expectations, IDs, and digests refer to S itself, not to a discarded reserialization.

The envelope status is ok, refused, or error. data.effect is changed, unchanged, or not_applied and describes the primary document transaction. A saved plan has data.phase generated and data.effect not_applied even though its plan artifact is committed. A successful dry-run has data.phase validated, data.effect not_applied, and a separate predicted_effect. A published document has data.phase committed and its actual primary effect. Artifact committed flags remain independent, and a post-commit reporting failure identifies already committed artifacts and their effects.

### Stage E — Add read --for-edit and the template generator

**Work**

- Expose exact source hash, target pairs, logical text hashes, readiness, and focused operation contracts in read --for-edit.
- Include --track and --existing-revisions in read --for-edit, and bind continuation cursors to all working-basis inputs.
- Keep bounded read previews distinct from complete editable values. A truncated preview must not masquerade as a complete rewrite payload.
- Implement edit template as a call to the shared selector/extractor and PlanV2 serializer.
- Canonicalize container descendants and deduplicate IDs, especially cell paragraphs already present in the story inventory.
- Include immutable original source_context and conditional/read-only/opaque inventory with coverage facts; report recomputed working readiness separately.
- Implement explicit file/stdout output forms, collision checks, atomic plan publication, and compact receipts.
- Generate practical help and examples from the same supported schema.

**Gate**

The entire generate → change text → apply → inspect workflow uses the production compiler. A full selected-scope plan is complete, even when the receipt is small. Selection is frozen on original S; --context adds no editable rows. Any admission limit or incomplete traversal is explicit. No run_operation, --runs scaffold, delete scaffold, or plan-format JSONL option is advertised. Generator-only success reports data.phase generated and data.effect not_applied, even when the plan file committed.

### Stage F — Evaluate on held-out workflows

**Work**

Run matched, randomized trials using the same model, task, budget, source bytes, and required author. Include people if human usability is a release claim. Instrument output bytes, time to first valid request, recovery calls, authored operations, committed effects, and artifact paths.

**Gate**

Correct required edits, preservation, and correct artifact delivery are the primary outcomes. Lower command count, smaller output, or an earlier first edit is insufficient if the substantive request is omitted. Use a defined regression margin for harmful or unintended edits and require adequate trial coverage before asserting improvement.

## 5. Acceptance test matrix

| Test | Expected behavior |
|---|---|
| Generate unchanged plain paragraph | Exact text and correct before_sha256; no activation field |
| Apply untouched generated file under keep | status ok, data.phase committed, data.effect unchanged; exact original-byte primary copy; no new marks |
| Same apply with --require-change | NO_CHANGES_REQUESTED before publication because primary effect is zero |
| Unchanged primary with render/clean/report requested and --require-change | Derivative creation cannot satisfy the primary-change requirement; no publication |
| Unchanged primary chosen over a provisional reserialization | Final validation, expectations, output IDs, and digests use the original bytes that will actually publish |
| Delete a row from the file | Requests no rewrite; selected decisions and other explicit operations still apply |
| Set desired text to empty | Empty paragraph; paragraph mark remains |
| Explicit delete_paragraph | Structural deletion only where supported; affected anchors reported |
| Change source after generation | STALE_SOURCE; no output |
| Move source path, retain identical bytes | Valid source identity; normal input/output collision rules still apply |
| Change both target IDs to a different paragraph with different text | before_sha256 mismatch on rewrite |
| Mismatched short and long IDs | Refusal even if each separately identifies an existing entity |
| Table traversal plus body traversal | One canonical row per cell paragraph |
| Settlement merges or splits a selected paragraph | Initial rewrite row is read-only unsupported; Origin or a shared destination is insufficient |
| Bare psN or retired pid selector | Usage/schema error; no invented conversion |
| Accept pending insertion and edit its text | One transaction; selected old revision resolved, new edit tracked |
| Reject pending change with old generated rewrite text/hash | WORKING_BASE_CHANGED; never recreate the rejected text silently |
| Resolve unrelated revision | Unchanged hashes remain usable; no whole-plan regeneration requirement |
| Same revision selected on both sides | Revision conflict before preparation |
| Refuse policy with a remaining revision outside edit scope | Refuses even if operations would otherwise be unchanged |
| Revision spans beyond a text selection | Explicit revision effect and dependencies disclosed; no unrelated direct edit is authorized |
| Comment through range skips an intervening out-of-scope paragraph | Refuses; endpoint membership alone is insufficient |
| Rejection changes the text that originally matched --find | Frozen original selection remains; exact W text is shown and original match context is labeled |
| --context adds adjacent paragraphs | Read-only S observations only; no extra rewrite rows or direct editing scope |
| Changed decisions with unchanged source_context | Original observations remain valid; W text/hash/readiness are recomputed |
| Cursor reused with different decisions/tracking/policy | Refuses continuation or requires a new query |
| Replace outside a pending insertion | Succeeds when supported; whole paragraph is not automatically blocked |
| Replace inside unresolved pending insertion | Precise blocking revision/range; no automatic acceptance |
| Two disjoint replacements in one paragraph | One correct composed result |
| Two occurrences of the same anchor | Resolved against the same baseline, independent of new text length |
| True overlapping replacements | Atomic refusal with both request locations |
| Changed full rewrite plus another active text edit in its paragraph | Conflict; use one rewrite or disjoint exact operations |
| Unchanged rewrite plus explicit replacement | Replacement takes effect; scaffold does not restore old text |
| Unchanged rewrite with attached_comment | No body rewrite, but the requested nonempty paragraph comment is created |
| Replace find equals text with attached_comment | No body replacement, but the requested nonempty replacement comment is created |
| Multiple inserts after one anchor | Final order equals request order |
| Text insertions at the same source position | Final order equals array order; other ambiguous boundary ownership refuses |
| Text insertion strictly inside a replaced/deleted span | Refuses unless a separately tested combination is explicitly supported |
| Source anchor deleted by another incompatible operation | Clear conflict; no guessed insertion location |
| attached_comment on replace/rewrite/insert/insert_paragraph | Exact specified resulting nonempty anchor and plan attribution; no guessed output pN |
| attached_comment with empty resulting text | Refuses; no neighboring or deleted-text anchor is invented |
| Existing comment removed by revision settlement | Explicit affected-comment refusal for a later reply |
| delete_comment combined with another mutation/reply/state on affected comment/thread | Conflict before any publication |
| Two edit_comment operations for one target | Conflict even if their order would make a last writer possible |
| Multiple resolve_comment states for one thread | Conflict; no last-writer rule |
| Replies plus one explicit thread state | Sibling order follows array order; explicit state applies after replies |
| Replies without explicit thread state | Preserve the original surviving thread state |
| Tracked deletion retains comment in markup only | Report visibility/state accurately; do not conflate with an orphan |
| Unsupported object in otherwise selected document | Preserved context/inventory; no fabricated plain-text placeholder edit |
| Changed read-only context text | Clear refusal; never silently ignored |
| Exact spaces, NBSP, tabs, and soft breaks | Applied exactly or refused; no normalization-based success |
| Passed anchor resolution but invalid serialized bookmark | Rejected before primary publication |
| Dry-run with requested report/path | No files written; only result stream |
| Successful apply dry-run | status ok, data.phase validated, data.effect not_applied, separate predicted_effect |
| --json or --jsonl with -o - | Usage error, with recognized machine error mode as specified by shared parser |
| Plan file written successfully | Plan artifact committed; data.phase generated; data.effect not_applied; no document artifact |
| Reporting failure after document commit | Error includes the real committed artifact state and digest |
| Initial expectation text_equals/text_contains/text_absent fails | No primary publication; precise failed check and coverage |
| Expectation paragraph is missing, merged, or split | Refuses; missing target never passes text_absent |
| No expectations supplied | Ordinary operation allowed; no claim of task-level validation |

Tests must assert the resulting package, revision/comment relationships, exact text, and output artifacts where relevant. A test that merely checks that an operation name appears in a receipt does not verify that the requested effect occurred. The fixture-only text oracle may remain an explanatory test, but must not be the gate for revision handling, structural preservation, or multi-operation composition.

## 6. One end-to-end scenario

Use a synthetic fixture with a paragraph whose clean earlier wording says “Payment is due within 30 days.” An existing revision changes 30 to 45. Another unaffected sentence in the same paragraph says “Late interest is 2%.” Add a comment anchored to that sentence and preserve one unrelated revision elsewhere. The fixture supplies the actual source pairs; examples must not fabricate long revision IDs.

1. read --for-edit for the paragraph returns its exact final logical text, source hash, paired paragraph identity, and the pending revision pair. Readiness says that an edit overlapping 45 is conditional on a revision decision; an edit to 2% is available if its range is otherwise supported.
2. Generate the selected template with --accept-change for the pending revision. The generator simulates that choice and records the correct working text/hash, while unselected revisions remain kept.
3. Author two disjoint operations in that paragraph: 45 → 60 and 2% → 1%. Add attached_comment:{"text":"..."} to one replacement. Leave any verified unchanged rewrite text row in place or remove it; explicit attachments and other operations retain their own effects.
4. Apply to a new output file with the plan and, optionally, an independently supplied expectation file checking the final paragraph text using text_equals or text_contains. Preservation of the unrelated revision is a mandatory engine invariant under keep, not an initial expectation predicate.
5. The executor verifies S, frozen direct selection, separate revision authorization, exact one-to-one paragraph correspondence, before hashes, literal matches, overlap, tracking support, and comment ranges. It composes the candidate, selects the actual primary bytes, and reopens those bytes for final checks and expectations before deriving output IDs/digests and committing.
6. The receipt identifies the actual document path and digest, one selected revision acceptance, two new text effects, the new comment, preserved pre-existing marks, and the exact checks performed. It does not say that the entire contract or negotiation is complete.

Negative branch: start with a template generated without settlement, then add rejection of the 30 → 45 revision without refreshing the rewrite row. Application must fail WORKING_BASE_CHANGED before treating the old 45 text as an instruction. A recovery response supplies the new 30-day text/hash for the affected row and a focused regeneration command. It must not ask the caller to accept all revisions or add --force.

## 7. Tradeoffs and evaluation criteria

**One desired text plus a hash:** easier to author than duplicated before/after paragraphs, and protected against revision-policy drift. It still requires a faithful rewrite engine. Exact replace remains preferable for small edits and supplies its own literal precondition.

**Full file, compact receipt:** preserves complete selected coverage without forcing document-sized terminal output. Agents still need scoped discovery; saving a huge plan alone does not reduce the amount of content they must reason about.

**Normal unchanged success:** status ok with data.effect unchanged supports source-bound desired-state workflows. Optional --require-change serves jobs that require a nonempty primary document effect. Explicit refuse can still block because of remaining revisions anywhere in the source. Neither policy proves semantic task completion.

**Early readiness:** exposes the supplied corpus's late structural obstacles in an existing read/generate call. It may still report conditional or unknown support; it must not overpromise arbitrary future edits.

**Selective settlement:** enables a common real workflow while preserving explicit review decisions. It does not authorize the tool to choose negotiating positions or accept old changes merely to reduce a refusal rate.

**One prepared candidate:** avoids publishing a scratch artifact solely to discover structural failures. It may add internal work to dry-runs, so measure latency and resource use. Do not optimize away the validation that prevents a known bad publication.

For agent A/B evaluation, separate these changes where feasible: compact discovery alone; readiness plus selected settlement; generated desired-state templates; and optional external expectations. Compare exact requested effects and preservation on held-out tasks. Include difficult mixed revisions, duplicate phrases, comments, tables, long paragraphs, and meaningful unchanged tasks. Record unsupported-but-honest refusals separately from execution defects and omitted requirements.

Do not infer success from operation counts. The supplied audit reports runs with 31, 35, and 95 applied operations that still omitted required content. A persuasive result is fewer omitted required changes and fewer unintended changes, with correct committed artifacts and no regression in preservation. Time and token reductions strengthen that result; they do not substitute for it.

## Evidence informing this plan

The two supplied failure-audit versions describe the same 75-run corpus. They provide concrete pending-revision refusals, late discovery, suppressed receipts, and mechanically valid but substantively incomplete outputs. Some aggregates and run classifications conflict, so this plan uses the mechanisms as design evidence rather than claiming a quantified improvement.

The supplied comparative report describes a successful dry-run report followed by a bookmark finding on the resulting artifact, and mixed 21-operation and 95-operation plans. Current main already calls apply_plan from preview_plan; the proposed remedy is shared postchecking of the actual primary bytes, not merely making preview execute edits. The follow-up memo's updated status distinguishes shipped source binding and plan occurrence from unshipped CLI flags and withdrawn identity proposals.

The earlier proposal and its fixture-only validator were also reviewed directly. The old schema accepted two disjoint replacements in one paragraph, but its oracle rejected them as overlapping. That illustrates why the release gates above test composition and actual DOCX behavior, beyond schema-valid examples.
