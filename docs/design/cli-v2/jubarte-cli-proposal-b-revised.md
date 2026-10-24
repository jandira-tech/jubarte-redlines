# Jubarte CLI proposal B — final reassessment

**17 October 2026 · Proposed contract, not implemented behavior**

This revision incorporates the four newly supplied reports and three independent critical reviews. It supersedes the earlier proposal in this package. The new CLI, schemas and guarantees require implementation and conformance tests; no repository code or pull request was changed.

The objective is to make a requested document change easier to express, prepare, publish and inspect. Structural validity, correctly attributed revisions and fulfilled user requirements are distinct claims.

## 1. Decisions after the evidence review

The reported problems include reconnaissance without delivery, late discovery of tracked-revision constraints, repeated schema repair, output navigation overhead, and completion claims unsupported by content. Several counts contradict one another. The evidence review separates report-level observations, independently checked source behavior and design hypotheses.

| Earlier recommendation or suggested remedy | Final decision |
|---|---|
| Disabled operation for every block | Remove `run_operation`. A desired-text change is a request. Unsupported content remains identified context. |
| Root `template` or `create template` | Use **`edit template`**, next to `edit apply`. `create --template` retains its reusable-DOCX meaning. |
| Text-only template before selective revision handling | Reverse the order. Selected revision decisions, body edits and review operations must work together first. |
| Defer source-to-working correspondence | Reuse and verify existing origin/settlement machinery. It is an essential dependency. |
| Refuse unchanged plans by default | Explicit unchanged success and exact source-byte copy when output is requested. `--require-change` is opt-in. |
| Separate plan-check and plan-simulate commands | One preparation pipeline for apply and dry run, including candidate serialization, reopening and required checks. |
| Mark every paragraph containing revisions uneditable | Operation- and range-specific readiness; safe edits outside pending revisions remain possible. |
| Compare scaffold text with whatever baseline exists at apply | Generated `before_sha256` guards each rewrite's working baseline. Refresh affected rows after changing it. |
| A source-only template solves missing requirements | Optional separately supplied expectations, precise receipts, and unassessed natural-language requirements. |
| Require a named author to pass the benchmark gate | Neutral `Jubarte` attribution unless supplied. Report actual attribution. |
| Full template on stdout by default | Require `-o FILE` or explicit `-o -`. Files are complete; normal receipts are compact. |
| Complete a large plan before any edit | Inline edits and coherent batches are both first-class, using the same execution guarantees. |

Desired-text templates, read budgets and expectations-assisted completion remain empirical usability hypotheses. No percentage improvement can be estimated from the supplied reports.

## 2. Canonical command tree

```text
jubarte
  create
  read
  edit
    replace | rewrite | insert | delete | format | fill | fields
    template | apply
  review
    changes | comments | accept | reject | comment | reply
    resolve | reopen | edit-comment | delete-comment
  compare
  render
  convert
  assemble
  check
  sanitize
  capabilities
  self-update
  help
```

There are twelve domain commands; help is discovery infrastructure. Humans and agents share one substantive interface. Terminal detection changes presentation, not data protocols or side effects.

### Root help

```text
Create, inspect, revise and publish Word documents with a local engine.

Usage: jubarte [OPTIONS] <COMMAND>

Examples:
  jubarte read contract.docx --find "payment" --context 1 --for-edit --json
  jubarte edit replace contract.docx --at p12 --find "30 days" --text "45 days" -o revised.docx
  jubarte edit template contract.docx --select p12,p20-p25 -o edits.json

Author and revise:
  create        Create DOCX from Markdown, a reusable DOCX or a supported spec.
  read          Read/search content, structure, exact targets and edit readiness.
  edit          Change content, prepare an edit template or apply a plan.
  review        Read and act on revisions and comments.

Compare and publish:
  compare       Compare content, package structure or rendered pages.
  render        Render PDF and page images.
  convert       Import legacy DOC or translate DOCX and Markdown.
  assemble      Join documents in order.

Verify and prepare:
  check         Check integrity, quality, tracking and explicit expectations.
  sanitize      Remove selected metadata or comments.

Discover:
  capabilities  Actual schemas, examples, constraints and runtime support.
  self-update   Explicitly check or install a release.
  help          Show command help.

Options:
  --json / --jsonl     Explicit machine results; mutually exclusive.
  -q, --quiet         Suppress status/progress, not requested data or errors.
  --color MODE        auto | always | never.
  --timeout SECONDS   Deadline with explicit publication state.
  -h, --help          Common options and examples.
  --help-all          Complete applicable contract.
  -V, --version       Version, build identity and runtime.

Document inputs remain unchanged. Document writes require an explicit destination.
```

Ordinary help leads with examples and the side-effect contract. It points to a relevant schema instead of dumping all operation definitions and rendering options.

## 3. Normal editing workflow

A known exact change can be made immediately. Investigation can return exact text and constraints in one response. A batch can be generated or authored without requiring a full-document planning pass.

```sh
jubarte read contract.docx --find "payment" --context 1 --for-edit --json

jubarte edit replace contract.docx \
  --at p12 --find "30 days" --text "45 days" \
  --expect-sha256 SOURCE_SHA256 \
  -o revised.docx --json
```

`SOURCE_SHA256` is a documentation placeholder; real calls copy the complete returned hash. An unbound inline call validates the current anchor but does not prove continuity with a previous observation.

For several changes:

```sh
jubarte edit template contract.docx --select p12,p20-p25 -o edits.json
# Edit desired text or add typed operations in edits.json.
jubarte edit apply contract.docx --plan edits.json \
  --out-dir review --emit document,patch,report --json
```

The same apply may be run with `--dry-run` first. Normal apply already performs required preparation. A dry run is not a mandatory additional permission step.

After mutation, use the receipt's output digest and output references, or read the output snapshot. Never guess how old pN indices shifted.

## 4. Read for editing and document-specific readiness

```text
Usage: jubarte read [OPTIONS] <FILE>

Default:
  Bounded addressable text with revisions/comments, hash and completeness.
  No layout pass.

Selection:
  --select IDS          Printed IDs or closed ranges.
  --story STORY         body | a printed story | all. Default all.
  --find TEXT           Literal search.
  --regex PATTERN       Explicit regex; conflicts with --find.
  --ignore-case         Search only.
  --context N           Adjacent context, distinct from matches.
  --changed             Blocks intersecting a revision or comment.
  --by NAME             Existing-mark author filter; repeatable.
  --limit N             Default 100 primary records.
  --max-chars N         Default 24000 Unicode characters.
  --cursor TOKEN        Continue the same hash-bound query.
  --all                 Disable presentation limits, not admission limits.

For editing:
  --for-edit            Exact logical working text, rewrite before_sha256,
                        paired JSON targets and range-specific readiness.
  --accept-change ID    Inspect a working view after selected acceptance.
  --reject-change ID    Inspect a working view after selected rejection.
                        Decision flags require --for-edit; repeatable.
                        They do not modify or resolve the source file.
  --existing-revisions keep|accept|reject|refuse
                        With --for-edit; default keep, matching new plans.
  --track on|off        With --for-edit; readiness for new edits, default on.

Views:
  --view text|summary|tables|styles|controls|stories|fields|fonts|
         package|runs|xml|numbering|elements|property-changes|
         style-definitions|render-input|textboxes|bookmark-chains|snapshot

Presentation:
  --revision-view markup|final|original
  --comments inline|ids|hide
  --dates
  --page-markers none|cached|layout
  --pages SPEC          Requires layout; physical pages, 1-based.
  --fields LIST         Machine-field selection; required provenance/IDs remain.
```

