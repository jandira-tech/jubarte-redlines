# Jubarte edit implementation plan

## Outcome and scope

Build one editing pipeline that accepts a typed inline action or a versioned plan, prepares the actual DOCX result, verifies the requested effects and required preservation rules, and publishes an explicit output. `jubarte edit template` generates a convenient authoring form for that same plan model.

The first release slice must handle **selected revision decisions, ordinary body edits, and comments in one atomic transaction**. It must support disjoint replacements in one paragraph and comments whose anchors can be mapped through those replacements. A four-operation template demonstration without these combinations would omit the behavior most relevant to the supplied failure reports.

This is proposed implementation work. The code references below were inspected at **`0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3`**; they are not claims that the proposed contract has shipped. The experiment report identifies a different development build, `0.12.0-main.d003d8e8`. Earlier probes of the released 0.12.0 binary and the later follow-up document’s audit at `aba48bd5` remain separate evidence.

### Decisions fixed by this plan

- Use typed `edit` actions and `edit apply`; use `edit template` for source-bound authoring. Standalone `review` commands remain available, while plans can combine review and content effects.
- Normal apply and `--dry-run` use the same internal `PreparedEdit` path, including candidate serialization and required checks after reopening. A separate dry run is optional; no new public `check`/`simulate` ceremony is required.
- Preserve typed, actionable refusals. Expose the conditions earlier where possible; do not automatically accept revisions, select a similar target, or substitute a comment for a requested body edit.
- Every v2 rewrite has a generated `before_sha256` over the exact UTF-8 logical text of its effective working paragraph. The initial rewrite profile requires exact one-to-one original-to-working paragraph correspondence; merge/split mappings remain gated. The whole source is independently bound by its source-byte digest.
- Desired text is the only editable paragraph text value. There is no `run_operation` switch. A validated unchanged text component does not become an overlap constraint or output assertion; an explicit attached comment remains a separate active effect.
- Select the actual primary bytes before final reopening, validation, expectations, output-ID assignment and digesting. A true unchanged result uses the original bytes throughout those final stages, not an equivalent reserialized package substituted afterward.
- A valid request with no primary edit effects normally returns `status: ok` with `data.effect: unchanged`; a requested primary output is a byte-identical copy of the source and its `data.phase` is `committed`. `--require-change` concerns that primary edit effect, not rendered, clean or report derivatives. A desired change lost through normalization is an error, not an unchanged success.
- Generated plans use `data.phase: generated`; dry runs use `data.phase: validated`. Both have `data.effect: not_applied`. An edit dry run reports its candidate's predicted effect separately from committed effects.
- Optional `--expect FILE` evaluates source-bound typed literal expectations on the candidate. Natural-language requirements remain unassessed unless an external evaluator assesses them. A receipt records the expectations file’s digest and supplied provenance; it does not certify independence.
- Preserve access to existing v1 operations while the advertised v2 subset expands. Do not claim all current operation kinds have acquired the new end-to-end contract merely because they deserialize.

## 1. Starting point: reuse the engine’s existing strengths

The implementation already has more relevant machinery than the previous small template schema exposed:

| Existing code | What to reuse | What still needs work |
|---|---|---|
| `src/edit.rs`: `EditPlan`, `OperationKind`, `from_json` | Typed operations, strict request parsing, source digest, explicit revision policy and attribution | Versioned v2 adapter, generated schema/examples, exact text codec and effect semantics |
| `resolve_selected`, `settle` | Selected accept/reject before editing; conflict and unknown-ID errors | Full dependent-effect accounting and v2 selectors/receipts |
| `src/edit/origin.rs` | Existing paragraph correspondence and refusals through settlement | Verify exact one-to-one mappings for initial rewrites; integrate paired IDs. Existing merged-paragraph inference is not proof of exact range provenance. |
| `Transaction::resolve`, `check_conflicts` | Resolve against one source; allow disjoint text ranges; map supported comments; reject unsafe overlap | Preserve these combinations in the new adapter; distinguish unchanged context rows before conflict analysis |
| `resolve_rewrite`, `check_range`, `check_insert_position`, `opaque_structure` | Minimal changed spans; unchanged opaque spans survive; concrete structure/revision diagnostics | Exact desired-text guarantees and structured, bounded recovery data |
| `src/edit/tracked.rs` | Keep existing revisions through direct tracked emission; refuse unsupported combinations | Publish accurate operation × target × policy support, rather than blanket booleans |
| `apply_plan`, `preview_plan` | Actual in-memory edit, package production, comparison/emission, effect counts | One preparation result, checks on reopened candidates, reliable publication receipts |
| `src/validate.rs` | Package, relationship, revision, comment and range checks; text tracking audit | Explicit publication policy distinguishing required invariants, baseline findings, new regressions and optional quality checks |
| `src/edit/record.rs`, `src/edit/flags.rs` | Recording, flag-to-plan conversion and plan ownership checks | New typed-action adapter and separate save/apply semantics without changing old recorded plans silently |
| `src/cli.rs`, `src/bin/jubarte.rs`, `src/capabilities.rs` | Parser, dispatcher, output and discovery surfaces | Argument contracts, generated discovery, consistent result protocol and atomic publication |

