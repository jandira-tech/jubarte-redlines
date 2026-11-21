# Adversarial reassessment: template, plans, and completion

## Recommendation

Retain **jubarte edit template**, but change what must land before it is useful. The earlier proposal solved disabled scaffolds, source binding, and ambiguous command placement. It deferred the operation that the new evidence repeatedly identifies as a late blocker: explicitly resolving selected existing revisions and then editing their resulting text in the same transaction. It also treated multi-edit paragraphs and comment authoring as peripheral when the supplied trajectories contain mixed batches of replacements, deletions, insertions, and comments.

The revised priority is: implement the effective working baseline and selective revision resolution; compose disjoint edits against that baseline; serialize, reopen, and validate the candidate before publication; then generate templates from the same extractor and compiler. A new template command over the old executor would expose the same late refusals under a friendlier name.

Remove the default NO_CHANGES_REQUESTED failure. An unchanged request can complete with an explicit unchanged result and a byte-identical primary copy. Offer an optional --require-change for callers that need that condition. Keep hard failures for an intended change that was lost, stale sources, unresolved anchors, unsupported effects, and failed requested assertions. Neither “some edit occurred” nor a valid DOCX proves that the requested negotiating position was implemented.

These are design recommendations, not measured improvements. The four supplied files establish useful mechanisms and counterexamples, but do not measure this redesign.

## Evidence and its limits

The sources for this reassessment are:

- [final_turn_001.md](../upload/final_turn_001.md), especially “Non-delivery mechanisms,” “False success through verification substitution,” and “What to change.”
- [final_turn_002.md](../upload/final_turn_002.md), especially the worked refusals, the completion examples, and the methodology caveats.
- [deep-research-report(17).md](../upload/deep-research-report(17).md), especially the three trajectories, the passed dry-run followed by a bookmark finding, and its proposed agent-facing workflow.
- [2026-10-09-agent-edit-follow-ups.md](../upload/2026-10-09-agent-edit-follow-ups.md), whose updated status block must take precedence over its older implementation checklist.

The first two are versions of one audit, not independent replications. They report 75 runs, 56 judged runs, and 16 failures of a gate looking for work under the expected author. That does not establish that all 16 documents were untouched: the audit itself says the file opened by the judge was not recorded. A wrong output artifact or wrong attribution can also produce that gate result. The reports also disagree about how many gate-failed runs never invoked edit: eight in version one and fourteen in version two, while version two includes contradictory run classifications. Their aggregate scores and timing summaries need reconciliation against the raw traces before being used quantitatively.

The strongest material for design is the concrete sequence of actions and tool results: a late edit meets a pending-insertion refusal; a correctly rejected ambiguous anchor names the missing occurrence; a successful structural validation is cited as proof of substantive completion; a passed dry-run is followed by a serialized bookmark problem. The two reports' unadjusted comparison of skill readers with nonreaders does not show that documentation cannot help. Likewise, the comparative report explicitly lacks a matched vanilla-DOCX control. Its observed timing is not a causal estimate of Jubarte's cost.

The follow-up memo says its later audited snapshot had source SHA binding and plan-level occurrence, but lacked the proposed --digest and --nth flags, the general comment-anchor guard, and a move operation. Its withdrawn pid: and --pids proposals must not re-enter this design. Its dates and commit are supplied snapshot claims; this reassessment does not independently certify the later repository state.

## Where our earlier proposal falls short

