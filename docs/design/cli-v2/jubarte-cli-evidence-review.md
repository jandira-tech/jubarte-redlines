# Jubarte CLI: final evidence review and critical reassessment

**17 October 2026 · Interface design and implementation planning; no repository implementation**

## 1. Verdict: retain the direction, change the execution contract

Use **jubarte edit template** to generate a source-bound request and **jubarte edit apply --plan** to execute it. Keep one versioned PlanV2 model for generated and hand-authored requests. A paragraph row has a paired target, a generated working-text hash, and one desired text value. Changing that value requests an edit; no activation Boolean is needed. This remains a proposed interface, not functionality demonstrated by the existing binary.

The new reports materially change the priorities. The useful first workflow must combine **explicit selected revision decisions, disjoint body edits, and compatible comments in one transaction**. A simple template that postpones revision handling would encounter the same obstacles documented in the reports. Normal apply must prepare and check its actual output internally; a separate dry run is optional. A legitimate unchanged request succeeds truthfully, with an exact source-byte primary copy. An optional require-change condition serves callers that need a nonempty primary effect.

The earlier proposal was directionally sound but not sufficient. The four requested critical assessments are:

| Axis | Verdict and evidence | Correction | Residual risk |
|---|---|---|---|
| **Conflicting** | The reports disagree about eight versus fourteen nonediting runs, misstate a first-edit median, and give incompatible score means. Our drafts also disagreed about unchanged status, template phases, comment support, and expectations. | Separate verified arithmetic, reported episodes, source facts, and proposed behavior; adopt one contract across schemas, help, examples, and implementation plans. | Without the underlying run ledger, some report classifications remain unresolved. |
| **Incomplete** | Source-only text templates do not handle the reported pending-revision workflow, semantic omissions, comment anchors, or actual delivery identity. | Include selected settlement and mixed operations; verify exact published bytes; supply bounded readiness, artifact receipts, and optional explicit expectations. | Arbitrary Word features and task semantics still require separately established support and evaluation. |
| **Undesirable** | Default no-effect refusal can encourage cosmetic edits. Blanket acceptance, automatic retargeting, and whole-document planning can reduce errors by doing the wrong work. | Allow honest unchanged outcomes, require deliberate review decisions, preserve exact targets, and support sparse inline or batch edits. | A model can still choose poor negotiating positions or omit unstated requirements. |
| **Shallow** | More operations, earlier edits, validation silence, and lower token use are weak substitutes for completed work. A renamed CLI does not establish preservation or atomic publication. | Evaluate operation fidelity, required content, preservation, delivery, and claim accuracy separately. Build the necessary executor guarantees before advertising them. | Same-engine checks and imperfect grading can miss defects; independent fixtures and adjudication remain necessary. |

The rest of this review supports those judgments and explains the limits. The final proposed contract is in [jubarte-cli-proposal-b-revised.md](jubarte-cli-proposal-b-revised.md); the two implementation plans translate it into dependencies and release gates.

## 2. Evidence hierarchy and version identity

Different kinds of evidence answer different questions. A quoted episode can demonstrate a reported failure sequence without proving its frequency. A source inspection can establish a code path without reproducing it on a particular binary.

| Evidence tier | Material examined | Defensible use |
|---|---|---|
| Controlled released-binary observations | Fourteen invocations on one disposable four-paragraph DOCX | Exact outputs, statuses, and filesystem effects of those calls |
| Pinned implementation inspection | Rust CLI/engine/tests and Python/npm/WASM adapters at the audited commit | Existing mechanisms, dispatch differences, and implementation gaps |
| Supplied experiment reports | Two accounts of a 75-run corpus; one comparison of three trajectories; an updated engineering follow-up | Attributed episode evidence and reported aggregates, subject to consistency checks |
| Primary literature and guidance | Four papers, POSIX conventions, better-cli, and CLIG | Findings within their experiments and design principles; transfer requires inference |
| Proposed artifacts and design checks | Revised contract, schemas, examples, and a small text-model validator | Internal design consistency, not implemented DOCX semantics |

The identities must not collapse into a single “current 0.12.0”:

- The released Linux asset was **jubarte-0.12.0-linux-x86_64.tar.gz**, published 11 October 2026. Its archive SHA-256 is **1858b969f9e16065dfdab54cddf7ba6a78cb68867ab54eb96e101385fff67c20**; the executable SHA-256 is **6a2d2f6017e845646b5fe35877f6c8f12e998dfc1c4110053ab04bbae09b472e**. The archive matched the release checksum. That checks downloaded-byte consistency, not independent publisher authenticity. [Release identity][release-identity] · [release][release]
- Audited main is **0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3**. It also declares 0.12.0 but differs from the released executable. No Rust/main binary was built or newly exercised in this review. No repository code, commit, or pull request was changed. [Main snapshot][main-tree]
- F2's reproduced help identifies **0.12.0-main.d003d8e8**. GitHub metadata identifies the full commit as **d003d8e8d00ac50cb9782fd454f58efa4cf387e7**, dated **2026-10-15T03:48:12Z**. This establishes the quoted build reference, not that every run used one identical executable. (F2:337–344.) [Trace build][trace-commit]
- The follow-up names **aba48bd50e03584d99fbbcb2a6915e4d3b07ccf7**, dated **2026-10-17T05:21:58Z**. Its October 18/19 editorial annotations are future-dated relative to this review; they do not change the commit date or prove runtime deployment. (H:3–48.) [Follow-up build][followup-commit]

For precise local evidence references, this document uses:

| Key | Bundled supplied file |
|---|---|
| F1 | evidence/supplied/final_turn_001.md |
| F2 | evidence/supplied/final_turn_002.md |
| D | evidence/supplied/deep-research-report(17).md |
| H | evidence/supplied/2026-10-09-agent-edit-follow-ups.md |

References such as “F2:118–130” mean lines in that supplied file. These reports now provide case-specific evidence; the earlier statement that no redline experiment evidence was available is obsolete. However, we have **not reprocessed the complete underlying 114 MB corpus**, independently reconstructed every tool-result join, or bound every grade to the actual artifact. F2 explicitly says the judged file path was not recorded. The fourteen release probes remain diagnostic tests, not an agent study. (F2:348–352.)

## 3. What the numbers support—and what must be corrected

Some reported counts are arithmetically coherent and useful descriptions. They are not controlled estimates of interface effects.

| Reported quantity | Coherent interpretation and limit |
|---|---|
| 75 runs; 56 scored; 19 unscored | The score distribution uses 56. Behavioral observations may use all 75 if stated. Missing grades must remain visible. (F1:7–17; F2:74–90.) |
| 16 gate-failed, three gate-passed with zero score, 37 positive score | Counts sum to 56. Gate failure is 28.57% of judged runs, not a proven unchanged-document rate. (F2:78–90.) |
| 306 failed of 454 rubric items | 67.40%; the five category totals reconcile. This is pooled item failure, not mean run score; correlated rubric items are not independent tasks. (F2:92.) |
| 64 UNSUPPORTED_STRUCTURE events across 35 runs | Events and affected runs are different units. Repeated failures do not imply 64 independently preventable failures or 35 rescuable tasks. (F2:116.) |
| 190 of 2,531 tool results overflowed; 139 reread calls | A reported 7.51% result-overflow rate and observable navigation burden, with different units for overflow and rereads. Origins include help/read output. (F2:337–344.) |
| Three compared trajectories: two terminal, one censored | Observed durations are 159.2 s, 1,074.5 s, and 93.1 s censored; planned operations are 21, 95, and zero. These are case studies, not treatment arms. (D:289–307.) |

The following contradictions require explicit correction:

**Eight versus fourteen never editing.** F1 says eight of sixteen gate-failed runs never invoked edit/add. F2 says fourteen and lists them, but six listed names also appear in F2's first-edit table: US2zawW, 7R7A5GW, XmK2wgS, KAGcp6f, RjnX6Cy, and LqPodoU. F2 later admits an earlier count was off by six without resolving the displayed inconsistency. Eight nonediting plus eight editing is a coherent reconstruction, not independently certified corpus truth. (F1:23,35; F2:96,120–128,350.)

**The 89% median is wrong for the displayed table.** Its nine rounded percentages are 24, 46, 50, 57, 64, 89, 93, 93, and 99: median **64%**. Exact turn fractions give **63.6364%**, from 70/110. The iNcXKY8 row is incorrectly included among gate failures; later text describes it as gate-passed with 31 operations. Excluding it produces eight observations: median **76.5%** from rounded values, or **76.4610%** from exact fractions. Neither supports 89%. F2 also mixes first edit with first dry run for 7R7A5GW: 70/110 versus 98/110. The separate claimed 78%-of-budget refusal median has no reconstructible supporting table. (F2:118–130,217–221,289.)

**The two .319 means cannot both hold.** The reports assign .319 to 37 positive runs and also to all 40 gate-passed runs, including three zeros. If the positive mean is .319, the 40-run mean is **.295075**. If the 40-run mean is .319, the positive mean is **.344865**. Three-decimal rounding cannot reconcile them. Do not select one silently. (F1:13–17; F2:86–90.)