At the inspected snapshot, `run_edit` calls `preview_plan` for a dry run. `preview_plan` calls `apply_plan` in memory to obtain real counts; it is not merely a syntax or target check. The missing shared gate is verification of the completed serialized candidates and the promised effects. Refactoring should preserve this existing execution coverage while eliminating duplicate preparation. [Source: edit pipeline](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit.rs), [CLI dispatch](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs).

## 2. Proposed architecture and transaction boundary

Use a shared normalized request between the CLI adapters and the existing engine. `PreparedEdit` is a proposed internal type, not another public plan format or a persistent reservation:

```rust
// Illustrative responsibilities, not implementation-ready signatures.
struct PreparedEdit {
    source_digest: Digest,
    request_digest: Digest,
    primary: ChosenPrimaryBytes,
    derivatives: CandidateArtifacts,
    correspondence: SourceToPrimary,
    primary_effects: EffectReport,
    checks: CandidateChecks,
    expectations: Option<ExpectationReport>,
}
```

The normal path is:

1. Parse and normalize the request; identify the actual runtime support.
2. Read and bind source/resources; freeze template/read selection and direct-operation scope against original source S before settlement; preflight output identities and collisions.
3. Resolve explicit revision decisions against S, apply the remaining-revision policy, prepare working document W, and verify original-to-working correspondence. These decisions and their dependencies do not broaden direct body-edit permissions.
4. Validate rewrite baseline hashes and classify unchanged text components; retain explicit attached-comment effects. Resolve all active ranges against W; validate scope, supported combinations and overlaps.
5. Apply in memory through the existing engine and determine the primary edit effect. Choose the actual primary bytes: the serialized edited result, or original S bytes for a true unchanged result. Retain the correspondence for this chosen artifact.
6. Reopen that chosen primary; check exact effects, required integrity and preservation rules, and optional expectations. Derive its output references and digest from these same bytes. Prepare and verify requested derivatives under their own roles and policies.
7. For dry run, return `data.phase: validated`, `data.effect: not_applied` and a separate `data.predicted_effect`; publish nothing. For normal apply, recheck destination/source/resource state, commit those exact verified bytes and return `data.phase: committed` with the actual primary effect and artifact identities.

The source bytes used for resolution and preparation must be the bytes actually hashed. Do not read a new source midway through the transaction. If policy requires detecting a changed named source before publication, recheck its digest without substituting those new bytes into the prepared edit. Do not describe dry run as reserving a source or destination.

Never validate an engine-produced reserialization and then replace it with original bytes only at commit. Even when text is equal, the packages can have different metadata, native IDs or preserved structures. Validation, expectations, returned output references, digests and publication must describe the actual chosen primary. Derivative generation is reported independently and cannot satisfy `--require-change` for an unchanged primary.

A failed operation, postcondition, required integrity check, or expectation prevents primary publication. Output files are prepared on the destination filesystem and committed atomically; a bundle is staged and published as a new directory. No input may be overwritten through a path alias, symlink, or hard link. A post-commit reporting failure must disclose which files committed. An optional render belongs to the same declared artifact/policy contract; do not claim an atomic group of unrelated output paths.

## 3. Work packages, dependencies and acceptance gates

### P0 — Freeze the executable contract and the compatibility boundary

**Modules:** `src/cli.rs`, `src/edit.rs`, `src/edit/flags.rs`, `src/edit/record.rs`, `src/capabilities.rs`; native/Python/WASM dispatchers.

Inventory the actual operation kinds, argument/default behavior, plan versions, runtime exclusions, and output protocol. Separate the new contract from old adapters. Pin the build and binary digests used in every subsequent regression or agent experiment.

Implement or centralize an authoritative normalized request model from which the accepted schema, minimal examples, common help and discovery metadata are produced. Include mutually exclusive fields, required fields, operation-specific comment support, and policy restrictions. The existing parser has validation beyond an enum’s serialization shape; generating a JSON Schema that omits those rules would create a new source of disagreement.

Focused discovery should answer “how do I replace this range?” or “what fields does a reply use?” without returning the entire operation catalog. Document-specific capability answers distinguish readable, preservable and editable content and conditional support. Avoid an `agent-info` endpoint that embeds a full schema and a mandatory multistep workflow in every response.

**Acceptance:** the schema accepts all advertised examples and rejects the same malformed requests as the runtime; a reply’s valid fields and an unsupported field are discoverable without searching a large help dump; each runtime advertises only dispatched operations.

**Dependency:** none. This is a small contract baseline, not a requirement to redesign every command before editing can improve.

### P1 — Expose typed actions and preserve mixed transactions

**Modules:** new typed action structs in `src/cli.rs`; new adapter beside `src/edit/flags.rs`; `src/edit.rs`; `src/edit/origin.rs`; `src/edit/tracked.rs`; comment/thread helpers.

Use one operation grammar per action. For example, `edit replace` has an exact `--find`, one of `--text`/`--text-file`, an explicit target or declared unique search scope, and optional `--occurrence`. `edit rewrite` explicitly names a paragraph. `edit delete` distinguishes a literal range from `--whole-block`. Flag order must not begin or switch an operation. Exact UTF-8 file payloads preserve meaningful trailing newlines rather than applying shell-style trimming.

