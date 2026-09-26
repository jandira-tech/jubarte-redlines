# Jubarte Launch Assets and Partner Contributions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the maintainer ready-to-review acquisition assets that lead to real document tasks and can be tested before spending on promotion.

**Architecture:** Reuse the runnable recipes, local playground and artifact evidence as the destination for every announcement or partner contribution. Track independent task completion and repeat use through consensual observation rather than library telemetry.

**Tech Stack:** Markdown documentation, existing package registries, original examples/skills, local browser prototype, experiment ledger.

---

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## Positioning to test

Primary headline: **DOCX to PDF and Word tracked changes, from your application or agent.**

Subheading: **Keep your document-generation workflow. Add local PDF rendering, compare two versions, and return a reviewable Word document with Jubarte's Rust engine. Python, Node and browser packages are available.**

Primary action: **Try a document locally**. Secondary action: **Copy the Python example** / **Copy the JavaScript example**. Show real outputs before architecture detail. A future Apache release can add its verified license badge; do not advertise Apache while registry artifacts still ship AGPL.

Audience variants to test:

| Visitor | Headline variant | Supporting proof | Destination |
|---|---|---|---|
| Python generation | “Your Python-generated DOCX can also be a PDF.” | 15-line BytesIO recipe and working sample | Python recipe + tested wheel support |
| Node/browser | “Generate the Word file. Preview the PDF in the same app.” | npm docx sample, worker preview, local processing | JS recipe + bundler instructions |
| Agent developer | “Give your agent a Word redline you can inspect.” | original/edited/redline/PDF plus operation report | agent recipe + exact supported capabilities |
| Template team | “Review what a template change did to your documents.” | failing-page artifact and expected-change checks | document CI example |

The first two generation recipes create new documents with the upstream library, then render their bytes. They do not establish that editing an arbitrary existing Word package through that upstream library preserves every feature. Existing-document preservation belongs to Jubarte's own guarded edit/parity work.

## Proposed upstream contribution: Python generation recipe

**Title:** Add an in-memory DOCX-to-PDF integration example using Jubarte

**Body draft:**

This example saves a document to `BytesIO` using the existing public API, then passes the resulting bytes to Jubarte for local PDF rendering. It does not add a runtime dependency to the host project or change its document model. The example includes installation instructions, a generated sample, supported wheel platforms, and a note explaining that PDF layout depends on the document features and available fonts.

The submitted recipe will link to the exact tested Jubarte release and its license. Its validation record will include a clean-environment run and the generated sample output. The contribution is an optional documented integration, so users who only need DOCX generation keep the same installation.

**Prepare before asking to submit:** complete `python_docx_pdf.py`, checked dependency versions, actual output PDF, a concise compatibility note, contribution-policy review, and the exact docs path used by that upstream repository. Do not open a promotional issue whose only content is a link. Do not contact the author of an old closed support issue as though they are an active lead.

## Proposed upstream contribution: JavaScript generation recipe

**Title:** Document a local PDF preview workflow for generated DOCX files

**Body draft:**

This example takes the buffer produced by `Packer` and renders it with Jubarte's WASM package. The browser version initializes the module in a worker, reports conversion errors, and lets the user download the generated PDF. Document contents remain in the browser; the host serves the application code and WASM asset.

The example keeps document generation in the existing library. It adds explicit bundler/WASM-loading instructions and a small synthetic fixture so maintainers can reproduce the behavior without customer files. The integration remains optional and does not change the host package's exports or dependency graph.

**Prepare:** recipe, worker initialization, two browser runs with console-pipe logs, no-content-upload observation, download verification, and a documented memory/input cap. Do not state “works in every browser” or “zero copy.”

## Proposed skill-catalog submission

**Title:** Jubarte skill for local DOCX comparison and PDF rendering

**Description draft:**

