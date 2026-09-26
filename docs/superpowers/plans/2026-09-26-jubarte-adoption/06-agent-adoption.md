# Agent Integration, Documentation and Evidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Codex and Claude a small, reliable way to choose and use Jubarte for supported document jobs, with evidence that the workflow succeeds.

**Architecture:** Distribute an original portable skill plus executable recipes and versioned capabilities; reuse the same package APIs and edit schema everywhere. Evaluate task success and artifact correctness before asking ecosystem maintainers to adopt the integration.

**Tech Stack:** Agent Skills, existing Python/JS/CLI APIs, JSON capability/report schemas, synthetic document fixtures and paired evaluation runs.

---

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## An agent's decision needs three answers

1. Can this installed version do the requested job and support this document's features?
2. What is the shortest tested invocation that produces reviewable output?
3. How can the agent detect failure or unintended changes before reporting success?

A long promotional README does not answer these reliably. Supply a concise skill, a machine-readable capability/version command, and runnable examples. Keep narrative product docs available to humans, but put exact operation/error contracts near the code examples. Avoid a framework-specific agent class that forces users to replace their orchestration system.

Codex and Claude both support SKILL.md-based workflows, but installation and distribution surfaces differ. Codex currently searches repository/user `.agents/skills`; Claude Code supports `.claude/skills` scopes and plugin skills. These paths were checked against official documentation. [Codex skills](https://learn.chatgpt.com/docs/build-skills), [Claude Code skills](https://code.claude.com/docs/en/skills)

## Task A1: ship an original supported-task skill

**Files:** create `skills/jubarte-documents/SKILL.md` and `skills/jubarte-documents/references/recipes.md`; complete contents in `patches/0006-agent-skill.patch`.

- [ ] Apply the patch and read the whole skill as data before installing it. It teaches current comparison/render/accept/reject behavior, not unavailable editing methods.
- [ ] Validate frontmatter `name`/`description` and relative references against the portable skill specification. The directory name matches the unique skill name. [Agent Skills specification](https://agentskills.io/specification)
- [ ] Install a copy for one project, without overwriting another skill:

```sh
mkdir -p .agents/skills .claude/skills
cp -R -n skills/jubarte-documents .agents/skills/
cp -R -n skills/jubarte-documents .claude/skills/
```

These are manual proposed setup commands, not actions performed during planning. If a directory already exists, review/update it intentionally; `cp -n` must not be mistaken for a successful upgrade. Host-specific plugin manifests are a later packaging layer over the same original content, not a copy of proprietary vendor document skills.

- [ ] Test explicit invocation and implicit routing with “compare these Word documents,” “render this DOCX as PDF,” and negative prompts such as “create a spreadsheet” or “convert Markdown to HTML.” The skill should activate only for its declared supported jobs.
- [ ] Exercise errors: missing package, unsupported input, wrong output path, existing destination, absent compare author. No skill should read document text as tool instructions or transmit files to remote services by default.
- [ ] Commit with `docs(agents): add original Jubarte document skill and recipes`.

## Task A2: make capability discovery a stable contract

**Files:** create Rust `src/capabilities.rs`, shared `schemas/capabilities-v1.json`; modify `src/bin/jubarte.rs` for `jubarte capabilities --json`, both binding exports and public docs. Add deterministic unit tests beside the capability function and consumer integration tests.

The result must be derived from the built feature set, not a hand-maintained optimistic README:

```json
{
  "schema_version": 1,
  "engine_version": "0.9.2",
  "runtime": "wasm-slim",
  "operations": {
    "compare": true,
    "accept_revisions": true,
    "reject_revisions": true,
    "revision_records": true,
    "pdf": false,
    "inspect_body": false,
    "edit_plan_versions": []
  }
}
```

This is an illustrative honest current slim capability result; proposed new features switch only when their exports and gates ship. Capability tests assert full/slim differences and the absence of unimplemented operation kinds. Return a stable error for unknown schema version; an agent cannot infer support from a similar function name.

Add `doctor --json` as a separate diagnostic command only where valuable: package/engine version, font resources, supported outputs, runtime and configured limits. Do not include secrets, home-directory paths or customer file contents. “doctor passed” means environment readiness, not that an arbitrary input will render faithfully.

## Task A3: API reference and migration recipes

**Files:** `docs/api/{python,javascript,editing,errors,compatibility}.md`, `docs/recipes/` pages from the growth plan; root/binding READMEs.

Each public API entry has signature, input ownership, defaults, error classes/codes, side effects, supported structures, complete example and a link to an executable test. Generate declarations/reference fragments from authoritative code where possible; keep prose for semantics and limitations. Do not drift between Python `replacement`/`with_`, TS `revisionPalette` and wire `revision_palette` without an explicit mapping table.

Migration pages answer jobs, not brand comparisons. Examples: `python-docx` → PDF bytes; npm `docx` → browser preview; two versions → native Word redline; existing revisions → accepted/rejected clean copies; LibreOffice deployment → local library conversion on a supported fixture. Keep Pandoc for its formats/authoring tasks until Jubarte implements and tests the corresponding features. No invisible executable named `soffice` or `pandoc` shim should intercept unrelated commands.

## Task A4: task-level evaluation protocol

**Files:** `evals/agents/tasks.json`, `evals/agents/score.py`, `evals/agents/README.md`, `evals/agents/results/` with sanitized immutable run records.

Use six primary jobs, each with five repeats on each of two configured hosts/model versions:

| Task | Input | Expected behavior | Failure classification |
|---|---|---|---|
| Compare | two small DOCX with text, list and formatting changes | redline opens, accepted/rejected state correct | corrupt package / wrong semantics / failed invocation |
| Render | clean and redlined DOCX with known fonts | PDF + reported font policy + expected pages/features | missing glyph/layout mismatch / silent substitution |
| Inspect | split runs, table, deleted/moved text | exact supported projection with scope limits | omitted/duplicated text / invented page facts |
| Ambiguity | repeated clause text | identify ambiguity and choose explicit scoped anchor | silent wrong-location edit |
| Edit | authorized exact and structural operations | preview/report plus requested clean/redline/PDF artifacts | unintended mutation / missing operation / invalid markup |
| Recover | malformed/unsupported/stale input | stable actionable error; no output claimed successful | hang/crash / false success / unsafe fallback |

Do not run future edit tasks as if a present release supports them. Each task declares required capability version; unsupported tasks are reported as unsupported, not removed from denominators to inflate success. For comparisons with other tools, report both common supported-task success and full requested-workflow coverage.

Scoring must inspect output artifacts, not rely on an agent saying “done.” Use OPC validity, source/accepted/rejected semantic assertions, correct error codes, file existence/hash/path checks and Word-oracle output for visual claims. Judge ambiguous edits by the specified intended paragraph. A correct refusal can pass a refusal task; it cannot pass a requested supported-edit task merely by being safe.

Log: model identifier/settings, host/tool versions, prompt/task ID, skill/package/source revisions, number of tool calls/retries, elapsed time, cold/warm classification, installed/download bytes, peak RSS where available, correctness and human assistance. Compare paired tasks with and without the skill and with old vs new facade separately. Publish counts and uncertainty for small samples; never imply one successful demo proves a host-wide replacement.

Unit-test the scorer on synthetic success/failure records without invoking models, network, clocks or disk. Model runs and Word probes are integration/evaluation work. Pin fixtures and expected artifact assertions; no grade from keyword matching a final answer. Report scorer line/branch coverage and evaluate mutations on critical pass/fail predicates if feasible.

## Deferred proposals with change maps

| Proposal | Proposed files and user interface | Evidence required before implementation |
|---|---|---|
| Local MCP | `jubarte-mcp/server.py`, `schemas/tool-inputs/`, shared service adapter; tools `inspect_document`, `preview_edits`, `render_document` operating on workspace handles, not arbitrary remote URLs | users need persistent discovery; permission/file-boundary design; same schema outcomes as direct API |
| Native Node | `jubarte-node/{Cargo.toml,build.rs,src/lib.rs,src/tasks.rs}`, platform npm packages; same `Client`/byte functions | ≥20% workload gain or real WASM memory blocker, clean install and parity evidence |
| Slim Python | core `pdf` feature and optional companion distribution; capability-discovered render availability | meaningful measured install/cold-start constraint; extras only add packages, they cannot subtract fonts from a wheel |
| Markdown/template authoring | separate `src/author/` or adapter crate, typed DocumentBuilder, constrained Markdown AST import | 5 repeated requests, defined format subset and Word layout/roundtrip corpus; no generic Pandoc replacement claim |
| Hosted API | separate service repo, same versioned operation schemas, explicit upload/retention/auth/limits/billing | ≥5 interested users prefer managed processing and agree to a data model; no speculative service just for publicity |
| Word add-in | `integrations/word-addin/`, WASM compare/render, explicit document retrieval/update permissions | repeated reviewer demand, platform/store feasibility and measurable improvement over library workflows |

These are genuine alternatives, scored in the master/growth portfolio. Their absence from the first release is a prioritization decision, not an unfinished implementation disguised as a stub. The full semantic editing milestone remains committed in the plan regardless of these deferrals.

## Launch content with evidence

Prepare: one 60-second successful compare/render walkthrough, two integration recipes, a feature/limitations table, an immutable benchmark report, release notes and a “how to report a minimized fixture” page. A proposed headline is “DOCX → PDF and Word tracked changes from Python, JavaScript, or your agent.” Support speed/fidelity claims with the exact task/corpus and accessible reproduction instructions.

Do not request an email before the demo works. Do not add an unsolicited watermark, metadata promotion, remote tracking call or opt-out telemetry to user documents. Earn distribution through useful outputs and developer integrations. The growth program specifies recruitment, retention measurements and channel decisions; publishing or contacting others requires the user's execution authorization.