For v2 literal anchors, count every exact start position, including overlapping matches, when checking uniqueness or selecting the 1-based `--occurrence`. In `aaa`, the anchor `aa` has two candidates: an unqualified edit refuses ambiguity, while occurrence 1 or 2 selects the corresponding range. `--all-matches` requires its expected count to match that complete candidate set and refuses if any candidate ranges overlap; it never greedily selects a disjoint subset. Apply this rule consistently to all supported literal-anchor operations. Keep historical v1 matching behavior in its versioned adapter.

`edit apply --plan FILE` only applies. Inline `--save-plan FILE` saves without applying. The new apply route never consumes or rewrites its plan. Existing record/apply syntax remains in its versioned legacy adapter during migration.

On the new apply route, v1 uses its recorded source digest or an explicitly supplied `--expect-sha256` when the plan lacks a digest. Without either, require the advanced v1-only `--allow-unbound-plan` option and label the receipt `unbound_legacy`. The waiver never ignores an existing digest mismatch or bypasses target checks. V2 always requires its embedded source hash and rejects this waiver. Keep this new CLI binding requirement distinct from v1 payload semantics and legacy entry-point compatibility.

The first v2 slice must retain `resolve_revisions` and body/comment operations in one transaction. Resolve selections against the original source, accept/reject those explicitly selected, and apply the remaining-revision policy afterward. Default `keep` preserves all unselected revisions. In v2, `refuse` is global: it refuses if any revision remains anywhere in the source after the explicit decisions, including for a review-only or otherwise no-effect request. Explicit broad `accept`/`reject` policies govern remaining history and have real primary effects. Preserve historical v1 behavior in its adapter.

Keep source IDs bound through verified correspondence; do not ask callers to guess post-settlement paragraph numbers. Current `Origin::follow` can return a merged paragraph index or refuse whole-paragraph access, and `fates` uses text matching to infer merged membership. That is useful existing machinery, not a proof of exact range provenance. Initial v2 rewrites require an exact one-to-one S-to-W paragraph mapping. Gate merged/split rewrite targets and any other operation lacking exact mapped-range support; retain corresponding legacy behavior only under its versioned contract.

Direct-operation scope and revision-decision scope are distinct. Every body target and insertion anchor must be in the declared direct scope. A `comment` with `through` requires every covered paragraph to be in that scope, not merely its endpoints. Explicit revision decisions authorize their logical changes and reported dependencies, including effects beyond the body selection; they never grant permission for unrelated body edits there. Broad remaining-history policies likewise broaden review effects only.

**Required combinations:**

- Two or more disjoint exact replacements in the same paragraph resolve against original working ranges and produce the composed desired result.
- A new comment on an unaffected span or a containing/compatible range can coexist with those replacements.
- Optional `attached_comment: {"text": "..."}` is part of the initial 11-kind schema for `replace`, `rewrite`, `insert` and `insert_paragraph`. It anchors to the nonempty resulting replacement, complete nonempty rewritten paragraph, inserted text, or inserted paragraph respectively, using plan attribution. An empty resulting attachment anchor refuses. No public operation-result ID is required.
- A comment boundary strictly inside removed text, deletion of a comment’s entire anchor, or a conflicting thread operation receives a specific refusal unless a defined preservation behavior exists.
- Selected revision resolution, an ordinary body edit, and a comment/reply can be atomic. A revision-only plan is valid.

Do not simplify conflict analysis into “one operation per paragraph.” Current `check_conflicts` already allows disjoint ranges and some comment combinations; the new adapter must preserve them. Conversely, the first slice need not support arbitrary dependent edits to newly inserted objects. Such dependencies need explicit operation-result references and separately tested semantics.

Comment mutations also have explicit composition rules. Compute each affected comment/thread and deletion dependency scope before applying operations. `delete_comment` conflicts with other mutations, replies or state operations on an affected comment/thread. Duplicate `edit_comment` requests for one comment conflict. At most one `resolve_comment` is allowed per thread, including aliases that name different entries in that thread. Replies may coexist with that one explicit final-state operation: add the replies first, then apply the requested state. Without an explicit state operation, preserve the thread's previous state. Sibling reply order follows array order; do not invent references to newly created replies or imply additional causal ordering.

Report selected and dependent revision effects separately. Both halves of a move are one logical decision; conflicting accept/reject requests refuse. A comment whose anchor was removed by settlement remains a meaningful error, not a reason to silently create an unanchored reply.

**Acceptance:** a received-redline fixture supports one explicitly selected settlement, two disjoint replacements, and a compatible comment in a single transaction. Test each of the four attached-comment anchor forms, including a refused empty anchor. Unselected revisions remain under keep; global refuse still applies to no-op/review-only requests. Removed/unsupported merged targets cannot redirect an edit, out-of-scope covered comment paragraphs refuse, and conflicting thread mutations prevent the entire primary write.

**Dependency:** P0’s model and schema boundary. The existing origin and settlement implementation are reused, not reimplemented from scratch. [Source: original IDs](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit/origin.rs), [source tests for mixed settlement](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/tests/edit_plan.rs).

### P2 — Introduce `PreparedEdit` and verify serialized candidates

**Modules:** `src/edit.rs` and a focused preparation/checks module if it keeps responsibilities clear; `src/validate.rs`; `src/bin/jubarte.rs`; existing output helpers.

Refactor `apply_plan`/`preview_plan` behind the shared preparation layer. Keep a single normalizer, resolver, execution path and candidate checker. Dry run and normal apply differ at publication, not in whether a structural candidate is actually constructed.