| Earlier choice | Concrete weakness exposed by the new material | Revision |
|---|---|---|
| Ship template with keep/refuse first; gate acceptance/rejection later | The reported dominant refusal requires selected revision settlement before editing. Separate save, reread, and regeneration add work at the exact late stage where runs stopped. | Selective resolution and original-to-working correspondence are dependencies of the useful template release. |
| One desired text field, recovered baseline from source SHA and IDs | Later changing revision decisions changes the working text even though the original source SHA is unchanged. An old scaffold can become an unintended rewrite. | Add generated before_sha256 to rewrite rows and validate it against the effective working paragraph before classifying a row. |
| Default untouched-plan refusal | Three reported false-completion runs made 31, 35, and 95 operations. A positive-effect guard accepts all three. It also makes normal unchanged applications exceptional. | Default status unchanged; optional --require-change; independent assertions where requested. |
| Disjoint same-paragraph replacements “can be supported” | Common contract revisions change several independent phrases in one paragraph. Requiring one operation per paragraph is unnecessary friction. | Disjoint span composition is a release requirement, with one final composed paragraph postcondition. |
| Comment authoring deferred outside the template core | The 21-operation and 95-operation trajectories mixed comments with body edits. Requiring another save/rebind cycle recreates the workflow tax. | The shared production schema must include the common comment/reply operations; generated paragraph templates need not scaffold comments as edits. |
| Full-document plan on default stdout | Supplied runs already suffered oversized tool results and follow-up paging. Changing Markdown overflow into JSON overflow does not remove that problem. | Require -o FILE or explicit -o -; make scoped examples first; report compactly when writing a file. |
| Read-only whole-paragraph exclusion | A paragraph containing a revision can still have an editable range outside it. “Paragraph unsupported” hides available actions. | Report readiness for the operation and affected range, including conditional blockers. |
| Stronger prose about validation | The actual bookmark case arose after serialization even though all textual operations passed preflight. A renamed check does not prevent it. | The executor must serialize and reopen the candidate, then run the specified structural/preservation checks before commit. |
| Fixture-only examples passed | The fixture excludes tables and revisions; it cannot establish the hard features that dominate the new evidence. | Label the examples as schema examples and add structural, mixed-operation, and revision fixtures to the real implementation gates. |

I independently tested one weakness of our old example validator without modifying it. The old schema accepts two disjoint replacements in fixture p1: replace the first “fee” with “charge,” and “payable” with “due.” The expected result is “The charge is due; the fee is due.” The fixture-only oracle rejects this as “Overlapping text edits,” because it refuses a second changed operation in a paragraph. This is a limitation of our design verification, not evidence of a production Jubarte defect.

## Canonical command and the shortest useful workflow

The command remains **edit template**. It prepares a request to edit an existing document, and that request is consumed by edit apply. create template suggests authoring or instantiating a reusable DOCX template; root template creates another task family without solving a distinct workflow. This placement is a semantic judgment, not an experimentally demonstrated advantage.

The canonical contract should be:

~~~text
jubarte edit template [OPTIONS] -o FILE SOURCE.docx
jubarte edit template [OPTIONS] -o - SOURCE.docx
jubarte edit template [OPTIONS] --dry-run SOURCE.docx
~~~

FILE is one strict versioned JSON plan. The first form prints a compact receipt, including source digest, selected scope, generated rows, conditional or unsupported regions, and the next apply invocation. The second emits only the raw plan on stdout; --json and --jsonl conflict with -o -. The file form may emit its receipt in the standard JSON or JSONL result protocol. A result stream is never silently accepted as a plan document. A dry-run writes no files and does not require a destination.

An entire selected scope can still be generated, with all stories as the default scope. It must be complete or report failure; the read presentation budget must never silently cut a saved plan. The normal help examples should begin with a few selected clauses rather than encouraging every caller to author a document-sized plan.

Add **read --for-edit** as a compact mode of the existing reader, not a competing top-level locate or agent-info family. It should share the template extractor and return exact logical text, paired IDs, full text hashes, available operations, and relevant revision blockers. It is useful before either an inline edit or a plan. An exact one-off replacement remains an acceptable complete workflow; plans and separate dry-runs must not become mandatory ceremony.

For selected settlement, the generator should accept repeatable --accept-change ID and --reject-change ID. Both compile to the same resolve_revisions field used by apply. Neither accepts whole groups merely because the agent encountered an error. A later --resolve FILE adapter can make long decision lists convenient, but is not a prerequisite for the initial generator.

The minimum normal batch is: read the relevant content and readiness; generate a source-bound plan with known settlement decisions; edit the desired text or append typed operations; apply once. Apply always performs full internal preflight. A separate --dry-run is optional for inspection and debugging. The output's changed evidence and requested expectations can be sufficient for verification; do not mandate a second scratch publication merely to find problems the executor could have found before the first commit.

## Revised plan model

Keep one strict versioned model behind inline commands, generated plans, and apply. Unknown fields fail, and versioned old-plan defaults retain their old meaning. A simplified schema must be described as a view of the production model rather than a separate incompatible dialect.

### Fields