The 100-record/24,000-character limits are tuning values. A saved template is governed by selected scope and admission limits. Counts/cursors expose truncation; `--all` never expands a semantic selection.

`--for-edit` uses PlanV2's exact text codec and the final logical view of the explicit working source. An incompatible original/markup text view is a usage error. Review marks/comments appear separately, not as syntax to copy into an anchor. Search matches and context are labeled; context does not expand edit scope.

For `read --for-edit` and `edit template`, select original-source identities before settlement. Literal/regex selection searches the original source's final logical text; `--changed` and `--by` inspect its original revision/comment inventory. Freeze that membership, then display the selected targets' working text. An original match can disappear after an explicit rejection; label original-match evidence separately from working text. `--context` adds read-only neighbors in the original source, not new editable targets. Cursors bind source, selection, decisions, tracking and remaining-revision policy. Ordinary reads continue to search their declared display projection.

Each returned text value declares completeness. A truncated paragraph preview cannot be copied as a complete rewrite, even when its full-text hash is shown. Return a focused continuation or an explicit template-file action for the complete value. A policy that cannot form a working basis produces a diagnostic result with its blockers, not a purportedly usable rewrite hash.

### Readiness is conditional evidence

Report availability, conditional support, unavailability or unassessed constraints for an operation and its affected range. A paragraph can support a replacement outside a pending insertion while refusing a rewrite that changes that insertion. Whole-paragraph deletion has its own dependencies.

Readiness records contain source hash, paired target, working policy, operation family, exact available text, relevant blocker ranges, revision/comment references, and safe next actions. Unknown constraints dependent on proposed replacement text remain explicitly unassessed.

Offsets, if emitted, are zero-based Unicode scalar positions with an exclusive end in the returned logical text. They are observations, not another address family.

A scan cannot guarantee arbitrary future text is representable. Apply recomputes constraints. Do not turn uncertainty into a refusal to inspect, or an unsupported paragraph into a false promise of editability.

Candidate matches may be suggested but never silently selected. A supplied `--at` must not be retargeted. A desired rewrite or refusal does not authorize accepting an earlier revision.

## 5. Edit action contracts

```text
Usage: jubarte edit <ACTION> [OPTIONS] <FILE>

Actions:
  replace    Replace exact literal text.
  rewrite    Set desired paragraph text with minimal safe edits.
  insert     Insert an explicitly typed object.
  delete     Delete exact text or an explicit whole block.
  format     Apply typed formatting.
  fill       Fill typed content controls.
  fields     Update supported caches or an on-open setting.
  template   Generate a source-bound edit plan.
  apply      Apply a source-bound plan.

Writes require -o FILE or --out-dir NEW_DIR.
Dry runs require no destination and write no files.
Inline actions and saved plans use the same preparation engine.
```

| Action | Essential arguments | Semantics |
|---|---|---|
| `replace` | `FILE [--at ID] --find TEXT (--text TEXT\|--text-file FILE)` | Exactly one match unless deliberate occurrence/all-match selection is supplied. |
| `rewrite` | `FILE --at ID (--text TEXT\|--text-file FILE)` | Exact desired paragraph text; empty text retains the paragraph. Inline adapter calculates its working-text precondition. |
| `insert` | `FILE --kind KIND` plus position and typed payload | text, paragraph, image, table, footnote or TOC where implemented; position never infers kind. |
| `delete` | `FILE --at ID (--find TEXT\|--whole-block)` | A missing anchor cannot become whole-block deletion. |
| `format` | One of `--run`, `--style`, `--paragraph FILE`, `--list FILE`, `--section FILE` | Explicit units/scope; false differs from restoring inheritance. |
| `fill` | `FILE --data VALUES.json` | No absent-key clearing; unknown, duplicate, locked or unsupported targets refuse. |
| `fields` | `FILE (--update\|--update-on-open) --track off` | Direct-only cache/settings effects; no external fetch or macro execution. |
| `apply` | `FILE --plan PLAN.json` | Consumes only; never records into or deletes the plan. |

Anchored operations expose `--occurrence N`, 1-based, wherever the typed schema supports it. Inapplicable occurrence flags are errors. Without `--at`, replacement searches the declared story, default body, and requires uniqueness. `--all-matches` requires `--expect-count N` and tested count/non-overlap semantics.

Literal uniqueness and occurrence count every possible exact start position, including overlapping matches: `aa` in `aaa` has two candidates. Without an occurrence this is ambiguous. `--all-matches` checks the complete candidate count and refuses if their ranges overlap; it never chooses a greedy subset. These v2 matching rules do not silently reinterpret the legacy parser.

Text files are exact UTF-8. Plain text is not Markdown. A supported Markdown insertion subset requires an explicit format and advertised capability. Images use admitted local file resources.

### Shared edit options

```text
Policy:
  --track on|off                  Default on for new edits.
  --existing-revisions POLICY     keep | accept | reject | refuse.
                                 Default inline/new generated plan: keep.
  --accept-change ID              Explicit selected acceptance.
  --reject-change ID              Explicit selected rejection.
                                 Repeated IDs form a selection, not a sequence.
  --author NAME                   New marks only; default Jubarte.
  --datetime ISO8601              Timezone required; default current UTC.
  --allow-loss CODE               One specifically supported loss permission.

Outputs:
  -o, --output FILE               Primary edited DOCX.
  --out-dir NEW_DIR               New bundle; conflicts with --output.
  --emit LIST                    document,clean,patch,pdf,png,report.
                                 Default document,patch,report.
  --clean-revisions all           Required for a clean derivative.
  --report FILE                  JSON receipt, not JSON Lines.
  --preview changed|none         Default changed; bounded, completeness-aware.
  --force                        Replace a named output file only.

Preparation:
  --dry-run                      Same required candidate checks; no files.
  --expect-sha256 HASH            Full source hash.
  --expect FILE                  Optional source-bound expected-result contract.
  --require-change               Require a change to the primary document.
                                 Creating derivatives alone does not satisfy it.
  --save-plan FILE                Inline only; saves, does not apply.
                                 Conflicts with document outputs.
```

Apply cannot override plan-owned author, date, tracking, existing-revision policy or selected revision decisions. A redundant `--expect-sha256` must agree with the plan. `--expect` adds an independently identified input contract, not an override. `--save-plan` freezes targets, policy and time and excludes document-output, report, emit, expect and dry-run options; an expectation can be supplied when that plan is applied.

Advanced v1 compatibility: the new `edit apply` route accepts a legacy plan's existing source digest, or an explicit `--expect-sha256` precondition when that legacy plan lacks one. Otherwise require `--allow-unbound-plan`, available only for v1. This option never ignores an existing mismatched digest or weakens target checks; the receipt labels the request `unbound_legacy` and makes no stale-source continuity claim. V2 requires its embedded digest and rejects the waiver. This is a declared new CLI boundary, not a silent reinterpretation of v1 plan fields; legacy entry points retain their documented behavior until migrated.