Choose the actual primary bytes before final checks. For an edited primary, use the serialized edited candidate. For a true unchanged primary, use the original source bytes without reserialization. Reopen those chosen bytes and verify:

1. Required package, relationship, revision, comment and range invariants.
2. Exact requested logical text and structural effects through verified correspondence.
3. Preservation of designated source structures outside the intended operation footprint.
4. Explicit revision decisions and new-effect attribution where promised.
5. Any caller-supplied typed expectations.

Define publication policy explicitly. Do not blindly use the current `validate::fails` predicate: at the inspected snapshot, every finding in the `word` or `edits` sets fails that predicate, including warnings. Some source documents already contain nonfatal findings. Distinguish input admission failures, baseline findings, introduced/worsened preservation regressions, and optional quality checks. Compare findings by code, corresponding entity/range and material details, not aggregate counts.

For `BOOKMARK_CROSSES_REVISION`, add a focused regression from the reported insertion case and preserve the intended bookmark endpoints. A known new range-preservation violation can block the candidate even if the general validator calls it a warning. Do not suppress the diagnostic, delete the bookmark, or assume all warnings are equivalent. Do not automatically change “after p33” to “before p34”: those positions may differ structurally.

Exact text checks must detect whitespace or soft-break changes normalized away by the current rewrite algorithm. Compare decoded logical characters; do not reuse a word-normalized diff as the postcondition oracle. A requested change that disappears is `POSTCONDITION_FAILED` or a precise unsupported-effect refusal, not a successful no-op.

With no semantic primary effects, retain the original source bytes through reopening, validation, expectations, output-ID generation, digesting and publication. A rewritten ZIP with identical text is not evidence of a change. Conversely, an empty paragraph insertion, revision resolution, new attached comment, thread-state change or formatting change can be a real primary effect without changing aggregate visible text. A clean derivative that resolves revisions or a newly rendered PDF does not turn an unchanged primary into a changed primary.

**Acceptance:** both dry run and apply reject the same bad serialized candidate before publication; a valid unchanged request produces a source-identical primary copy; disjoint text effects and allowed comment anchors remain correct; the bookmark regression is detected; an unrelated baseline warning is reported without arbitrary blockage.

**Dependency:** P1’s normalized operations and effect identities. Publication changes must land before the new path advertises transaction guarantees. [Source: current validation policy](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/validate.rs).

### P3 — Add early boundary context and actionable errors

**Modules:** `src/inspect.rs`, read/view adapters, `src/edit.rs` range helpers, `src/capabilities.rs`, result/error serialization.

Reuse the engine’s projection and support logic to expose relevant restrictions during targeted reads and template generation. A paragraph containing r7 is not wholly uneditable: changes outside r7 may already work. Return range-specific facts, the revision’s authoritative ID, and the kinds of decisions or operations that the engine supports.

Define selection before settlement. For `edit template` and `read --for-edit`, resolve IDs and literal/regex searches against original S and its final logical text; evaluate `--changed` and `--by` against S's original revision/comment marks. Freeze the selected original identities before applying decisions to W. Thus accepting a revision for inspection cannot make its original changed-block selection disappear. Neighboring context is immutable original-S context and does not expand direct edit scope.

`read --for-edit` must accept the same `--track` and `--existing-revisions` policy inputs and explicit selected decisions used to derive template/apply's W. Its hashes and readiness then describe that declared working basis, while selection remains based on S. Bind continuation cursors to source digest, frozen selection, filters, projection/codec, tracking/remaining-revision policy and selected decisions. A changed decision/policy cannot continue a previous query silently.

A missing or ambiguous anchor should return a stable code, operation pointer/ID, source digest, requested target, actual match count, bounded exact context and a continuation/read action. Candidate matches elsewhere are suggestions, not automatic retargeting. Return structured argv or parameters rather than shell text assembled from document content. Document text remains untrusted data.

Do not claim early metadata can predict every later refusal. Some failures depend on the final changed ranges, cross-operation interactions, serialization, comparison, or rendering. Keep those checks in P2 and identify their stage when they fail.

**Acceptance:** an agent can see the pending-insertion restriction before drafting a large plan; the same restriction is enforced on apply; normal text in that paragraph remains editable; a wrong p15 receives usable p15 context and bounded candidates without changing p20 automatically. Original-mark filters retain their selected identities after settlement, read/template working hashes agree under the same policy, and changed policy invalidates an old cursor.

**Dependency:** P0 and P1. This can run alongside P2 as long as both reuse the same support rules.

### P4 — Generate and apply guarded desired-text templates

**Modules:** proposed `src/edit/template.rs`; the v2 request model; exact logical text codec; existing source/target inspection and preparation layer.

The canonical route is `jubarte edit template FILE -o edits.json`. With a named plan, return a compact receipt rather than printing the complete document again. Offer selected-clause generation and explicit all-document coverage; do not require a full template before a one-line edit or known sparse batch.

Each eligible paragraph row has a canonical target, `before_sha256`, and one desired `text` value. Eligibility for an initial rewrite requires an exact one-to-one original-S-to-working-W paragraph mapping; merged/split targets remain gated. `before_sha256` is generated from the exact logical working text after the template’s explicit selected revision decisions and remaining-revision policy. It is a SHA-256 over UTF-8 text, not raw XML, normalized display Markdown, a paragraph index, or a digest of serialized JSON string escapes. The source-byte digest remains separately mandatory for v2.