**The SKILL comparison has unresolved denominators.** “58 of 75 read” implies seventeen nonreaders, not twelve. Twelve may refer only to judged nonreaders, but the cross-tab is absent. Difficulty, documentation-seeking, and incomplete runs also confound the score comparison. Opening a document is not understanding it. The statement that documentation placement “does not help” exceeds this observational evidence. (F1:65–67; F2:194.)

**The author gate is not a byte comparison.** Its quoted predicate is absence of revisions/comments by the expected author, alongside hundreds of other authors' revisions. That does not establish empty documents, unchanged bytes, or absence of all work. Wrong attribution, direct edits, missing outputs, wrong paths, and genuine nonexecution remain different possibilities. The gate is evidence of that predicate, not ground truth for a broader label. (F2:78–90,350–352.)

Other caution is needed. “Each run” carrying 2,531 calls conflicts with later use of 2,531 as a corpus total. The claim that nineteen unjudged runs are excluded from every rate conflicts with all-75 behavioral rates. The 37-versus-two comparison among edited runs cannot establish equivalence between validation-citing groups. (F1:7,140; F2:74,242,337,352.)

F2 reports zero confirmed false-failure cases, but describes only two investigated candidates. Both apparently involved genuine refusals. That supports retaining those truthful diagnostics; it does not prove perfect classification across every run or justify keeping the error channel “exactly as it is.” Missing target context and incomplete inline recovery options remain independently observed gaps. “Syntactically valid” also does not imply semantically supported on a document with pending revisions. (F2:261–281.)

The report's explanatory primer needs caution too: its highlighted CriticMarkup example does not by itself encode a tracked replacement; its in-place example conflicts with its claim that originals are never modified; and pending-insertion restrictions are not intrinsically limited to another author's insertions. F2 says some examples are synthetic, so they cannot establish event frequency. (F2:7–31,55–63.)

D's separate 971-request telemetry contains nonoverlapping model/harness groups and lacks a reliable task/session join. It cannot estimate a Jubarte treatment effect. A censored trajectory cannot be assigned a final failure outcome solely because it has not yet edited. Two completed cases do not establish a scaling law between plan size and effort. Rubric-item confidence intervals do not repair task mismatch or item dependence. (D:149–200,233–283,289–374.)

## 4. Strongest episode evidence and its implications

### Pending revisions were discovered too late

F2 reports repeated reading and imminent-edit announcements, followed by plan/schema repairs and pending-revision refusals. One sequence includes a repair script stripping required reply IDs after an earlier field correction. These excerpts support a costly recovery sequence; they do not support the stronger claim that the agent understood the tool perfectly at every step. Likewise, a loud refusal to cross an unsupported revision boundary is not silent disabling. (F2:98–140,190–213.)

The appropriate response is **earlier, operation-specific readiness in an ordinary read**, plus selected settlement within the same eventual transaction. A paragraph containing a pending insertion is not automatically uneditable: an unrelated range may be safe. Readiness must identify exact blocking ranges and authoritative revision references without choosing acceptance or rejection.

Acceptance is a substantive review decision. It cannot be inferred from desired text, an “agreed” comment, or a refusal. Global acceptance may make an operation executable while destroying the negotiation history the user intended to retain. Selected decisions are therefore an essential dependency, not an optional convenience after template release.

### Correct refusal prevented a wrong-target edit

D describes a plan targeting p15 with a sentence actually in p20. The dry run returned ANCHOR_NOT_FOUND; the agent recognized the mismatch and repaired it. This is evidence of a useful failure boundary, even though recovery was expensive. (D:448–481.)

A search that returns a unique literal target reduces bookkeeping, but cannot make wrong-target errors “nearly impossible.” Repeated clauses and an explicit incorrect target still require refusal. If --at is supplied, the tool must not silently relocate the edit. Bounded actual target text and candidate matches elsewhere improve recovery while preserving the caller's declared scope.

Generated targets and text hashes reduce transcription risk. They do not know which commercial clause the user meant. Identical paragraphs remain a semantic ambiguity even when both IDs and hashes are internally consistent.

### Many successful operations did not establish completed requirements

F2 reports three gate-passed zero-score runs with 31, 35, and 95 applied operations, satisfying zero of eight, three, and twelve respective rubric items. In one, validation returned no findings and the agent declared the redline complete despite an immediately preceding failed verification attempt. The report identifies missing substantive changes, alongside typographic work. These are strong **reported false-completion episodes**, subject to the missing artifact binding. (F2:217–229,352.)

The distinction is between operation fidelity and requirement coverage. An exact postcondition can establish that “Provicer” became “Provider”; it cannot establish that the required indemnity or liability position was added. Requiring at least one change, a particular author, or many revisions does not solve that problem.