Comment conveniences require a defined resulting anchor and capability coverage. The initial profile includes `attached_comment: {"text": "…"}` on replace, rewrite, inline insertion and paragraph insertion. Corresponding `--comment` flags compile to that field only on tested combinations. Unsupported combinations refuse rather than drop a comment.

## 6. Selected revisions and the working baseline

Every transaction distinguishes:

1. **Original source:** immutable bytes, hash and public target identities.
2. **Working source:** the in-memory document after selected revision decisions and explicit treatment of remaining revisions.
3. **Candidate:** the working source with new edits/review operations, serialized for validation.

Existing selected-resolution and origin-mapping machinery is reused and verified.

```json
{
  "resolve_revisions": {
    "accept": [
      {"short_id": "r12", "long_id": "document.w:ins.w:id.12"}
    ],
    "reject": []
  },
  "policy": {
    "track": "on",
    "existing_revisions": "keep",
    "text_view": "final"
  }
}
```

This illustrates shape, not a reference to the supplied plain-text fixture.

Resolve every revision against the original source. Accept/reject sets must be disjoint after aliases and both move halves are expanded. Report dependent revisions/comments carried away by a selected parent separately.

Apply selected decisions, then the policy for remaining revisions. `keep` preserves them; `refuse` refuses if any revision remains anywhere in the document, including outside the direct editing scope. An unknown revision form cannot establish that the document is revision-free. This global refusal rule also applies to unchanged and comment-only plans. Explicit global `accept`/`reject` resolves remaining history. Global policies have effects even with an empty operations list. Their capability is gated by correspondence/preservation tests.

Revision-only and comment-only transactions are valid work. No unrelated body edit is required. Decisions are never inferred from desired text, an “agreed” comment, a display projection or a refusal.

### Correspondence and anchors

Public targets remain original-source references. Anchors and `before_sha256` refer to effective logical working text. Initially, a whole-paragraph rewrite requires an exact one-to-one original-to-working paragraph correspondence. A merged paragraph cannot acquire two independent whole-paragraph rewrite owners; a split has no single working paragraph to hash. Generate read-only context for these cases and return the specific mapping limitation.

Removed targets refuse. Range editing through a merge or split is separately gated on exact, tested range provenance. Existing origin/fate mappings are an implementation starting point, not proof of that stronger guarantee. Never infer new pN addresses or expand a rewrite into another source paragraph.

`scope` authorizes direct content and comment-operation targets. `resolve_revisions` separately authorizes whole logical revision decisions and reported dependencies; adding a revision to that set never permits unrelated text edits in its affected paragraphs. Global remaining-revision policy explicitly broadens review effects to remaining history. For a `comment` through-range, every covered paragraph must be in the direct scope, not merely its endpoints. Descendant expansion and canonicalization happen before these checks.

### Rewrite baseline guard

Every saved rewrite carries `before_sha256`: SHA-256 of UTF-8 exact logical working paragraph text. It is not an XML, escaped JSON, display-Markdown or normalized-word hash. The codec is fixed by PlanV2.

Generation, `read --for-edit` and inline compilation supply the hash. The agent need not calculate it.

Apply verifies the hash **before** discarding unchanged rows. If later rejection changes the working text, an old scaffold cannot restore that rejected wording accidentally. `WORKING_BASE_CHANGED` identifies row pointers, exact current text/hash within the presentation budget, and relevant decisions. Refresh the affected selection, remove an unnecessary row or deliberately update its basis; never silently refresh desired text.

The guard costs one digest per rewrite and can require refreshing an untouched affected row. It avoids duplicated before/after paragraphs and does not force refresh after unrelated decisions. Identical duplicate paragraphs still require correct targeting and intent.

## 7. Edit-template creation

### Canonical help

```text
Generate a source-bound JSON plan for editing an existing DOCX.

Usage:
  jubarte edit template [OPTIONS] <FILE> -o <PLAN.json>
  jubarte edit template [OPTIONS] <FILE> -o -
  jubarte edit template [OPTIONS] <FILE> --dry-run

Change a row's desired text to request an edit.
Leaving an unchanged row requests no rewrite.
Removing a row requests no rewrite; explicit decisions and other operations still apply.
An empty text value empties a paragraph; delete_paragraph deletes its structure.
There is no activation flag.

Scope:
  --select IDS          Explicit IDs/closed ranges; deduplicated canonical leaves.
  --story STORY         body | a printed story | all. Default all.
  --find TEXT           Literal selection.
  --regex PATTERN       Explicit regex; conflicts with --find.
  --ignore-case         Selection only.
  --context N           Read-only neighboring source context.
  --changed             Blocks intersecting revisions/comments.
  --by NAME             With --changed; existing-mark author filter.

Working basis:
  --accept-change ID    Include this selected acceptance in the saved plan.
  --reject-change ID    Include this selected rejection in the saved plan.
  --existing-revisions keep|accept|reject|refuse
  --track on|off        Default on.
  --author NAME        Default Jubarte.
  --datetime ISO8601   Frozen in the generated plan; default current UTC.
  --expect-sha256 HASH Require the inspected source bytes.

Output:
  -o, --output FILE     Complete PlanV2; explicit - means raw JSON on stdout.
  --json               Compact result envelope when -o names a file.
  --jsonl              Typed result records and terminal result.
                        --json/--jsonl conflict with -o -.
  --report FILE        Explicit JSON receipt path; not another plan.
  --force              Replace a named output plan file only.
  --dry-run            Preflight generation; stdout receipt, no files.
                        No destination required; conflicts with -o -.

Examples:
  jubarte edit template contract.docx --find "payment" --context 1 -o payment.json
  jubarte edit template review.docx --select p12 --accept-change r49 -o edits.json
  jubarte edit apply review.docx --plan edits.json -o revised.docx --json
```

Generation reads only. Selecting acceptance/rejection here records explicit review decisions and computes a working view; it does not modify the DOCX. The receipt distinguishes selected decisions from committed decisions.

No `--runs`, `--ids-only`, `--scaffold delete`, `--scaffold format`, executable comment scaffolds, or JSONL plan dialect is added. They multiply authoring modes without resolving the measured workflow constraints. Typed structural/review operations can be added deliberately to the same plan.

### Coverage and eligibility

The default file covers the complete selected scope across all stories. It is not a claim that every Word object is editable. A saved plan is never silently truncated. If traversal or file admission cannot complete, generation refuses before publishing a misleading partial plan. The caller can choose a smaller explicit scope.

A valid selection filter matching no source entities returns `EMPTY_SELECTION`, a zero-match diagnostic and no plan file. It is distinct from applying a valid unchanged plan. A nonempty scope containing only read-only/opaque entities can produce a complete inventory with zero rewrite rows; the receipt makes that lack of editable rows explicit. Ordinary read/search still returns a normal complete empty result.

