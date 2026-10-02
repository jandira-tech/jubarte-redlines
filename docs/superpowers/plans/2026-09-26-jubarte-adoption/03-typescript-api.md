# JavaScript and TypeScript APIs Implementation Plan

> **Status (2026-10-01, bd262981): not started as written — patch 0003 must
> not be applied.** The repo substituted raw additive WASM exports
> (`applyEditPlan`/`previewEditPlan`, `EditOutput.patch`,
> `diffDocuments`; bf1e0b21) plus the second npm package
> `jubarte-redlines` (a CLI over `jubarte-wasm`; a6d5e8ae). No JS facade
> or worker exists — re-evaluate JS1/JS2 against those raw exports before
> building either.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Node/browser usage readable and typed without breaking existing WASM users.

**Architecture:** Layer a handwritten byte facade over existing generated WASM exports, followed by a worker lifecycle layer. Keep native Node as an evidence-gated optional backend.

**Tech Stack:** JavaScript ESM, TypeScript declarations, wasm-bindgen, Vitest/v8

---

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## First-release examples

Node (proposed additive subpath; existing raw imports remain unchanged):

```typescript
import { readFile, writeFile } from "node:fs/promises";
import { compare, revisions, toPdf } from "jubarte-wasm/api";

const original = await readFile("contract.docx");
const modified = await readFile("contract-v2.docx");
const redline = compare(original, modified, { author: "Legal review" });
for (const item of revisions(redline)) console.log(item.kind, item.part);
await writeFile("redline.docx", redline, { flag: "wx" });
await writeFile("redline.pdf", toPdf(redline, { revisions: "word" }), { flag: "wx" });
```

Browser (proposed subpath, explicit async initialization):

```typescript
import { load } from "jubarte-wasm/web-api";
const api = await load();
const render = async (file: File): Promise<Blob> => {
  const input = new Uint8Array(await file.arrayBuffer());
  return new Blob([new Uint8Array(api.toPdf(input))], { type: "application/pdf" });
};
```

This first facade's compute is synchronous; the browser example is appropriate only for small tasks/worker code. Main-thread UI use must adopt the worker milestone. `await` around a synchronous call does not keep the event loop responsive.

`jubarte-wasm/slim-api` has compare/accept/reject/revisions only, without a misleading PDF method that fails at runtime. Existing `jubarte-wasm`, `/node`, `/web`, `/slim`, `/node-slim`, `/web-slim`, `.wasm` exports and CJS usage stay intact.

## Task JS1: implement typed options and records

**Files:** complete source in `patches/0003-javascript-api.patch`: `jubarte-wasm/npm/api/{facade,index,web,slim}.mjs`, corresponding `.d.ts`/`types.d.ts`, additive package exports; `jubarte-wasm/api-tests/facade.test.mjs`, private development package/lock and `vitest.config.mjs`.

- [ ] Apply test/tooling files first and run `npm --prefix jubarte-wasm ci` then `npm --prefix jubarte-wasm test`. Expected red step: missing facade import, with coverage enabled in the script. Test utility is one deterministic implementation of the owned raw-engine interface, not a mock of wasm-bindgen internals.
- [ ] Apply complete implementation and export hunks. Check all legacy keys remain present. These source files are outside generated `node/` and `web/` directories; rebuilding WASM must not overwrite them.
- [ ] Run coverage and TypeScript consumer compilation. Expected unit assertions: exact option forwarding, argument validation before native work, schema validation/freezing, error cause preservation, no-PDF slim surface and no shared-buffer compute.

```sh
npm --prefix jubarte-wasm ci
npm --prefix jubarte-wasm test
```

In this planning pass: 27 reference wrapper tests passed, 100% lines/branches in `facade.mjs`. Browser initialization, `.d.ts` consumer resolution and actual generated WASM calls are separate integration gates. The test does not prove Rust fidelity.

- [ ] Rebuild all four WASM targets from a clean commit using `jubarte-wasm/build-npm.sh`, sequentially. Run `npm pack --ignore-scripts` from the publish directory; install that exact tarball into separate Node ESM/CJS and TypeScript NodeNext/bundler consumer fixtures. Exercise old and new exports. Verify declared files are in the tarball.
- [ ] Run browser integration with Vite plus `vite-plugin-console-pipe`, testing module-relative default WASM loading and an explicit URL. Capture startup/console errors in the terminal. Test Chromium and Firefox before claiming both; do not claim every bundler without a recipe.
- [ ] Commit source/tests with `feat(js): add typed options facade over existing wasm exports`; commit rebuilt artifacts only through the existing release ownership process.