The final design therefore permits optional, separately supplied expectations and explicitly reports unassessed natural-language requirements. A separate file is not automatically independent: a model can copy its mistaken interpretation into both the edit and the test. User/benchmark-supplied assertions can offer stronger evidence, but provenance must not be inferred from a filename.

### A serialized bookmark issue escaped the earlier workflow check

D reports that all 95 operations passed a dry run, after which application to a scratch DOCX revealed BOOKMARK_CROSSES_REVISION. The agent inspected XML and changed insertion from after p33 to before p34; subsequent validation reported no findings. F2 reproduces the diagnostic as a warning with exit zero. This supports a specific serialization/range-preservation regression fixture, not a claim that every such warning proves an unusable Word document. (D:483–528; F2:231–239.)

Current audited main already invokes apply_plan inside preview_plan to obtain actual in-memory counts. It is inaccurate to describe current dry run as only schema or selector checking. The required improvement is one shared preparation path that **reopens the actual selected output bytes and enforces declared invariants before publication**. [Current preview][preview] · [CLI dry-run dispatch][dryrun-dispatch]

“After p33” and “before p34” need not be structurally equivalent around hidden markers. The engine must preserve intended range ownership, not simply substitute the agent's successful workaround. Existing findings, new regressions, and optional quality warnings need separate classification; promoting every warning to failure would make many useful inputs unnecessarily uneditable.

### Output routing and status capture weakened verification

F2 shows a pipeline combining quiet mode, redirected diagnostics, tail, and a later echo of the shell's last status. Without appropriate pipeline-status handling, that status can describe tail rather than Jubarte. Truncated output can hide per-operation failures or receipts. The same quoted sequence writes intermediate outputs under /tmp but later claims the deliverable is /app/contract.docx. The excerpt demonstrates a reporting discrepancy; the complete artifact history would be needed to exclude an unquoted later copy. (F2:244–259,292–308.)

The CLI should emit compact, structured receipts with actual absolute paths, digests, committed state, and distinct operation/revision/comment counts. The harness should capture subprocess status directly and grade the artifact identified by the receipt. The agent should verify and report that artifact, rather than assume a conventional filename.

An immutable-input policy must also fit tasks that mandate a fixed delivery path. Preserve an explicit baseline source and publish to the required distinct destination; do not silently let force overwrite an input. This is a workflow-design requirement, not permission to weaken source identity checks.

## 5. Existing CLI strengths and remaining engineering

### What the released probes establish

The fourteen probes used one simple fixture with repeated words, double spacing, and a nonbreaking space. Thirteen calls read, previewed, or failed; one wrote. Their ledger captures stdout, stderr, process status, and files. [Probe ledger][probes]

| Released v0.12.0 observation | Consequence |
|---|---|
| Addressable read returned p0–p3; read --json was rejected | Address discovery exists, but universal JSON cannot be assumed across versions |
| Empty changes JSON output was empty; edit success mixed result types | A consistent explicit envelope and completeness rule are useful |
| Missing and repeated anchors refused with stable codes and counts | Preserve exact refusal boundaries; improve actual-context recovery |
| Content without an anchor became a whole-paragraph rewrite | Typed actions should make scope explicit |
| Double-space rewrite preview reported nothing changed; ordinary space matched NBSP | Exact v2 semantics require engine/adapter work, not help wording alone |
| run_operation was rejected as an unknown v1 field | Disabled scaffolds are new schema work, not an existing behavior |
| Plan plus inline flags conflicted | Do not attribute audited-main recording behavior to the release |
| Default writing created clean.docx, redline.docx, patch.diff, report.jsonl | Product roles and side effects need explicit destinations and receipts |

The whitespace observation was a dry run, not independent inspection of a published output. The source-preserving simple write does not prove universal protection against aliases or forced overwriting.

### Reuse substantive capabilities

Audited main adds read JSON, source-bound plan recording, a default adjacent modified DOCX, shared Rust/clap parsing, per-operation errors, and meaningful plan ownership checks. Python and npm reuse the parser but have dispatcher-specific exclusions. A capability list must describe the executable route, not merely library operations. [Read][read-runtime] · [recording][record] · [Python][python-parser] · [npm][npm-host]

The engine already has typed operations, selected resolution, settlement, origin mapping, range conflict checks, comment combinations, minimal rewrite machinery, and tracked-emission refusals. These are the foundations for the revised interface. Disjoint replacements in one paragraph must survive the redesign; “one operation per paragraph” would be an artificial regression. [Edit engine][edit-engine] · [mixed-plan tests][mixed-tests] · [origin mapping][origin]