Generate one rewrite row per canonically addressed, representable paragraph leaf with exact one-to-one working correspondence. Tables expand to cell paragraphs; do not also create overlapping whole-table rewrites. Alias selectors deduplicate to canonical targets. Merged/split or removed paragraph mappings are inventoried with their limitation, not converted to competing rewrites. Comments are review entities, not ordinary body paragraphs. Text boxes and nonbody stories participate only when the exact operation/story capability is implemented.

A representable paragraph containing a pending revision can receive a row with conditional readiness. The compiler checks its actual changed spans; it does not rewrite unchanged runs or require resolution of unrelated marks. Paragraphs lacking faithful text/rewrite representation become read-only source context. Opaque regions are inventoried under a real containing reference with a reason; do not invent a public object ID.

The generator returns readiness alongside its compact receipt, with bounded blocker examples and continuations to `read --for-edit`. The plan may retain `source_context` observations from the immutable original input's final logical projection; they are not executable instructions or current readiness claims. Comment observations are their original comment bodies. Changing revision decisions cannot change this observation basis. Changed observation text is rejected against the same bound original basis, rather than silently treated as an edit. Removing observations is allowed. Working readiness is recomputed separately.

No ready/conditional flag in a saved file grants permission or replaces preparation. Actual editability, revision dependencies, source scope and resource availability are recomputed.

### Workflow cost

A complete file is useful for a deliberate whole-document review. It is not the default material to paste into the model context. The first examples select one clause or a small range. Agents can read a file in bounded sections and apply a coherent batch before preparing another.

The generator reports how many rows it produced, how many have conditional support, and what it could not represent. It does not claim to have identified what the user wants changed.

## 8. PlanV2 contract and operation composition

The delivered `jubarte-edit-plan-v2-core.schema.json` is a strict **proposed text-and-review profile**, expanded beyond the old four-operation illustration. It validates the bundled examples. It is not a replacement for the full schema generated from Jubarte's implemented dispatcher.

PlanV2 is not released, so this revision updates the proposed v2 design. Once released, its accepted fields, defaults and semantics are versioned. The separate ID contract's “version 3” is not an edit-plan version.

| Field | Meaning |
|---|---|
| `schema_version: 2` | Accepted plan model. |
| `source.sha256` | Mandatory full hash of original DOCX bytes. A moved/copied path with identical bytes is valid. |
| `policy` | Explicit tracking, remaining-revision policy and working text view. |
| `resolve_revisions` | Explicit original-source accept/reject reference sets. Empty sets are valid. |
| `author`, `date` | Recorded attribution and timezone-bearing timestamp for new marks. |
| `scope` | Direct operation scope: all source entities, or paired references with declared descendant rules. Revision decisions have separate explicit effect scope. |
| `operations` | Typed requests; an empty list can accompany review decisions. |
| `source_context` | Optional, immutable original-source final-projection observations; never actions or working-readiness authority. |
| `coverage` | Optional generated traversal/representation facts; never proof of task completion or authority. |

A document reference is exactly `{"short_id":"…","long_id":"…"}`. Both halves resolve to the same source entity. JSON Schema checks the shape; the compiled ID codec/resolver checks grammar and identity. No `pid`, bare `paraId` selector, colon-ID fallback, public run-ID family, or alternate document identifier is introduced.

Use the settled short/long contract: story-wide pN includes cell paragraphs; `cell.psN` is a scoped alias; supported box paragraphs use their structural psN inventory. Public addresses are snapshot-relative, even when the native key happens to survive a save. Missing native keys use the contract's `j:short` long form.

### Simplified authoring row

```json
{
  "kind": "rewrite",
  "target": {
    "short_id": "p0",
    "long_id": "document.w:p.j:short.p0"
  },
  "before_sha256": "GENERATED_HASH_OF_EXACT_WORKING_TEXT",
  "text": "Payment is due within 45 days."
}
```

The hash placeholder above explains the field. The bundled examples contain real hashes and validate. The only routinely edited value is `text`. Deliberate structural/review changes use their typed operations.

### Initial text-and-review profile

| Kind | Essential payload |
|---|---|
| `rewrite` | `target`, `before_sha256`, desired `text`, optional `attached_comment` |
| `replace` | `target`, exact `find`, replacement `text`, optional `occurrence` / `attached_comment` |
| `delete` | `target`, exact `find`, optional `occurrence` |
| `insert` | `target`, `text`, exactly one of `before_text`, `after_text` or `edge`; optional anchored occurrence / `attached_comment` |
| `insert_paragraph` | Existing anchor `target`, `position: before\|after`, `text`, optional `attached_comment` |
| `delete_paragraph` | `target` |
| `comment` | Paragraph `target`, `text`; optional exact `find`/`occurrence` or same-story `through` |
| `reply_comment` | Existing comment `target`, `text` |
| `resolve_comment` | Existing comment `target`, explicit `done` boolean |
| `edit_comment` | Existing comment `target`, replacement `text`; preserve recorded attribution |
| `delete_comment` | Existing comment `target`, explicit `include_replies` boolean |

Existing typed format, list, control, table, image, section, note, TOC, settings and other operations remain reachable through the pinned v1 contract until their v2 adapters pass conformance. Do not flatten them into text. Capabilities must name each runtime's actual accepted plan versions and operation profiles. “Eleven kinds” is the scope of this design artifact, not a claim that Jubarte only supports eleven engine operations.

An attached comment contains plain `text` and uses the plan's new-author/date policy. Its anchor is the complete nonempty resulting replacement, inserted text, inserted paragraph, or rewritten paragraph respectively. It is created after composition from the compiler's verified result span. Empty resulting text with an attachment refuses; no invented neighboring anchor is allowed. An unchanged rewrite or a replace whose `find` equals `text` still requests its attached comment, even though it requests no body change. No arbitrary operation-result identifier is needed.

### Exact text codec

For supported simple paragraphs: `w:t` contributes exact Unicode characters; tabs contribute U+0009; supported text-wrapping breaks contribute U+000A. The paragraph mark is outside `text`. A newline does not imply a new paragraph. A field, drawing, symbol, unsupported break or other opaque span needs a faithful typed mapping or read-only representation. It is not an editable magic placeholder.

No silent Unicode normalization, Markdown unescaping, fuzzy matching or first-match selection occurs in v2. The legacy normalization behavior remains versioned, not retroactively reinterpreted.

### Composition rules

All references resolve against one original snapshot and verified working correspondence. Operations do not implicitly observe earlier operations' inserted text.