Freeze the codec in the schema version. Supported `w:t` characters, tabs and soft breaks must have exact meanings; the structural paragraph ending is outside `text`. Unsupported fields, drawings or opaque content are preserved by supported mappings or represented as read-only context with a reason. Do not create editable magic placeholder strings and expect the model to preserve them reliably.

`source_context` contains immutable observations from original S: paragraph text in its final logical projection, original comment bodies, or explicitly identified opaque regions. Re-extract supplied text from that same basis when validating observations; never reinterpret it as text from W or a current readiness assertion. Changed supplied context text refuses; removing a context record changes nothing. Working readiness and blockers are separate observations derived from W and actual policy. Neither context nor caller-edited readiness flags expands direct-operation scope.

At apply, settle explicit revision decisions, resolve the original target through correspondence, then validate `before_sha256` against the actual effective working text. A mismatch is `WORKING_BASE_CHANGED`, with a bounded explanation of affected rows. A removed target has its own correspondence/refusal result. Do not silently regenerate an authored row or put rejected text back into the document.

After baseline validation, exact equality of desired and current text makes the row's text component unchanged. Remove that unchanged text component from active overlap checks and output postconditions. An explicit `attached_comment` remains active and its resulting nonempty anchor is still checked; it must not disappear with an unchanged rewrite. The same rule applies to an unchanged replacement carrying a comment. Thus an untouched scaffold cannot undo an explicit replacement appended to the same paragraph, while a deliberately requested comment still occurs. Changed rewrites participate in normal conflict analysis. Removing a row requests no rewrite; explicit revision decisions and other operations still apply. Empty `text` leaves an empty paragraph; deletion requires a typed delete operation.

Changing revision decisions after template generation can invalidate some rows. The initial implementation should require refreshing or removing only those affected rows and return enough context to do so. Do not silently refresh their desired values. A review-only request can omit body rows entirely. This strict rule is deliberately simpler than an automatic plan-rebase system and must be included in usability evaluation.

No-effect behavior is normal and explicit:

| Request/result | Primary behavior |
|---|---|
| Generate a plan | `status: ok`, `data.phase: generated`, `data.effect: not_applied`; committed plan artifact if saved, no committed document effect |
| Valid untouched template, normal apply | `status: ok`, `data.effect: unchanged`, `data.phase: committed`; byte-identical source copy at the requested output; zero effects |
| Same request with `--require-change` | `status: refused` with `NO_CHANGES_REQUESTED`; no primary artifact, even if derivatives were requested |
| Same request under edit dry run | `status: ok`, `data.phase: validated`, `data.effect: not_applied`, `data.predicted_effect: unchanged`; or the same required-change refusal; no writes |
| Selected revisions or comments changed but body rows unchanged | Report actual review effects; this is not a zero-effect result |
| Desired text differs but engine fails to realize it | Postcondition failure; never treat as allowed unchanged |

`--require-change` evaluates only primary edit effects; producing a report, render, plan or clean derivative cannot satisfy it. It does not waive global remaining-revision refusal, an invalid source, stale baseline, unsupported operation, or integrity failure. It is not a substitute for verifying required clauses, and examples must not encourage cosmetic changes simply to satisfy it.

The template schema is one adapter to the plan; it should not be a second execution language. A full exported file must declare its selected coverage and unrepresented regions. Runtime support governs actual edit eligibility. Public paired IDs follow the agreed short/long contract; do not introduce the withdrawn `pid:` selector or promise that IDs survive arbitrary saves.

**Acceptance:** a changed row applies without another activation step; an untouched row cannot override an explicit edit; per-row baseline guards prevent accidental restoration after rejection; comments/revision decisions can coexist; no-effect and structural-effect cases are distinguished correctly.

**Dependency:** P1–P3. Template generation must not ship with stronger guarantees than the apply path that consumes it.

### P5 — Add optional expectations and truthful receipts

**Modules:** proposed `src/edit/expect.rs` or a similarly focused checker; result schema; CLI output options; P2’s candidate checker.

`--expect FILE` is an optional, separately supplied, versioned expectations document. On edit/apply, bind it to the same original source digest as the edit request. Its initial executable predicates are exact paragraph-text equality, literal containment and literal absence, always in the final projection. Occurrence-count predicates, configurable projections, review-state predicates and block-absence predicates are deferred. A missing or unresolved target is a refusal, not a passing absence check.

Resolve original target references through verified correspondence for the actual chosen primary. Compute check results from its reopened bytes before publication, including when those bytes are unchanged S. Reject malformed, stale, unsupported or failed required checks with distinct codes and a bounded report. Do not fetch linked instructions or execute code from the expectations file.

For standalone `check OUTPUT --expect FILE`, the expectations hash and references must be bound to that inspected output snapshot. Old source IDs are not valid output IDs merely because their spelling still parses. Apply may return an explicitly rebound copy of evaluated text expectations using the chosen primary's actual output hash and references; it never rewrites the caller's input file or assumes all original targets survived.

Natural-language requirements may be carried as reference text, but the CLI reports them as `unassessed`. An attached typed check passing does not automatically mark the entire associated requirement satisfied. If a caller simply copies each operation’s desired result into the expectation set, that supplies useful additional checking but is not an independent coverage assessment.