There is a limitation: existing origin/fate machinery is not proof of arbitrary exact range provenance after merges or splits. An index mapping or text-based reconstruction must not be advertised as preservation-grade identity. The initial whole-paragraph rewrite contract therefore requires one-to-one correspondence; stronger mappings are separately gated.

### Fix the CLI boundary and publication layer

The present edit/add grammar uses flag order to group operations and changes behavior according to which fields are present. Main overloads --plan: inline operations select recording, while a plan alone selects application. Typed actions and a separate --save-plan remove these ambiguities without silently changing old plans. Legacy filenames equal to new action names also need routing tests. [Argument grouping][flag-groups] · [record/apply dispatch][run-flags]

JSON paths differ by command, and parser errors use a separate human framing path. Missing-anchor context is populated too late for useful repair. Main's recording summary constructs a shell command string from paths. These are concrete adapter problems: use consistent envelopes, bounded actual context, supported occurrence/source-hash flags, and structured next-action arguments. [Failure context][range-context] · [record summary][record-summary]

Publication requires new work. Main's single-document route uses direct writes; default naming can return the input name when it already ends in _modified. Bundles write sequentially and may be reused with force. Some render failures occur after artifacts appear. Plan-file atomic rename does not establish atomic document bundles, crash durability, or reliable post-commit reporting. [Output defaults][edit-target] · [single output][single] · [bundles][bundle] · [font failure order][font-fail]

The updated follow-up must also be read selectively: it reports source binding and plan occurrence while identifying missing inline adapters; its pid proposal is withdrawn, not a feature to resurrect. Its six-item historical list does not establish today's priorities. Global comment-detachment waivers, short digest prefixes, and text-containment approximations of anchor survival need scrutiny rather than automatic implementation. (H:15–44,199–218,225–350,440.)

## 6. Final design corrections and their boundaries

### One desired text, with a working-baseline guard

PlanV2 binds immutable source bytes, direct scope, paired IDs, explicit policy, selected revision decisions, author/date, and typed operations. Each rewrite adds before_sha256 over exact UTF-8 **working paragraph text**, not display Markdown, XML, or normalized words. The generator and reader calculate it.

The new author default is neutral Jubarte unless supplied; it is not a reason to infer a named person from Git configuration or mandate an author flag merely to pass a benchmark. Retain the audited CLI's --datetime spelling while keeping JSON date. Generated plans freeze attribution and time; old plans retain their versioned semantics. A fixed date alone does not establish byte reproducibility.

Consider a pending replacement of 30 days by 45 days. An initial final-view scaffold contains 45. If the caller later rejects that revision, the working text becomes 30. Comparing the old scaffold against this new baseline without a guard would recreate 45 as a fresh request. The hash makes that a WORKING_BASE_CHANGED refusal before unchanged rows are classified. Refresh or remove affected rows explicitly; never silently rebase desired text.

Validate hashes even on apparently unchanged rows. Then discard their body rewrite components before overlap checks and output postconditions, so an untouched scaffold cannot restore text over an explicit replacement. An attached comment remains an effect even when the body text is unchanged. Removing a row means no rewrite; explicit settlement and other operations can still affect the paragraph.

The initial rewrite gate is exact one-to-one original-to-working paragraph correspondence. Merges, splits, and removed targets receive explicit limitations. Otherwise two original rows could gain competing ownership of one working paragraph, or a scoped rewrite could overwrite text from another source paragraph.

### Freeze selection before settlement and separate authorities

Editing-read and template filters inspect original-source final text and original review marks, then freeze original identities before settlement. Working text is displayed separately; an original search match can disappear after rejection. Context is read-only original-source material. Source-context observations are checked against that same basis, while current readiness is recomputed.

Direct content/comment scope is separate from revision decisions. Selecting a revision authorizes the whole logical decision and disclosed dependencies, not unrelated body edits in its paragraphs. A through-range comment requires every affected paragraph in direct scope. Selected accept/reject sets resolve on the original snapshot and cannot overlap, including logical move pairs.

Apply selected decisions first, then the remaining-revision policy. Keep preserves remaining history; refuse means **any remaining revision globally**, including outside direct scope. Explicit global accept/reject operates on the remainder and must pass correspondence/preservation gates. These semantics also apply to unchanged and comment-only requests.

The version-3 ID design document is not PlanV2's schema version. Its story pN inventory includes cell paragraphs; cell.psN is an alias, and supported box paragraphs have scoped inventories. JSON uses short_id/long_id pairs. IDs remain snapshot-relative; no public run IDs, pid selectors, or guessed post-save indices are introduced. [ID contract][id-contract]