1. Validate source, target, working-basis hashes and selected scope before classifying rows.
2. Remove validated unchanged rewrite text from the active body-mutation set. It cannot restore source text over an explicit replacement/deletion or become an output assertion. Retain any explicit attached-comment request and compose its resulting anchor; otherwise the whole unchanged row has no effect.
3. Resolve every active text span against the effective working text before mutation. Multiple disjoint edits in the **same paragraph** are required support, not an optional later feature.
4. Compose one resulting paragraph text and one corresponding output postcondition. Do not check each replacement against an obsolete intermediate paragraph string.
5. Active whole-paragraph rewrites conflict with other active text edits to that paragraph. Overlapping replacement/deletion ranges refuse. Text insertions at the same source position retain array order; insertion strictly inside a replaced/deleted span or at an ambiguously owned replacement boundary refuses unless a tested rule defines that combination.
6. Multiple paragraph insertions on the same side of one source anchor retain array order. A paragraph insertion remains in the anchor's original supported container.
7. Insertion beside a deleted or unmappable anchor refuses in the initial profile. An empty inserted/deleted paragraph is a structural effect even if aggregated text is equal.
8. Comments may coexist with disjoint text edits. Map their ranges through the composed edit. If all anchor content disappears and no supported anchoring rule applies, refuse; never silently detach/drop or attach to unrelated text.
9. A reply to a comment removed by selected revision decisions refuses. A missing entity is not an already-successful resolution. Deleting a comment conflicts with other mutations, replies or state changes affecting that deleted comment/thread. Multiple edits to one comment conflict. At most one explicit `resolve_comment` state is allowed per thread; replies can coexist with that final state, applied after reply creation. Without an explicit state request, replies preserve the original thread's state. Sibling replies retain array order. No other review action acquires implicit array-order semantics.
10. Return committed output references and exact effects. Source-to-output correspondence is verified after serialization, not guessed from old indices.

Operation pointers such as `/operations/3` are request correlation, not a new document ID family. The initial profile does not introduce arbitrary operation-result references. Use the defined attached-comment field for a new text result; later review actions use verified output references or a new read of the output.

## 9. One preparation and publication pipeline

1. Admit inputs/resources, parse the request, and check source binding.
2. Resolve explicit revision decisions and source correspondence; validate working-text guards, scope and actual changed ranges.
3. Compose text/review operations and construct the candidate in memory.
4. Establish primary effects and select the actual primary bytes: serialize a changed candidate, or use the original source bytes for a truly unchanged primary.
5. Reopen those selected bytes; check required integrity, preservation and exact operation postconditions; evaluate supplied expectations. Only after these pass, evaluate an optional primary `--require-change` condition.
6. Derive output references and digests from those same bytes. Render requested derivatives and enforce their requested policies; record each derivative's own effects and digest.
7. Recheck publication collisions/source state, publish the validated primary file or new bundle, and return a receipt naming exactly what committed.

This is implementation order, not a required series of user commands. The dry run follows the same required candidate path and stops before publication. If optional rendering was requested, it is assessed too; otherwise a dry run does not imply visual validation. No output destination means path-specific permissions/collisions are unassessed.

Current main's `preview_plan` already invokes `apply_plan` in memory. The missing work is to make required serialized-candidate checks, preservation invariants and publication decisions shared and explicit. Do not describe the current implementation as merely a schema/selector check.

### Integrity without gratuitous refusal

Report baseline findings, newly introduced findings and unresolved findings separately. Required validity/preservation invariants block publication when violated. Preserve bookmark/comment range ownership across paragraph insertion; “after p33” and “before p34” cannot be assumed equivalent around hidden range markers.

A known protected-range regression, such as the reported new bookmark/revision crossing, must be detected before publication. General quality warnings such as style preferences or font substitution are not automatically promoted to fatal errors. Define the blocking integrity rules explicitly; do not reuse an existing validator's threshold predicate without examining its semantics.

If a pre-existing condition prevents establishing a required invariant, refuse with that specific limitation. Otherwise report inherited findings without making every imperfect input uneditable. Unsupported checks are reported as unavailable, never passed.

A requested exact text change that the engine normalizes away fails its postcondition. It is not “unchanged success.” Text equality alone does not prove formatting, media, revision or package preservation.

### Unchanged outcome

If the primary transaction has no actual document effect, select an exact byte copy of the source before final checks, output-reference resolution and digest calculation. Publish those same bytes when an output was requested and report `effect: "unchanged"`. Never validate a reserialized package and then substitute the original bytes at commit. Dry run predicts that state without writing. No revisions or metadata are invented to make the result nonempty.

A selected revision resolution, a reply, a comment-state change or a structural empty-paragraph change counts as an effect even with zero newly inserted body characters. Unchanged detection includes these categories.

`--require-change` returns `NO_CHANGES_REQUESTED` before publication when primary document effect is zero. A rendered preview, report, patch file or explicitly requested clean derivative does not satisfy it. An unchanged primary can have a clean derivative that resolves existing revisions; the derivative's effects are recorded separately. The condition never waives a failed postcondition or missing anchor and is not evidence of substantive completion.

## 10. Optional expectations and honest verification

`--expect FILE` accepts a separately supplied, source-bound expectations specification. It is optional on inline edit/apply and on `check`; it must not become compulsory paperwork for every edit.

The initial expectations schema supports exact paragraph-text equality, containment and absence in the final projection. Requirements can have no executable checks; these remain **unassessed**. The specification records source hash, requirement keys/descriptions and typed checks. The receipt records the actual expectation file's path and digest.

A separate file is not automatically independent evidence. Checks copied from the agent's own edits can detect implementation mistakes but can repeat its mistaken task interpretation. Checks supplied by a user or benchmark can provide a stronger external specification. The CLI cannot infer that provenance from a filename or a caller's assertion.

For apply, expectation references resolve against the original source and map through the prepared transaction. Evaluate them on reopened actual primary bytes. The initial paragraph predicates require exact one-to-one correspondence; merged/split or removed targets refuse until an explicit alternative predicate/mapping is supported. A missing/ambiguous mapped paragraph or unsupported predicate is not a passing absence assertion. Any failed executable check blocks publication with `EXPECTATION_FAILED`.

For standalone `check FILE --expect FILE`, the expectation hash and references must refer to that inspected file. Reusing pre-edit IDs on the output is invalid. The apply receipt may expose an explicitly output-bound copy of evaluated text expectations, with new references/hash, for later rechecking. Never rewrite the caller's input expectation file.

Text absence in a chosen final paragraph is not secure redaction, and text containment is not a legal-quality test. Original/deleted text may remain in revision history. Broader requirements such as commercial reasonableness remain unassessed by this deterministic profile.

### Completion reporting

A result distinguishes:

- Operation postconditions: did this prepared request produce its specified effects?
- Integrity/preservation: which implemented checks ran on the candidate, and what do they cover?
- Supplied expectations: which checks passed/failed, and which requirements had no executable check?
- Task semantics: not assessed by the deterministic CLI unless a separately implemented, explicitly identified evaluator supplies that evidence.

A schema-valid plan, a successful dry run, many edits, a named author or “no integrity findings” cannot be labeled “task complete.”

No automatic LLM legal grader or opaque false-success classifier is placed inside the CLI. A harness may use independent grading and trajectory triage, but must bind the graded artifact to its actual path/hash and report uncertainty.

## 11. Results, counts and next actions

Machine output uses a consistent envelope:

```json
{
  "schema_version": 1,
  "command": "edit apply",
  "engine_version": "BUILD_ID",
  "runtime": {"name": "native"},
  "status": "ok",
  "source": {"sha256": "SOURCE_HASH"},
  "data": {
    "phase": "committed",
    "effect": "changed",
    "counts": {
      "requested_operations": 4,
      "unchanged_operations": 1,
      "body_operations_applied": 2,
      "review_operations_applied": 1,
      "revisions_accepted": 0,
      "revisions_rejected": 0
    },
    "verification": {
      "operation_postconditions": "passed",
      "integrity": "passed",
      "expectations": "not_requested",
      "task_semantics": "not_assessed"
    }
  },
  "artifacts": [],
  "warnings": [],
  "error": null,
  "effective_options": {},
  "continuation": null
}
```