The result records the expectations path, file digest, schema version, actual checks run, target paragraphs, fixed final projection and any caller-declared provenance. Treat declared provenance as unverified unless the implementation has a real verification mechanism. Do not label a file “independent” because it was supplied through another flag. There is no mandatory requirements ledger.

Receipts distinguish requested changes, prepared changes, committed effects, unchanged rows, selected revision decisions, dependent effects and comment changes. They identify the actual output path, media role and digest, and state which checks ran, were unsupported or remained unassessed. Counts of operations, logical revisions, revision XML entries and comments must not be conflated.

In JSON mode, `status` is `ok`, `refused` or `error`. Successful plan generation uses `data.phase: generated`, edit dry run uses `validated`, and primary publication uses `committed`. `data.effect` is `changed`, `unchanged` or `not_applied`: templates and dry runs use `not_applied`; an edit dry run carries `data.predicted_effect` separately. Plan/artifact committed state is recorded on the artifact and never implies a document effect. Errors and empty/unchanged success use the same envelope. JSONL has a terminal result; missing termination is incomplete. `--quiet` suppresses human status, not explicit machine data or errors. Human success text states the committed artifact and verified effects, not “all requested negotiations complete.”

**Acceptance:** a mechanically valid edit missing a declared literal expectation is refused before publication; a valid unchanged state can satisfy expectations; an unassessed natural-language requirement remains unassessed; receipts retain file identity and do not overstate provenance.

**Dependency:** P2; schema/discovery from P0. P5 is optional for basic editing and must not block P1–P4 behind a large task-evaluation framework.

## 4. Coverage: preserve 24 existing kinds without claiming universal v2 support

The inspected capability list contains these 24 kinds. Selective revision resolution is a plan-level field, not a twenty-fifth kind. [Source: capabilities](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/capabilities.rs).

| Group | Existing kinds | Initial treatment |
|---|---|---|
| Required text and review slice | `replace`, `insert`, `delete`, `rewrite`, `insert_paragraph`, `delete_paragraph`, `comment`, `reply_comment`, `resolve_comment`, `edit_comment`, `delete_comment` | Include in the initial v2 operation model where the exact target/policy combination passes preparation and preservation tests. Guard dependent reply deletion and anchor changes explicitly. |
| Formatting and paragraph composition | `format_paragraph`, `format_run`, `merge_paragraphs`, `list` | Keep v1 access; add v2 individually with exact effect and conflict contracts. Do not force text-only templates to represent them. |
| Structured content and document features | `insert_table`, `watermark`, `fill_control`, `insert_footnote`, `insert_image`, `page_setup`, `insert_toc` | Keep v1 access; gate new adapters on resources, stories, tracking support and candidate checks. |
| Direct and sensitive effects | `redact`, `settings` | Keep their actual supported legacy routes; expose v2 only with specific mode and verification contracts. Redaction must not copy removed content into a receipt/redline. |

This is an implementation sequence, not permission to silently drop unsupported kinds from a saved request. Unsupported new-schema operations refuse with a discovery path; supported v1 plans continue through their exact versioned adapter. A cross-runtime capability matrix states what each executable actually dispatches.

Tracking support is a combination, not a single boolean. Explicitly gate `track: off` with `existing_revisions: keep` until the adapter preserves old history while making only new changes directly. Never obtain that primary by choosing an internal clean artifact that resolves old revisions. Unsupported combinations refuse during preparation. An explicitly requested clean derivative has its own revision-resolution effect and receipt; it does not change the primary's effect or satisfy the primary `--require-change` condition.

## 5. Focused fixture and test plan

Use existing helpers in `tests/common/docx.rs` and preservation assertions already used by the edit tests. Extend existing suites first; the suggested new names below are proposed tests, not assertions that they exist. Do not add a large snapshot suite that only repeats implementation choices.