### Compose effects and preserve review relationships

All literal spans resolve against one working snapshot. Disjoint replacements in a paragraph compose into one desired result and one final postcondition. Active whole-paragraph rewrites conflict with other active text edits there. Same-position insertions have declared array order; ambiguous insertion boundaries and true overlap refuse.

The core includes optional attached_comment on replacement, rewrite, inline insertion, and paragraph insertion. Its anchor is the complete nonempty resulting span or paragraph. Empty anchors refuse. This supports explanations on new content without an arbitrary operation-result addressing language.

Existing comment ranges map through the composed edit. If the anchor disappears and no supported rule preserves it, refuse rather than detach, drop, or attach elsewhere. A reply to a comment removed by settlement refuses. Comment deletion conflicts with mutations/replies affecting that deleted thread; multiple edits to one comment conflict. A declared final thread state follows reply creation; absent that request, replies preserve the original state. These are explicit review semantics, not incidental array ordering.

### Check the actual primary bytes; report unchanged outcomes honestly

Every apply admits inputs/resources, resolves decisions and scope, validates guards/ranges, composes operations, and prepares effects. It then selects the **actual primary bytes**: a changed candidate or the exact original bytes for a genuinely unchanged transaction. Reopen those bytes, verify exact postconditions and required preservation, evaluate supplied expectations, derive output references/digests, and enforce requested derivative policies before publication.

Selecting bytes before final checks avoids validating one package and publishing a different source copy. Text equality alone is not no-effect detection: revision resolution, comments, formatting, and empty-paragraph structure can change without new visible characters. Conversely, a requested whitespace change lost by normalization is POSTCONDITION_FAILED, not unchanged success.

Require-change applies only to the primary transaction. A report, render, patch, or clean derivative cannot satisfy it. Derivative effects remain separate: an unchanged tracked primary can accompany an explicitly requested clean derivative that accepts revisions.

Dry run follows the same preparation but writes no files, including temporary filesystem files or persistent caches. This requires bounded in-memory preparation and honest runtime limitations. It does not reserve source or destination state. Without an output destination, path-specific checks remain unassessed. Single files and new bundle directories have explicit commit rules; unrelated output paths are not falsely called one atomic group. All input identities, including plans, expectations, and resources, are protected from output aliases.

### Keep expectations useful and limited

The initial expectations profile has exactly three final-paragraph text predicates: equality, containment, and absence. It binds the original source and uses verified one-to-one correspondence to inspect the actual primary bytes. Missing, merged, split, ambiguous, or unsupported targets refuse; missing text because the paragraph vanished is not a passing absence assertion.

Standalone check expects references bound to the inspected file. An output-bound copy may be generated explicitly for later rechecking; the caller's input file is never rewritten. Natural-language requirements without executable checks remain unassessed. A passing predicate does not certify the complete associated commercial requirement. Text absence in final view is not secure redaction of revision history.

Receipts separate status, phase, and effect. Status is ok/refused/error. Editing phases include inspected, generated, validated, and committed. Generation and dry run report document effect not_applied; dry run separately reports predicted_effect. Only actual primary publication reports committed document effects. Every artifact has its actual role, path, digest, and commit state.

## 7. What the guidance and literature justify

Better-cli and CLIG support predictable data output, noninteractive operation, discoverable schemas, bounded results, and actionable errors. Their examples are guidance, not authority to reinterpret Jubarte's existing safety semantics. Keep explicit JSON rather than switching protocols automatically with TTY detection. Keep force narrow. Use existing meaningful exit conditions; changing usage status from 2 to 64 requires versioned migration. “Transient” does not mean a retry is safe after uncertain publication. POSIX supports predictable argument syntax and an explicit option terminator; its conventions do not prohibit useful comparison operands or ordered file lists. [Better CLI][better-cli] · [CLIG][clig] · [POSIX][posix]

In *Loud Failures, Quiet Failures*, explicit errors were detected more often than plausible corrupted results: 91.3% versus 58.8%. The study analyzed 1,694 of 1,920 attempted trials over 24 sampled tasks and six models. Detection concerned the next message; recovery was agreement with an independent clean run, not an ideal task outcome. This supports exposing known mismatches, but does not predict Jubarte improvement or prove that prompts never help. The “about four in ten undetected” figure belongs here, not to ToolUniverse. [Loud/Quiet][loud]

The ToolUniverse audit traced errors across resource, API, wrapper, and agent interpretation. Its selected candidate cases are not a random failure-rate sample: 91 confirmed failures among 198 reviewed suspicions cannot estimate ordinary tool-use prevalence. Its useful implication is to preserve meaning through adapters—library support is not CLI availability, a saved plan is not an edited document, and a scoped graph is not full coverage. It does not validate a particular envelope or establish that one detector class universally replaces human/LLM evaluation. [ToolUniverse v1][tooluniverse-v1] · [v2][tooluniverse-v2]