This is an abbreviated result-shape illustration; real changed results include committed artifact records with role, media type, absolute path, digest and committed state. Build/source strings above are placeholders. Volatile timing/invocation metadata is separate from stable result data.

`status` is `ok`, `refused` or `error`. The edit workflow's `data.phase` values are `inspected`, `generated`, `validated` and `committed`; a failure reports the last completed phase or null. `data.effect` is `changed`, `unchanged` or `not_applied` for the primary document. Generation and dry-run use `not_applied`; a successful document dry-run separately returns `predicted_effect: changed|unchanged` with proposed rather than committed counts. A plan artifact may be committed with phase `generated` while no document was applied. Only actual primary publication reports phase `committed` and actual effects; post-commit reporting errors retain that publication state. Other read/compare/render commands use their documented phase values without inventing document-edit effects.

Counts distinguish operations, logical revisions, revision carrier elements, root comment threads, replies, matched records, contextual records and changed paragraphs. Do not call all of them “changes,” or invite line counting as a substitute.

`--json` emits one object including empty results and recognized-mode usage failures. `--jsonl` emits a header, typed records and exactly one terminal result. Without its terminal record the stream is incomplete. Neither protocol mixes logs, ANSI, progress or human prose into stdout. `--report FILE` is JSON, irrespective of filename suffix.

Errors identify code, operation pointer, source hash, resolved target, match count where applicable, bounded actual context and safe next actions. Keep the existing useful per-operation refusal information. Add `retryable`/commit-state detail; an unknown publication state is not authorization to retry with force.

Next-call suggestions are argument arrays, with prerequisites and purpose. Do not construct executable shell strings from document text. A pending-revision refusal offers explicit inspect/decision choices without choosing acceptance for the caller. A successful artifact receipt uses its actual output path, not a conventional filename the harness may expect.

## 12. Shared argument, stream and safety contract

### Arguments and discovery

Every value-taking option requires a value. New `--plan FILE`, `--save-plan FILE` and `--out-dir DIR` have no optional-value or inferred recording mode. Options within one action are order-independent. Repeats accumulate only where documented; otherwise duplicates fail. A target flag never begins another implicit operation.

Support `--` as the end of options. One primary operand is natural for editing; two ordered operands are appropriate for before/after comparison; multiple ordered inputs are appropriate for assembly. Long options and interspersed options are intentional extensions, not a claim of full POSIX utility conformance.

Clap enforces grammar and mutually exclusive groups. The compiled operation model also drives JSON Schema, concise/complete help, examples and capabilities. Parser validity is distinct from document-specific support. Recognizing machine mode for usage errors must respect option values and `--`; document text equal to `--json` cannot switch the output protocol.

Generate bash, zsh and fish completion definitions from that same command model as follow-on human-discovery work. Completion support must not become a prerequisite for noninteractive calls or introduce a separate action grammar.

### Streams and bounded output

Stdout contains requested data; stderr contains diagnostics/progress. Explicit machine modes produce their structured failures on stdout; human diagnostics do not contaminate them. Quiet never suppresses explicitly requested JSON, read data, comparison output or errors.

A single `-` is stdin only on commands whose help declares it. Only one document/plan/text/resource argument can consume stdin. DOCX input is buffered under package limits, not represented as streaming ZIP mutation.

Binary stdout requires explicit `-o -`, a declared target format and nonterminal stdout; it conflicts with machine receipt output. A raw JSON plan on `template -o -` is the primary product and also conflicts with result-envelope modes. JSONL is a result protocol, not an executable plan format.

Broken pipes on read-only text output must not create files or cause a long panic traceback. No TTY-dependent substantive semantics or automatic switch from text to JSON. Respect NO_COLOR, TERM=dumb, CI and a documented no-input option/environment signal.

Filters are OR within repeated fields and AND across fields. Matched records differ from contextual parents/dependencies. Revision/comment selection includes ranges that intersect a block, not only those beginning in it. Cursors bind source digest, view, filters and working decisions; partial results state known omissions.

### Source protection and publication

All input files, including plan/expectation/resource inputs, are protected from output alias collisions. Detect symlinks, hard links, normalized path aliases and explicitly identical paths. `--force` replaces only a named existing output file. It does not authorize overwriting an input, ignoring a stale source, resolving history, bypassing a lock, suppressing an invariant failure or installing dependencies.

Document writes require `-o FILE` or a new `--out-dir DIR`. Single files are staged on the destination filesystem and published by atomic rename after required checks. Bundles are staged under one parent and published as a new directory with a manifest. Existing bundle directories are not replaced in place.

A multi-file result at unrelated paths is not an atomic transaction. Prefer a new bundle; otherwise report each artifact's committed state. Recheck source/collisions before commit. Before-commit timeout or policy failure publishes no primary artifact. After-commit reporting failure names what already committed. Atomic rename alone is not proof of crash durability.

Dry runs write no document, plan, report, render, extracted asset, temporary filesystem file or persistent cache. Use bounded in-memory preparation; a runtime unable to uphold this must report the limitation rather than claim conformance. Dry-run success reserves neither source state nor output paths.

Edits default to bundle artifacts `document.docx`, `changes.diff` and `report.json` plus a manifest. Optional clean/PDF/PNG roles have explicit filenames and revision state in the manifest. The primary document remains included when using edit `--emit`. A clean derivative accepts all revisions and requires `--clean-revisions all`; it is never an implicit engine working copy.

### Tracking, fidelity and direct mode

New edits default to tracking on and remaining revisions kept. Reading final view does not accept history. A faithfully unrepresentable tracked operation refuses; direct-only operations require deliberate `--track off`. Direct mode with keep must still preserve pre-existing revisions. Do not select an engine's “clean” output if it silently resolves them.

Known unsupported content loss blocks DOCX publication unless that exact diagnosed loss has an implemented explicit allowance. Opaque preservation does not mean understanding or editing an object. Generic Markdown export is a lossy projection, not the source for partial document replacement.

No document command implicitly invokes Office, LibreOffice, pandoc, a package manager, browser or network. Only explicit self-update uses the network. Resources are admitted from declared local roots; external relationships are not fetched or executed.

ZIP expansion, entry count, XML depth/size, resource size, regex computation and rendering time/memory have documented limits. Document prose, comment bodies, filenames and properties remain untrusted data, never instructions. Reserved display delimiters are escaped; machine provenance stays separate.

### Attribution and configuration

`--author` attributes new marks; `--by` filters existing ones. Default author is Jubarte. Do not infer a person's identity from unrelated Git configuration. `--datetime` follows audited main; JSON `date` remains stable. A generated plan freezes the datetime. A fixed datetime alone cannot promise byte reproducibility across ZIP metadata, compression, fonts, resources or engines.