| Field | Meaning |
|---|---|
| schema_version | The accepted version of the complete request model and logical text codec. |
| source.sha256 | SHA-256 of the original input bytes. File location is not identity. |
| policy.track | Whether new content changes require tracked representation. |
| policy.existing_revisions | Keep by default. The canonical selective path keeps unselected revisions. |
| policy.text_view | Final logical text after the explicitly selected revision decisions. This is a projection, not permission to accept all changes. |
| resolve_revisions | Explicit accept and reject selections using original paired revision references. Empty selections preserve every revision. |
| author and date | Recorded attribution for newly authored marks, not rewritten history. |
| scope | Original selected entity roots and permitted descendants, or all. |
| operations | Typed body/review requests against the effective working baseline. |
| optional context/coverage | Source-derived observations, never permission or an assertion of task completion. |

For the selective core, existing_revisions:keep is compatible with nonempty resolve_revisions. The inspected local implementation already runs resolve_selected before applying existing_revisions to what remains; retain this ordering. Thus refuse means that any unresolved remaining revision blocks execution, not that explicitly selected resolution is itself forbidden. Keep is the normal path. Global accept/reject, if retained, must explicitly apply to the remaining revisions after the selected decisions; the small new schema can instead omit those broad policies while the historical adapter retains them. Never leave the order implicit or interpret an explicit rejection as permission to accept that same revision.

Selected scope requires care. A paragraph selection must not accidentally authorize accepting an unrelated revision. Explicitly naming r7 does authorize that logical revision, including both halves of a move and reported dependent effects. The normalized scope must include the explicitly selected revision entity, even if its range extends beyond a paragraph selection. Do not silently widen a text selection merely to make a requested operation pass. A removed, split, or otherwise unmappable target must produce a specific refusal.

### Rewrite rows need a baseline precondition

A generated row should contain kind, target, before_sha256, and one editable text value. The hash is SHA-256 of UTF-8 exact logical working text, with no Unicode, whitespace, Markdown, or line-ending normalization. The versioned codec determines tabs and soft breaks. The tool generates the hash; the agent copies it unchanged. The hash adds a precondition without duplicating the paragraph.

Consider an original paragraph containing a pending replacement of “30 days” with “45 days.” A template generated before settlement observes final text “45 days.” If the caller later adds rejection, the effective baseline becomes “30 days.” Without a hash, the old unchanged scaffold now requests “45 days” and recreates the rejected edit. With before_sha256, application fails WORKING_BASE_CHANGED before treating the row as active or inactive. The refusal returns the effective current text, its hash, the relevant decisions, and a bounded way to refresh the affected row.

A whole-plan resolution fingerprint would also prevent this drift, but it needlessly invalidates rows whose text did not change when an unrelated revision was resolved. The paragraph hash is more precise. Source SHA, paired identity, actual structure, and runtime capabilities still need independent verification. Identical text in two different paragraphs has the same before hash; the hash cannot detect every mistaken choice of both IDs.

Exact replace already carries a source-text precondition in find, so the extra hash need not be mandatory for every replacement. All occurrences still resolve against the effective working base, not against intermediate mutations.

### Illustrative request using the existing plain-text fixture

This request is a proposed revised-model example, not an input accepted by the earlier illustrative schema. Its file digest and paragraph-text digest are real values for the existing synthetic fixture. Its comment field is a proposed typed attached-comment convenience.

~~~json
{
  "schema_version": 2,
  "source": {
    "sha256": "54ef59e60b1986dd06131bfab3d6647d2f70b095ef3c09d698b4e221855c383e"
  },
  "policy": {
    "track": "on",
    "existing_revisions": "keep",
    "text_view": "final"
  },
  "resolve_revisions": {
    "accept": [],
    "reject": []
  },
  "author": "Jubarte",
  "date": "2026-10-17T07:00:00Z",
  "scope": {"kind": "all"},
  "operations": [
    {
      "kind": "rewrite",
      "target": {
        "short_id": "p0",
        "long_id": "document.w:p.j:short.p0"
      },
      "before_sha256": "63275eebdca0d61b331ef6408059f5a0df710dfcec01c23efa89331bf4284120",
      "text": "Payment is due within 45 days."
    },
    {
      "kind": "replace",
      "target": {
        "short_id": "p1",
        "long_id": "document.w:p.j:short.p1"
      },
      "find": "fee",
      "occurrence": 1,
      "text": "charge"
    },
    {
      "kind": "replace",
      "target": {
        "short_id": "p1",
        "long_id": "document.w:p.j:short.p1"
      },
      "find": "payable",
      "text": "due",
      "comment": {
        "text": "Use the same payment term consistently."
      }
    }
  ]
}
~~~