*Look Before You Leap* demonstrates tradeoffs in constructed shell and Python-edit experiments. Command checks had high recall but false positives when extracted help omitted valid options. Exact replacement avoided silent errors in a stress condition but succeeded only .176 of the time; greater fuzzy applicability introduced silent errors. Its robust policy still had one silent misapplication among 8,320 trials. The transferable reasoning favors current schemas, source binding, exact ambiguity handling, and output checks—not blind fuzzy repair or a mandatory extra command. Word structure and legal task quality were not tested. [Pre-action verification][leap]

*From Confident Closing to Silent Failure* separates completion claims from programmatically measured environment state. Its experiments also show why a longer action sequence or confident closing statement is weak evidence of success. The paper's lightweight detectors are domain-dependent triage tools; their reported cross-domain degradation and false alarms do not justify adding an autonomous completion classifier to Jubarte. Use direct artifact and requirement checks when available, and audit external grading. Its lower false-success incidence in a dual-control domain is observational; the paper itself notes domain confounding. That is motivation to evaluate independent checks here, not proof that an expectations file will produce the same result. [False-success study][false-success]

The proposals here are engineering hypotheses informed by those results. No literature number is a forecast of reduced Jubarte agent failures.

## 8. Implementation, validation, and evaluation

### What has actually been checked

The delivered schemas and examples are design artifacts. The revised validation record checks two schemas, five plan examples, one expectations file, twenty-one semantic-model cases, and nineteen schema-negative cases at the reviewed checkpoint. These checks use a four-paragraph in-memory text model. **No proposed CLI execution, actual DOCX mutation, revision settlement, comment construction, package preservation, publication fault handling, rendering, or runtime conformance was tested.** The validation record is the authoritative inventory if later cases are added. [Design validation][example-validation]

Our earlier text oracle incorrectly rejected disjoint valid replacements in one paragraph, although its schema allowed them. That was a **design-validator bug, now corrected**, not evidence of a production-engine defect. It illustrates why matching schema examples is insufficient: integration gates must inspect actual packages and independently specified effects.

Cross-review also found precision gaps in the proposed schemas: end-anchored patterns admitted a trailing newline in hashes and requirement keys, and date-time enforcement depended on an optional validator component. Exact hash lengths, explicit character/zone constraints and a registered date-time check close those demonstrated gaps. A further matching review established that `aa` has two possible literal start positions in `aaa`; v2 uniqueness must count both rather than treat a greedy nonoverlapping scan as proof of uniqueness. The delivered negative cases and text model cover these corrections. They remain contract checks, not production-engine findings.

The implementation sequence is: freeze contracts and identities; retain selected settlement and mixed-operation capability; establish exact guards, composition, and actual-byte validation; add early readiness and generated discovery; then expose template generation on that shared executor. Optional expectations must not block ordinary editing behind a large task-evaluation framework. Advanced v1 operations remain available through their exact compatibility adapter until v2 counterparts pass their own gates.

### Separate responsibilities

| Layer | Responsibility |
|---|---|
| CLI/engine | Correct targeting, declared transformations, preservation boundaries, useful refusals, source/output identity, exact artifact receipts, honest verification scope |
| Agent | Interpret authorized substantive requirements, choose explicit review decisions, construct necessary edits, assess remaining work, and make claims supported by results |
| Harness | Preserve subprocess status and streams, budgets/termination metadata, actual artifact collection, reproducible input/build identity, and independent scoring |
| Evaluation process | Define outcomes, verify artifact/grade joins, adjudicate uncertain labels, distinguish legitimate refusal from defects, and report missing data |

Calling every omission an engine defect is as misleading as declaring all gate failures outside the tool's influence. Better navigation can reduce wasted budget even when the eventual cutoff is external. Conversely, a CLI cannot verify a requirement it was never given.

### Stage 1: conformance before agent trials

Use real DOCX fixtures for clean paragraphs, received redlines, duplicate anchors, cells, nonbody stories, bookmarks, comments, and known unsupported objects. Assert selected decisions and preservation of unselected revisions; unchanged source-byte identity; correct attached anchors; thread conflicts; one-to-one rewrite gates; same-paragraph composition; and exact whitespace/Unicode. Match candidates at every possible literal start so overlapping occurrences cannot become false uniqueness.