Flags override environment/project/user presentation defaults. Ambient configuration cannot override a saved plan's policy, accept revisions, authorize loss, disable a hash guard or add publication products. Source-specific decisions require explicit request data. Resolved options and relevant configuration provenance appear in machine output.

### Exit status

| Code | Meaning in the proposed versioned CLI |
|---|---|
| 0 | Command completed, including an explicit unchanged outcome and ordinary comparison differences. |
| 1 | Input, operational or I/O failure. |
| 2 | Standalone check/validation findings meet the requested failure threshold. |
| 3 | Refused stale, ambiguous, unsafe, unsupported request, failed operation/expectation invariant, or unmet opt-in require-change condition. |
| 4 | Requested rendering/fidelity policy failed before publication. |
| 5 | Comparison differences when `--exit-code` was requested. |
| 64 | Invalid CLI usage or malformed request/schema. |
| 124 | Deadline exceeded before commit. |

Moving usage errors from existing code 2 to 64 is breaking and belongs to a declared protocol/release boundary. Keep detailed error codes and retry classification in JSON.

An explicit repair command may commit a repaired output and still return 2 for remaining findings. A requested comparison report may exist with 5. Neither exception permits ordinary edit publication after a required check fails. Receipts state both condition and artifact state.

A Git external-diff adapter maps difference 5 to Git's 1, keeps 0, and maps actual failures to a fatal status outside 0/1. Raw `--exit-code` is not advertised as a drop-in Git convention.

Retries require source/request identity and known publication state. A transient I/O condition does not establish safe retry. A replay receipt, when supported, identifies the original source, normalized request, expectations/resource/options digests and actual output digest. Reapplying to the output is a different request, not replay.

## 13. Remaining command contracts

These contracts preserve the draft's useful workflows without making all new adapters prerequisites for the edit/template release. Ordinary help shows implemented capabilities only.

### Review

```text
jubarte review changes FILE [filters]
jubarte review comments FILE [filters]
jubarte review accept FILE (--all | selection) -o OUTPUT.docx
jubarte review reject FILE (--all | selection) -o OUTPUT.docx
jubarte review comment FILE --at ID (--text TEXT | --text-file FILE) -o OUTPUT.docx
jubarte review reply FILE --id COMMENT (--text TEXT | --text-file FILE) -o OUTPUT.docx
jubarte review resolve FILE --id COMMENT -o OUTPUT.docx
jubarte review reopen FILE --id COMMENT -o OUTPUT.docx
jubarte review edit-comment FILE --id COMMENT --text TEXT -o OUTPUT.docx
jubarte review delete-comment FILE --id COMMENT [--include-replies] -o OUTPUT.docx
```

Changes/comments are read-only, bounded and complete-aware. Filters include IDs, by-author, kind/state, story and anchor intersection as applicable. A summary exposes semantic counts and distinguishes threads from replies and logical moves from carrier elements.

Accept/reject requires all or a selection. Unknown IDs refuse. Empty filtered selection refuses unless `--allow-empty`; explicit all on a revision-free input is unchanged success. Moves resolve together and dependent effects are reported. Comments are not resolved by accepting text.

Comment/reply text is separate from body text. Existing-comment edits preserve attribution. Already-resolved/reopened state is unchanged success. Deleting a root with replies requires explicit inclusion of those replies. Mapped anchors, auxiliary parts and relationships must remain consistent.

Standalone review uses the same engine as equivalent plan operations. It is not a second set of review semantics.

### Create

```text
jubarte create [INPUT.md] -o OUTPUT.docx
jubarte create --template TEMPLATE.docx --data VALUES.json -o OUTPUT.docx
jubarte create --spec DOCUMENT.json -o OUTPUT.docx
```

No content input creates a blank DOCX. Exactly one source mode is allowed. Template/data copies supported body/sections and fills typed controls; reference-doc provides formatting only. These meanings remain distinct. Unknown/duplicate/locked controls refuse. Template revisions default to refuse unless a policy is explicitly chosen.

Page size/orientation/margins use explicit units and section scope. Conflicting reference/template/spec definitions fail rather than silently override. Local resource roots are explicit. CriticMarkup interpretation has an explicit parse/literal setting. Authorship dates and output policies follow the shared contract.

Bundles can include document, PDF, PNG and report. They use the common render pipeline; validation does not imply visual review. The typed document-spec and strict template/data adapters remain gated new engineering. “Template creation” in the implementation plan in this package means creation of an **edit-plan template**, not creation of reusable Word forms.

### Convert

```text
jubarte convert FILE --from doc|docx|md --to docx|md -o OUTPUT
```

Infer formats only when unambiguous. DOCX/Markdown projections report preserved, approximated and dropped features. DOCX→Markdown→DOCX is not lossless editing. Markdown authoring tutorials use create; a legacy conversion adapter can invoke that same engine without changing semantics. Layout output belongs to render.

Revision resolution is explicit. Asset extraction names a new output directory and records mappings; it is a gated adapter where unsupported. No implicit Office conversion or external asset fetch.

### Render

```text
jubarte render FILE -o OUTPUT.pdf
jubarte render FILE --pdf --png --out-dir NEW_DIR
```

Inputs are DOCX. PDF/PNG share one full-document layout. Page selection emits a subset but does not skip layout needed to determine it. Defaults: markup view, conventional markup, comment balloons, 100 DPI, font policy warn. DPI range is 1–1200. PDF compression is deflate or none.

Expose final/original/markup display, supported markup styles/palettes, comments mode, physical pages, changed-pages, font directories/report, DPI and font policy. Known missing content defaults to refusal. Font-policy error blocks the complete requested publication before artifacts appear.

Manifest records source/build/options/resources/font provenance, physical indices versus printed labels, page dimensions/count and available diagnostics. Changed-pages identifies review marks, not all reflow impact. Contact sheets and external-render interoperability are separately gated. Same-engine rendering is not Microsoft Word equivalence.

### Compare

```text
jubarte compare OLD NEW [OPTIONS]
jubarte compare OLD NEW --redline OUTPUT.docx
jubarte compare OLD NEW --out-dir NEW_DIR --emit PRODUCTS
jubarte compare CANDIDATE --scope visual --baseline-render DIR
```

Default is a read-only content report. `-o/--output` writes a report, never a DOCX based on extension. `--redline` is an explicit single-file document product and conflicts with bundle mode. Bundle products are report, redline, PDF, changed-page PDF, PNG and PNG archive where implemented; manifests define each role. Rendered content-comparison products use the explicit comparison redline/projection, not an unspecified input.

Scopes are content, package and visual. Content report views include Jubarte patch, CriticMarkup, unified, word, normal, context and side-by-side. Package views are semantic/raw with declared part/component/ignore filters. Inapplicable scope options are usage errors.

Reports default to markup input projection. Creating a comparison redline from inputs with revisions requires final or original input projection explicitly; arbitrary review-history merging is not promised. Neither report presentation nor output extension implicitly accepts revisions.

Two ordered inputs are normal. More inputs are accepted only for documented package comparison. An external visual baseline requires a source/render/font/page manifest and compatible geometry; no silent rescaling. Text/structure evidence and visual scores remain separate.