An original skill for comparing two DOCX versions into Word tracked changes, rendering a DOCX as PDF, and listing/accepting/rejecting revisions. It checks the installed package and uses documented byte APIs. It preserves original files, reports failures explicitly, and does not transmit document contents. Semantic editing examples are enabled only when the installed version advertises the required capability.

**Evidence package:** licensed skill source, runnable recipes, model/host/version matrix, six-task evaluation report, failure table, and a sample review bundle. Submission should follow the catalog's current policy and license requirements. Do not copy proprietary document-skill instructions or imply approval from the host vendor.

## A concrete design-partner offer

Offer: **Bring one difficult, shareable DOCX workflow. We will help you get a local PDF/redline result and record the exact compatibility limits.**

Participation options: use a synthetic public sample, run a private document entirely on the participant's machine while sharing only error codes/screens they choose, or explicitly authorize a minimized fixture for a private issue. Do not make uploading a confidential original the default support process.

Session structure (30 minutes):

1. Participant describes the job and current tool chain (5 minutes).
2. Participant follows the public quickstart without undocumented help (10 minutes).
3. Inspect output and identify task-specific correctness criteria (10 minutes).
4. Record success/blocker and agree whether a 14-day follow-up is welcome (5 minutes).

Success is an independently completed job or a reproducible blocker with a realistic fix. It is not a compliment, a star, a newsletter signup or a maintainer completing the task for the participant.

Draft invitation for a consenting contact or an appropriate authorized community post:

> We're testing a local Rust-based DOCX renderer and Word redline engine with Python and JavaScript APIs. If you already generate or review Word documents and struggle with the PDF/comparison step, we'd like to observe a 30-minute trial of the workflow. You can use a sample or keep your document entirely on your machine. The goal is to find where the integration works and where it fails; there's no requirement to adopt it or share the document.

This is draft text only. No messages or posts were sent during planning.

## Content sequence tied to working artifacts

| Asset | Exact deliverable | Trigger to publish | Outcome to measure |
|---|---|---|---|
| Python recipe | complete code, dependency lock, sample input/output | clean wheel smoke passes | qualified completions, later repeat use |
| JS preview recipe | complete code, asset-loading notes, supported browsers | worker/browser smoke passes | app integrations committed |
| Agent demonstration | recorded task with actual artifacts and error recovery | evaluation record available | independent replication with same skill |
| Feature scorecard | document-feature rows with successes/failures | immutable corpus/report generated | external reproductions/minimized reports |
| CI tutorial | template change → artifact showing intended/unintended diff | pilot repo retains workflow | retained CI installations |
| License announcement | effective release number, actual metadata, scope/provenance note | cleared license release in registries | previously blocked integrations resumed |

## Weekly prioritization from user evidence

Create a short issue template with fields: job, installed version, runtime, capability, observed/expected result, reproducible synthetic/minimized sample if available, privacy constraints, and whether the issue blocks repeat use. Never require a public upload to report a bug.

Assign each candidate fix a score `blocked_repeat_projects * severity * confidence / estimated_days`. This is a triage heuristic, not a replacement for correctness/security gates. Count independent projects, not the number of repeated messages from one person. Preserve core Word correctness even if a popular request asks for a shortcut that produces invalid output.

Maintain a visible compatibility backlog tied to the measured task. For example, “font fallback changes the last page of a two-column report” is actionable; “PDF isn't perfect” is not. Fixes can become new public scorecard rows and better recipes, creating a useful loop between support, quality evidence and discovery.

## Execution tasks

- [ ] Select two initial audiences and use their exact headline/recipe variants; do not launch every channel at once.
- [ ] Assemble each contribution draft with actual tested artifacts and current license metadata.
- [ ] Run five trials per initial audience; fill the pseudonymous ledger and record observed blockers.
- [ ] Prepare a concrete reviewable submission/post before requesting authorization to send it.
- [ ] After authorized distribution, measure independent completions and 14-day repeat use with the predeclared criteria in the growth program.
- [ ] Keep the two channels that produce repeat users; repair or stop the others. Do not inflate download counts with CI loops or promotional installs.