| Fixture / suggested test | Existing home or proposed suite | Required assertion |
|---|---|---|
| `selected_revision_then_disjoint_edits_and_comment` | `tests/edit_plan.rs`, `tests/edit_keep_revisions.rs` | Explicit settlement, two nonoverlapping replacements and a compatible comment produce one coherent artifact; other revisions remain. |
| `source_ids_survive_selected_settlement` | `tests/edit_resolved_ids.rs` | Earlier paragraph merge/deletion and table changes cannot redirect original targets. |
| `rewrite_requires_one_to_one_working_paragraph` | proposed `tests/edit_template_v2.rs` | One-to-one surviving paragraphs can rewrite after unrelated settlement; merged/split targets refuse until exact support is advertised. |
| `revision_decisions_do_not_expand_body_scope` | proposed `tests/edit_plan_v2_scope.rs` | Selected/dependent revision effects may extend beyond a body selection, but unrelated body edits there refuse; every paragraph in a through-comment must be in direct scope. |
| `global_refuse_applies_to_review_only_and_noop` | proposed `tests/edit_plan_v2_scope.rs` | Any remaining revision anywhere after selected decisions refuses under v2 refuse, including untouched and review-only requests; keep succeeds where otherwise supported. |
| `comment_boundary_inside_replacement_refuses_atomically` | `tests/edit_structured_comments.rs`, `tests/edit_comment_threads.rs` | Unsupported anchor mapping returns the real operation error and no primary write; supported containing/unchanged ranges still succeed. |
| `attached_comments_use_nonempty_result_anchors` | proposed `tests/edit_plan_v2_comments.rs` | All four attachment kinds bind correctly; empty resulting anchors refuse; an unchanged rewrite retains its explicit new comment effect. |
| `comment_mutations_and_final_thread_state_compose` | proposed `tests/edit_plan_v2_comments.rs` | Delete conflicts with affected mutations/replies/state; duplicate edits and multiple state requests refuse; replies retain array order and explicit final state is applied last, otherwise prior state survives. |
| `pending_insertion_only_blocks_its_changed_span` | `tests/edit_opaque_refusals.rs` | Unchanged revision/hyperlink/field spans survive a rewrite; touching the unsupported span gives its specific restriction. |
| `wrong_target_and_repeated_anchor_return_context` | `tests/edit_occurrence.rs`, `tests/edit_anchor_normalization.rs` | Exact occurrence semantics, bounded context and no automatic retargeting. |
| `overlapping_literal_candidates_are_counted_and_batch_refused` | `tests/edit_occurrence.rs`, proposed v2 CLI tests | `aaa` with anchor `aa` reports two candidates; no occurrence refuses ambiguity; replacing occurrence 1 with `X` gives `Xa`, and occurrence 2 gives `aX`. `--all-matches --expect-count 2` refuses overlapping ranges without publishing; no greedy subset is applied. V1 compatibility expectations remain versioned. |
| `dry_run_and_apply_check_the_same_serialized_candidate` | proposed `tests/edit_prepared_result.rs` | Both paths detect introduced range/integrity failures; dry run leaves no files. |
| `unchanged_primary_is_chosen_before_checks_and_ids` | proposed `tests/edit_prepared_result.rs` | Validation, expectations, output references and digests use original bytes for unchanged primary; no post-validation package swap; derivative work cannot satisfy require-change. |
| `insert_after_bookmarked_paragraph_preserves_endpoints` | proposed `tests/edit_bookmark_preservation.rs` | Reproduce the report’s structure with a minimized fixture; the intended bookmark range survives serialized output. |
| `baseline_finding_does_not_hide_new_same_count_finding` | proposed `tests/edit_prepared_result.rs` | Finding identity/correspondence detects the new defect despite unchanged counts; unrelated baseline warnings are disclosed. |
| `rewrite_whitespace_and_soft_break_are_exact` | `tests/edit_run_glyphs.rs`, proposed v2 tests | Exact requested characters are realized or refused; normalized-away change is never counted as an effect. |
| `template_rejection_invalidates_only_affected_baselines` | proposed `tests/edit_template_v2.rs` | No accidental restoration; current context enables refreshing/removing affected rows without rewriting authored values. |
| `selection_and_source_context_remain_bound_to_original` | proposed `tests/edit_template_v2.rs` | Search uses original final text and changed/by use original marks; selection freezes before settlement; context remains original-S text; W readiness matches read/template policy; changed decisions invalidate cursors. |
| `unchanged_scaffold_does_not_conflict_with_explicit_replace` | proposed `tests/edit_template_v2.rs` | Validated unchanged rows cannot restore text or block explicit disjoint operations. |
| `unchanged_copy_and_require_change_are_distinct` | proposed `tests/edit_template_v2.rs`, `tests/cli_output.rs` | Byte-identical no-effect output and correct status; opt-in no-change refusal; review-only/empty-paragraph structural effects count. |
| `expectations_bind_source_and_check_reopened_output` | proposed `tests/edit_expectations.rs` | Equality/contains/absence use chosen-primary final text; stale/missing targets refuse; standalone check requires output-bound references; natural-language text stays unassessed. |
| `publication_failure_reports_actual_commit_state` | `tests/cli_failure_paths.rs`, `tests/edit_single_output.rs` | Source aliases/collisions refuse; precommit failure writes no primary; postcommit reporting failure identifies committed files. |
| `v1_plan_and_v2_schema_do_not_change_each_other` | `tests/cli_plan_record.rs`, `tests/cli_parser.rs` | Historical defaults/unknown fields remain versioned; schemas match actual dispatch; apply does not consume a plan. |

Run focused suites while changing each work package. Run the project’s required broader gates at integration boundaries. Add render/Word interoperability tests where making concrete claims about bookmark/revision presentation; ordinary action-wrapper changes do not justify re-running unrelated layout research. Do not “fix” a failing preservation test by deleting its assertion or accepting a new baseline merely because it matches current output.

## 6. Workflow evaluation and division of responsibility

The reports motivate the changes; they do not supply a controlled estimate of their effects. The two 75-run summaries contain conflicting no-edit counts and timing/mean statements. Their expected-author gate cannot establish that the required clauses changed, and the reported grader path is missing. R17’s three trajectories are useful case studies; its separate telemetry groups have disjoint models and no reliable session/task join. Do not optimize release acceptance against those uncorrected aggregate rates.

After deterministic regressions pass, run a paired pilot on held-out tasks with the same inputs, model/version, prompts, budget and corrected harness. Compare the current interface with the new interface; randomize order and repeat. A pilot of 12 tasks across six families, two interfaces and two repetitions is 48 runs—enough to identify large recurring failures, not to promise a precise universal effect. Expand where release-relevant uncertainty remains.