## Error, ownership and schema details

`JubarteApiError` has `code: "ENGINE_FAILURE" | "REVISION_SCHEMA"` and `cause`. The first code wraps legacy string/Error throws; it deliberately does not claim to distinguish invalid DOCX, missing fonts, unsupported operation and internal bugs without a Rust error DTO. New edit/report exports will use stable explicit Rust codes, not string matching.

The revision decoder validates `type`, string metadata, nullable move fields and formatting properties, maps `type` to `kind`, and freezes nested arrays. The original raw `getRevisions` still returns its JSON string. Unknown current-version kinds/schema fail loudly; versioned future records can add an explicit unknown-kind policy when the contract requires it.

Inputs are Uint8Array (Node Buffer qualifies). SharedArrayBuffer is refused by the reference facade until copied by the caller; bytes must be owned at any asynchronous/native boundary. Returned generated WASM bytes retain their existing ownership behavior. Never claim zero-copy across WASM linear memory or N-API without measurement and lifetime proof.

## Task JS2: worker-backed production API

**Files:** create `jubarte-wasm/npm/worker/{protocol,client,node-entry,web-entry,node,web}.mjs` and `.d.ts`, `jubarte-wasm/api-tests/worker.test.mjs`, integration fixtures; add explicit `/worker` and `/web-worker` exports.

Public proposed contract: `createClient({ maxPending: 8 }) -> Promise<Client>`; `client.compare(a,b,{author}, {signal?}) -> Promise<Uint8Array>`; `client.toPdf(bytes,pdfOptions,{signal?})`; `await client.dispose()`. No implicit global pool. One worker initially; add a pool only after workload evidence. Raw synchronous API remains available.

Protocol request is `{id, method, args}` and response exactly one of `{id, ok:true, value}` / `{id, ok:false, error:{code,message}}`. Only enumerated methods are dispatchable; never evaluate code or property names from untrusted documents. Validate incoming IDs/message shapes before resolving promises. Copy inputs to owned buffers before transfer, so the caller's ArrayBuffer is never detached. Reject a full queue before allocating those copies.

Lifecycle states: STARTING → READY → DISPOSING → CLOSED, with FAILED from startup/crash. A request submitted outside READY is rejected except explicitly queued startup policy. A job-level engine error leaves the worker usable. A WASM trap or malformed protocol frame marks that worker failed; reject the active request and recreate only through a documented client policy. `dispose` is idempotent and rejects/drains queued requests exactly once.

Cancellation: pre-aborted rejects without copying; queued abort removes that job; active abort terminates its dedicated worker, invalidates active state, then restarts for queued work if configured. Do not promise cooperative interruption of synchronous Rust. Keep request metadata outside transferred buffers; always remove signal listeners when settled. Event listeners are removed on disposal and no worker keeps Node alive after disposal.

- [ ] Build a deterministic fake WorkerPort with manually delivered messages/errors; no real timers in unit tests.
- [ ] Write explicit assertions for each state transition, duplicate/unknown response IDs, full queue, queued abort, active worker crash, init failure, job error with sibling success, repeated disposal, and caller buffer non-detachment.
- [ ] Implement the finite-state client, then Node/browser factories. Keep logic shared; platform entrypoints only adapt messaging/init.
- [ ] Run `vitest --coverage` with ≥90% lines/85% branches on lifecycle/queue modules. Use real workers only in integration tests with coverage and an external timeout. Test browser responsiveness and Node shutdown.
- [ ] Publish worker recipes and memory/concurrency guidance only after the packed-artifact integration tests pass.

## Native Node decision gate

NAPI-RS is feasible but adds platform packages, libc/CPU coverage, provenance and maintenance. Prototype behind the same interface only if native end-to-end throughput improves by ≥20% for a named production workload or fixes a measured WASM memory limit; this is a decision threshold, not a predicted improvement. Include cold start, installation and RSS in the comparison.

Copy JS input buffers into Rust-owned memory **before** asynchronous dispatch; retaining a JS Buffer while JS can mutate it is not sufficient. Node-API ABI stability does not eliminate per-platform artifacts. An AsyncTask abort signal must not be described as guaranteed running-compute interruption. [NAPI-RS async tasks](https://napi.rs/docs/concepts/async-task), [Node-API](https://nodejs.org/api/n-api.html)