Test refusal boundaries: stale sources, mismatched pairs, out-of-scope targets, missing expectations, empty attachment anchors, unsupported tracking, and changed working baselines. Include legitimate unchanged tasks and valid supported edits to measure unnecessary blocking. Test changed candidates whose requested effect disappears, and wrong-target candidates whose output remains structurally valid, to measure missed defects.

Fault-inject before and after commit, including output alias collisions, report failure, cancellation, and bundle publication. Compare findings by mapped entity/range and material details, not only counts. Verify native/Python/WASM common semantics and honest runtime exclusions. External Word/render checks are required only for the corresponding interoperability claims, not inferred from same-engine agreement.

### Stage 2: repair the evidence and pilot the workflow

Build a one-row-per-run ledger linked to event and artifact records: task/input/prompt/build hashes, model/harness versions, documentation exposure, requested author/output, first valid request, first committed required effect, stop reason, actual output path/digest, gate predicate, grade, and exclusion reason. Distinguish edit invocations from quoted commands, dry runs from commits, and repeated error events from affected runs. Reconcile the disputed aggregates before using them as baselines.

Run a held-out paired pilot with fixed task, source, model/version, budget, and corrected harness. A 12-task, two-interface, two-repetition pilot yields 48 runs; it can expose recurring mechanisms, not establish a universal effect. Randomize execution order, start each run from clean task state, and keep the harness identical across CLI arms. Study harness interventions separately.

### Stage 3: release-relevant matched evaluation

Evaluate compact discovery, readiness/selected settlement, desired-text templates, and optional expectations separately where feasible. Include sparse direct edits, coherent multi-change batches, large documents, revision-heavy cases, legitimate no-ops, unsupported requirements, and duplicated clauses. Human usability claims require human evaluation.

Primary outcomes are required changes completed, unintended edits, preservation, correct artifact delivery, and truthful final claims. Measure **false-positive refusals** on supported valid requests and **false-negative acceptance** of failed postconditions, wrong targets, or preservation defects. Report legitimate refusals separately from nonexecution and avoid counting cosmetic edits as progress.

Secondary measures—tokens, returned bytes, calls, time to first required effect, recovery attempts, remaining budget—explain costs rather than substitute for completion. Use task/run-level uncertainty, account for paired and clustered observations, and keep ungraded/censored runs visible. Audit an artifact sample independently when LLM rubrics are used; a deterministic text predicate and a legal-quality judgment answer different questions.

Release when the received-redline workflow works with verified effects and preservation, and remaining uncertainty is acceptable for the advertised scope. The evidence supports these concrete tests and design corrections. It does not support a predicted percentage reduction in agent errors.

## References

[release-identity]: evidence/release-identity.json
[probes]: evidence/release-probes.json
[example-validation]: evidence/example-validation.json
[release]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.12.0
[main-tree]: https://github.com/jandira-tech/jubarte-redlines/tree/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3
[trace-commit]: https://github.com/jandira-tech/jubarte-redlines/commit/d003d8e8d00ac50cb9782fd454f58efa4cf387e7
[followup-commit]: https://github.com/jandira-tech/jubarte-redlines/commit/aba48bd50e03584d99fbbcb2a6915e4d3b07ccf7
[edit-engine]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit.rs
[preview]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit.rs#L1685-L1695
[dryrun-dispatch]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs#L953-L955
[origin]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit/origin.rs
[mixed-tests]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/tests/edit_plan.rs
[read-runtime]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs#L782-L865
[record]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit/record.rs#L16-L219
[python-parser]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/jubarte-python/python/jubarte_redlines/__main__.py#L986-L1075
[npm-host]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/jubarte-wasm/cli/bin/jubarte-redlines.mjs#L676-L738
[flag-groups]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/cli.rs#L262-L315
[run-flags]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs#L1333-L1406
[range-context]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit.rs#L2647-L2655
[record-summary]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/edit/record.rs#L88-L125
[edit-target]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/cli.rs#L207-L227
[single]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs#L1199-L1311
[bundle]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs#L946-L1135
[font-fail]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/src/bin/jubarte.rs#L574-L625
[id-contract]: https://github.com/jandira-tech/jubarte-redlines/blob/0ca2d3ea8191fec56d7bf0f0adb4fd492c8327b3/docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md
[better-cli]: https://github.com/yogin16/better-cli
[clig]: https://clig.dev/
[posix]: https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap12.html
[loud]: https://arxiv.org/html/2610.10062v1
[tooluniverse-v1]: https://arxiv.org/html/2609.26836v1
[tooluniverse-v2]: https://arxiv.org/html/2609.26836v2
[leap]: https://arxiv.org/html/2609.11957v1
[false-success]: https://arxiv.org/html/2606.09863v1