Keep the harness identical across CLI arms. Directly capture subprocess return code and separate stdout/stderr; record the actual committed artifact path/digest and grade that artifact. Do not make an agent infer success from `tail`, a shell’s last-command status, or a hash of an assumed output. If investigating harness changes, run that comparison separately.

Measure three outcomes independently:

1. **Tool correctness:** declared operations and selected revision decisions are faithfully realized, or correctly refused, with required preservation.
2. **Task completion:** the final artifact satisfies the independently specified user requirements. Integrity, author attribution and revision count are separate checks, not substitutes.
3. **Claim accuracy:** the final response’s completion/failure claims match the artifact and the checks actually run.

Secondary measures can explain mechanisms: schema-recovery calls, bytes returned/persisted, time to the first required effect, remaining budget at a refusal, recovery success, and time/resource cost. Early cosmetic edits, fewer refusals or more revisions are not automatically better. A legitimate unsupported requirement should be reported accurately rather than scored as a silent success.

Capture stop reason, timeout/cancellation, model/provider/version, task/input/prompt hashes, CLI commit/binary digest, timestamps, structured argv, return codes, committed receipt and independent grade. Mark incomplete traces as right-censored where appropriate. Missing grades remain visible in delivery outcomes rather than disappearing from the denominator. Use an adjudicated artifact sample to check any LLM rubric; do not treat correlated rubric items as independent trials.

The CLI owns request correctness, effect fidelity, preservation, output identity and truthful checks. The harness owns status capture, execution budgets, termination metadata and artifact collection. The model owns choosing and implementing the requested substantive changes within its authorization. Each layer can help the others, but a CLI flag cannot prove an unexpressed legal obligation has been handled.

## 7. Migration gates and material risks

### Migration gates

- Keep plan-v1 parsing, omitted-field defaults, matching rules and output semantics under an exact compatibility adapter. New v2 fields and behavior need explicit version ownership.
- Do not silently accept unknown v2 fields or downgrade unsupported v2 kinds to v1 behavior. A valid schema is necessary, not sufficient: document-specific support is checked during preparation.
- Resolve ambiguity between `edit <action>` and legacy filenames equal to action names. Adding subcommands is not automatically backward-compatible with every positional input. Test the routing boundary and document the migration path.
- Preserve legacy author/date/output behavior where promised; new canonical defaults are explicit. Plan-owned policy never changes because of environment or process defaults.
- Do not introduce three template aliases when none shipped. Preserve a real legacy alias only when it routes to the same versioned semantics.
- Preserve availability of all 24 existing v1 operation kinds on each runtime that currently supports them until replacement wrappers pass conformance. New v2 support does not silently remove legacy access or imply a formerly unavailable runtime now supports an operation. Runtime differences are disclosed; a common supported operation must not quietly mean different things across native/Python/WASM.
- Update examples, help, schema, error recovery and wrapper metadata together. An example should end in a real apply and artifact check when the task is to make changes; dry run is available without becoming mandatory ceremony.

### Risks that need explicit handling

| Risk | Implementation response |
|---|---|
| Desired-state template encourages large or cosmetic rewrites | Selected generation, sparse batch examples, exact changed-span compilation, effect previews; measure required changes rather than volume. |
| Revision decisions invalidate full-template baselines | Per-row hashes, affected-row diagnostics, explicit refresh/removal; no automatic rebase of desired text. |
| Existing origin mapping is mistaken for exact merge/split provenance | Initial rewrites require one-to-one S-to-W paragraph mapping; gate broader mappings and test actual ranges before advertising them. |
| Validated candidate differs from the bytes actually committed | Choose primary first; reopen/check/derive output IDs and digests from those same bytes, including original S for unchanged output. |
| Same-engine checks repeat the same mistake | Reopen serialized bytes; test independent range/effect invariants; use selected external interoperability evidence for claims requiring it. |
| Baseline problems cause every write to fail | Separate admission, baseline findings, new regressions and optional quality policy; do not suppress material output defects. |
| Extra safety flags become another planning burden | Put normal checks in `PreparedEdit`; keep expectations and required-change policy optional; preserve direct typed actions. |
| Author gate rewards cosmetic work | Enforce supplied attribution separately from substantive task completion; retain a neutral default when no author is prescribed. |
| Template support is mistaken for all Word support | Advertise operation/target/policy coverage and opaque regions explicitly; keep advanced adapters gated. |
| Normalized no-op classification hides failed edits | Exact postconditions distinguish already-satisfied state from an unrealized requested difference. |

The release gate is a working received-redline workflow: discover a relevant contract, select any required revision decisions explicitly, apply disjoint body and comment changes, verify the actual candidate, publish the intended artifact, and report exactly what happened. Command count, template simplicity, author-gate passes and validator silence cannot substitute for that result.

## Source notes

Implementation references are pinned to the inspected commit in the links above. Relevant existing tests include [selected settlement](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/tests/edit_plan.rs), [source ID mapping](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/tests/edit_resolved_ids.rs), and [opaque boundaries and safe rewrites](https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/tests/edit_opaque_refusals.rs).

The direct-evidence inputs are the user-supplied `final_turn_001.md`, `final_turn_002.md`, `deep-research-report(17).md`, and `2026-10-09-agent-edit-follow-ups.md`. They support the concrete recovery and workflow cases discussed here. They do not establish a measured benefit for `before_sha256`, the template format, `--require-change`, or the complete proposed redesign; those benefits are hypotheses for the specified evaluation.