In real documents all pairs must be copied from the tool. Revision selections contain the same two-field paired-reference shape; an operation JSON pointer is a request location, not an additional document identity.

## Composition, comments, and IDs

All original entity pairs resolve first. Apply selected revision decisions in a transient working document while retaining explicit original-to-working correspondence. Then resolve each active find and occurrence once against that effective base. Build the edits as source intervals, reject genuine incompatible overlaps, and construct one composed paragraph outcome. Never mutate the string and then reinterpret the next occurrence in the mutated string.

For example, two replacements in “x x” with occurrence 1 and occurrence 2 must produce the requested two outputs even when the first replacement changes length or introduces another “x.” Non-overlapping text spans in one paragraph are a required capability. Multiple insertions at the same edge use request array order. An unchanged rewrite scaffold is verified against before_sha256 and then removed from the active mutation set, so it cannot conflict with a later explicit replacement or restore text that operation changed.

A changed full-paragraph rewrite can initially be exclusive with other text edits in that paragraph. That is a useful, explainable boundary: use one rewrite expressing the full desired text, or several disjoint exact operations. It should still compose with compatible insertion beside the paragraph and supported comment operations.

Mixed review work needs the same compiler. Standalone comment and reply_comment operations should use the common target pair and text field, with exact source range selection for a new anchored comment. An attached comment on replace or insert_paragraph can address the resulting text directly without inventing another public result-ID family. The compiler already knows the resulting span. This convenience is more useful than a generic operation-result reference language for the initial redlining workflows.

Comment preservation is geometric and structural, not a test for whether a substring happens to survive elsewhere. A deletion can leave an anchor in tracked history while hiding it in the final projection; that is different from an orphaned package anchor. A whole-range replacement can have a defined transfer to its replacement span even if no old characters survive. If a selected revision removes the actual target comment or its unrepresentable anchor, a reply must refuse with that precise cause. The older follow-up checklist's general “original characters survive” rule is not sufficient.

Keep the established two-ID contract. Story pN inventory includes cell paragraphs; a cell-scoped selector can normalize to the canonical story paragraph. Walking the body and walking tables must not generate duplicate rewrite rows. Textbox psN remains scoped. A bare psN, pid, paraId-as-selector, or third at/ref alias must not appear. Do not scaffold runs as independently addressable operations where the public registry provides no paired run identity.

Source path relocation with identical bytes is valid. A changed source hash is stale even if the visible text happens to match. Successful output addresses are newly read from the output or a verified correspondence report; guessed numerical index shifts are never an interface.

## Readiness should shorten the path to a valid request

A readiness result must describe the actual operation and range. A paragraph can be readable and preservable, have replace available outside a pending insertion, and have rewrite conditional on a specific revision decision. It should not be represented by one misleading editable Boolean.

Useful states are available, conditional, unavailable, and unknown. Conditional records name the exact blocking revision pairs, the affected source ranges, the applicable operation, and the choices the caller can make. A choice list can include accepting or rejecting a revision and selecting an unaffected range; it must not select a negotiating position automatically. A refusal should not suggest --force because output replacement has no relationship to these content decisions.

Readiness is not a guarantee that every future desired text can be represented. It cannot know the semantics of an unwritten rewrite. Capabilities describe the implementation; a range probe reports the present structure; candidate preparation verifies the requested change. State these scopes separately.

The evidence supports exposing the pending-revision obstacle before the agent writes a large plan. It does not justify excluding every paragraph that contains a revision, nor does it establish that another mandatory inspection command helps. read --for-edit and template should use one extractor so the necessary information arrives in a call the workflow already needs.

## Unchanged output and real completion

Default unchanged success should have status unchanged, zero effective mutation counts, the source and output digests, and the exact artifact path. With an explicitly requested output, copy original bytes rather than reserializing a semantically untouched document. A dry-run emits the same predicted outcome but commits no file.