Stdout can be bounded. Persisted reports contain the complete selected comparison and full lines; difference status is computed over that scope, not the visible page of results. `--exit-code` returns 5 for differences across every comparison scope.

### Assemble

```text
jubarte assemble FILE FILE... -o OUTPUT.docx
jubarte assemble --files-from PATHS.txt -o OUTPUT.docx
```

Join in order; do not reconcile competing versions. At least two inputs. List-file paths are relative to the list file; stdin paths are relative to the current directory.

Defaults preserve sections, supported comments/revisions/media/notes; styles are renamed on conflict to preserve semantics. Options explicitly choose section handling, break type, comment removal, revision resolution, style conflicts and specific losses. Remap IDs/relationships without accidental list continuation. Unsupported preservation refuses rather than being hidden by the name assemble.

### Check

```text
jubarte check FILE --profile integrity|quality|layout|all
jubarte check FILE --expect EXPECTATIONS.json
jubarte check FILE --baseline ORIGINAL --require-author NAME
jubarte check FILE --repair -o REPAIRED.docx
```

Default integrity has no layout pass. Results state checks run, coverage, skipped/unavailable checks, findings and thresholds. Quality/layout rules are separate, and `--part` is an explicit partial scope. Example limits affect presentation, not actual checking.

The final line of human output states the check's scope, for example “Integrity checks passed; user requirements not assessed.” Machine results convey the same distinction. Do not name a structural success “document complete.”

Baseline tracking audit and author requirements examine new supported text changes, distinguishing preserved earlier revisions. Do not imply comprehensive formatting/media coverage from a text-only audit. Expectation rules follow section 10.

Repairs are supported conservative operations, rechecked after serialization, and always write a separate output. Remaining findings can produce exit 2 with a committed repair, explicitly reported.

### Sanitize

```text
jubarte sanitize FILE (--remove ITEMS | --author-alias NAME | --preset NAME) -o OUTPUT.docx
```

Selections are explicit and additive. Metadata categories include document properties, revision-session IDs and comments; an author alias applies only to supported recorded identities. The share-metadata preset's exact actions are documented. Dry run states coverage gaps.

This is metadata preparation, not secure redaction of prose/images/embedded files/history. Content redaction needs a separate direct operation and leakage checks. Removed secrets must not be copied into comments, patches, previews or reports.

### Capabilities

```text
jubarte capabilities
jubarte capabilities --command "edit apply" --json
jubarte capabilities --operation replace --json
jubarte capabilities --schema edit-plan
jubarte capabilities --example replace
jubarte capabilities --file FILE --operation rewrite --json
jubarte capabilities --require FEATURES --json
```

Default is a compact task map. Per-operation output includes actual input/result schemas, minimal validated example, story/target support, tracked/direct modes, dependencies, constraints, side effects, error codes and limits. The file-specific variant describes known support/unknowns; it must not equate preservable with editable.

Availability is derived from the executable runtime dispatcher, including native, Python and WASM differences. A name list is not a schema. No need to read an entire external skill file before the first basic edit. Full schemas remain discoverable without being pasted into every bootstrap response.

The new `--schema` menu includes edit-plan, operation schemas, result, snapshot, expectations and implemented authoring/render manifests. A public profile is advertised only when its conformance gate passes. No fallback dependency installation.

### Self-update

```text
jubarte self-update --check
jubarte self-update --install (--release VERSION | --latest) --yes
```

Default check never installs. Document operations never check for updates. Managed packages report their package-manager route; the command must not replace an unrelated executable. Explicit install verifies artifact integrity and separately reports signature/authenticity assurance. Noninteractive installation requires the explicit install choice and yes.

## 14. Migration and delivery

The final tree is a target contract, not permission to change existing commands silently. Existing plan-v1 continues with its recorded attribution and historical defaults. New schema fields are not accepted secretly by v1, and old missing existing-revision policy is not reinterpreted as keep.

Changing argument grammar, JSON/JSONL interpretation, ID serialization, output defaults, match normalization, revision defaults, usage exit status and quiet behavior requires a declared breaking CLI boundary, preferably the 1.0 contract. Until then, additive discovery and tested adapters can be introduced without reinterpreting existing calls. Legacy adapters must preserve exact legacy side effects or return a migration error; aliases must not silently promise compatibility.

Maintain a manifest of all current operation kinds and their runtime availability. Common supported operations must not disagree across native/Python/WASM. An unavailable adapter returns a precise capability/refusal result rather than raw-XML advice or flattened content.

Delivery order:

1. Freeze build identities, existing contracts, shared result vocabulary and regression fixtures.
2. Implement source/refusal/receipt correctness and selected-review/body/comment preparation with serialized checks.
3. Add working-text guards, exact composition, external expectations and operation/runtime schemas.
4. Expose typed inline/review/apply routes and bounded `read --for-edit`.
5. Add `edit template` on that same extractor/compiler, with complete files and compact receipts.
6. Run controlled workflow comparisons and release the supported profile.
7. Add broader create/projection/render/comparison adapters independently after their preservation/usability gates.

The separate edit and template implementation plans specify modules, dependencies, fixtures and exit criteria. They take precedence over arbitrary estimates or “one large rewrite” sequencing.

## 15. What the CLI can and cannot resolve

The CLI owns correct targeting, supported transformation, useful refusal context, exact publication receipts and truthful verification scope. It can reduce navigation and prevent late avoidable structural surprises.

The harness owns full process exit capture, real deliverable-path/hash selection, model/budget configuration, truncation/censoring records, loop detection and independent grading. A receipt cannot repair a harness that still grades a different fixed path.

The agent owns substantive interpretation and appropriate edit selection. Do not optimize for passing an author gate, maximizing edit count, or making some arbitrary edit early. Measure the first **required** committed change, eventual requirement coverage, unwanted changes and completion-claim accuracy.

A small coherent batch can reduce late failure cost, but a blanket tiny-batch policy can increase re-reading and make dependent edits inconsistent. Evaluate transaction size by workflow, retaining atomicity for each batch.

## 16. Sources and accompanying deliverables

Read `jubarte-cli-evidence-review.md` for the claim ledger and source citations; `jubarte-edit-implementation-plan.md` and `jubarte-template-implementation-plan.md` for implementation gates; and the schemas/examples for concrete proposed request shapes.

The evidence baseline is the published v0.12.0 binary (fourteen preserved probes), source at `0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3`, and the newly supplied reports. Their trace help identifies `d003d8e8d00ac50cb9782fd454f58efa4cf387e7`, a different build. The follow-up document references `aba48bd50e03584d99fbbcb2a6915e4d3b07ccf7`; its future-dated editorial audit heading is not proof of later runtime behavior.

Primary code: [edit/settlement](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit.rs), [CLI](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/cli.rs), [executable dispatch/publication](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs), [short/long ID contract](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md).

Design guidance: supplied better-cli, [POSIX utility conventions](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap12.html), [CLIG](https://clig.dev/), [clap](https://docs.rs/clap/latest/clap/), and the [GNU Bash pipeline reference](https://www.gnu.org/software/bash/manual/html_node/Pipelines.html). These guide interfaces; they do not establish an experimental reduction in Jubarte task failures.