This is not permission to replay a source-bound plan on its own output. That is a different source and must still fail its SHA precondition. Nor does unchanged success bypass output collisions or permit overwriting an input. Replay receipts and output replacement remain separately defined.

Explicit selected revision resolution, a comment, a reply, or an empty structural paragraph insertion can be real requested work without a newly changed body-text string. A meaningful direct edit can also produce no new tracked marks. Therefore requiring a positive count of new revisions is not a universal success rule.

An intended rewrite that requests different exact text but loses the difference in normalization is a failed postcondition, not an unchanged success. An unmatched find must still refuse even if some desired text happens to occur elsewhere. --require-change can require at least one effective domain mutation before commit, but must not advertise that this proves a substantive contractual improvement.

The supplied audit's 31-, 35-, and 95-operation counterexamples make the distinction concrete: all would pass an “edits exist” gate. Structural validation and new-author detection measure useful things, but neither checks an indemnity carve-out or a required deletion. Replace completion-shaped prose with measured statements: candidate package checks passed; these exact effects committed; these specified assertions passed; this scope was not assessed.

Allow an optional external, source-bound expectations file at apply time. It can ask for exact text, absence or count in an explicit scope, a structural deletion, preservation of specified content, or required attribution. Evaluate it against the prepared candidate before commit and record its digest and results. Unsupported assertions must be reported, not counted as passing. Independent user or harness expectations provide better evidence than assertions automatically copied from the plan being tested.

Do not mandate a natural-language requirement ledger for every edit, and do not claim that machine text assertions decide legal or commercial correctness. A same-plan expected string still has value as a mechanical invariant; it simply is not independent evidence that the agent selected the right requirement. The default result should remain explicit when no task expectations were supplied.

The CLI can preserve compact receipts and exit codes. It cannot prevent a caller from discarding them through a shell pipeline. Canonical examples should invoke apply directly or capture its exit status before filtering. --quiet should suppress progress and incidental prose, not an explicitly requested machine receipt. A requested persistent report and exact output path reduce wrong-artifact ambiguity but do not prove which file an external judge used.

## Delivery sequence and evaluation

1. **Freeze executable contracts.** Generate current schemas, examples, operation support, and error definitions from the implemented dispatcher. Preserve PlanV1 and its defaults. A list of operation names is not a schema.
2. **Implement the effective base.** Resolve original paired IDs and explicit revision selections; retain source-to-working correspondence; preserve unselected revisions. Add the rewrite before_sha256 check and precise blocker information.
3. **Implement complete batch preparation.** Compose disjoint text edits and common review actions; verify anchoring and comment effects; serialize, reopen, and check the candidate. Every apply takes this path. Optional dry-run exposes the same preparation.
4. **Expose shared edit readiness.** Add the read --for-edit adapter and concise structured refusals, using the same logical text and range resolver. Return actual target context and current hashes within declared bounds.
5. **Ship template as the thin generator.** Explicit output, scoped first examples, full selected-scope file, one desired text, generated preconditions, conditional/unsupported inventory, no run_operation. It must compile through the same production model.
6. **Add external expectations and completion evidence.** An optional expectation artifact should not delay the mechanical foundation, but the product must not label mechanical validation as semantic completion in the meantime.

Release tests must cover the supplied failure mechanisms, not just a JSON schema round trip. Include pending inserted text accepted then edited; pending inserted text rejected with an obsolete scaffold hash; disjoint repeated anchors in one paragraph; combined edits and comments; an existing comment removed by resolution; aliases that otherwise duplicate cell paragraphs; source paths changed with identical bytes; stale source; hidden bookmark structure; exact whitespace; untouched template; an effective change lost by the engine; and an output reporting failure after commit.

For agents, run matched trials on held-out documents and requirements with the same model, budget, task, initial file, and attribution. Randomize interface assignment. Separate command naming, compact discovery, early readiness, template generation, selected settlement, and expectation checking in ablations where practical. Measure correct requested effects and preservation first; time to first valid request, recovery calls, bytes returned, and total time are supporting measures. An earlier typo edit must not count as evidence of useful early progress.

Compare unchanged success with an opt-in require-change gate rather than assuming either improves autonomy. Record failures where the tool blocks a valid action as well as failures it prevents. Inspect artifact paths and exact files scored. A template design should earn claims of reduced errors through these tests; this reassessment identifies plausible mechanisms and a testable implementation order.
