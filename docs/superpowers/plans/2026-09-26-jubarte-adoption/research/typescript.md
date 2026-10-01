# Jubarte JavaScript/TypeScript API and distribution research

> **Status (2026-10-01, bd262981): the facts under "Verified current
> state" describe 0.9.2.** As of 2026-10-01 (0.10.1 + unreleased main): a
> second npm package, the `jubarte-redlines` CLI, exists (a6d5e8ae); raw
> edit/diff WASM exports shipped (bf1e0b21); there is still no JS facade
> or worker. See CHANGELOG [0.10.1]/[Unreleased]. The recommendations
> below stand as written.

Prepared 2026-09-26; source inspected, no Cargo commands or source edits performed. Recommendations describe new APIs, not exports already available. The root checkout has unrelated uncommitted comparer/test changes, which must be preserved. Memory lookup had no relevant results.

## Verified current state

The project already has real JavaScript bindings and TypeScript declarations; its main shortcoming is a low-level, synchronous API and thin distribution/testing, not an absent binding. `jubarte-wasm/Cargo.toml`, `jubarte-wasm/src/lib.rs`, and `jubarte-wasm/npm/package.json` currently describe `jubarte-wasm` 0.9.2. Its Rust dependency is canonical `jubarte-redlines = { path = "..", default-features = false }`; the library path is `jubarte::`. Do not replace this with the old project name. Four generated builds exist: Node full/slim and browser full/slim. Existing exports:

```ts
compareDocuments(original: Uint8Array, modified: Uint8Array, author: string): Uint8Array;
acceptRevisions(docx: Uint8Array): Uint8Array;
rejectRevisions(docx: Uint8Array): Uint8Array;
getRevisions(docx: Uint8Array): string;
docxToPdf(docx: Uint8Array, compress?: boolean | null,
  revisions?: string | null, revision_palette?: string | null): Uint8Array;
pdfPageCount(pdf: Uint8Array): number;
initPanicHook(): void;
```

Node is CommonJS and initializes immediately; browser ESM requires `await init()`. `pdfPageCount` returns 0 for unreadable input and does not perform strict validation. `getRevisions` returns JSON with `type`, `author`, `date`, `part`, `moveGroupId`, `isMoveSource`, `formatChange`, and `text`; enum values are `Inserted`, `Deleted`, `Moved`, `FormatChanged`. Author/date/text use empty strings for absent fields. `formatChange` is null or `{ changedProperties: string[] }`. Preserve these legacy contracts.

`docxToPdf` actually already accepts revision style and palette although the npm README's API table still lists only a single argument. `src/convert/mod.rs` also has `docx_to_pdf_report -> ConvertedPdf { pdf, font_report }`; bindings do not expose this report yet. This is a small, valuable extension, not a reason to rebuild the renderer.

`jubarte-wasm/build-npm.sh` builds four wasm-pack targets sequentially and stamps `ENGINE_COMMIT.txt`. It rejects a dirty tree, with an explicit override that should never be permitted for release builds. `.cargo/config.toml` carries required WASM compile flags; do not override `RUSTFLAGS`. `npm-smoke.mjs` exercises full/slim compare, revisions, accept/reject and PDF. It compares package length and revision JSON because ZIP ordering is nondeterministic; length is not a sufficient long-term fidelity oracle. Preserve the smoke but strengthen the semantic parity suite.

`.github/workflows/release.yml` currently builds desktop CLI archives and Python wheels; it has no npm build/publish job. `.github/workflows/ci.yml` tests the Rust engine but has no npm package consumer/browser/type suite.

`~/T/reconciliation_plan/GET_JUBARTE_RUST.md` is partly stale: it still says the adapter is benchmark-owned and names the legacy GitHub remote. Canonical `jubarte-wasm/README.md` now identifies this checkout as the single binding source and the benchmark consumer as a symlink. Live `origin` is `https://github.com/jandira-tech/jubarte-redlines.git`.

## Recommendations and scores

Scores are planning judgment, not measured probabilities. Feasibility assumes the existing engine behavior is retained and the advertised target matrix is tested.

| Proposal | Feasibility | Desirability | Order and acceptance gate |
|---|---:|---:|---|
| Add typed options-object facade and typed revisions over existing WASM | 0.98 | 0.98 | First; old imports/signatures remain unchanged; generated definitions pass consumer type tests |
| Add worker-backed asynchronous Node/browser clients with bounded queues | 0.91 | 0.96 | Next; CPU work stays off event loop, lifecycle/abort/queue behavior tested |
| Add canonical inspect/edit protocol shared with Python and CLI | 0.84 | 0.98 | After core edit semantics; JS does not reimplement XML transformations |
| Expose typed conversion report, font substitutions, engine/capability metadata | 0.96 | 0.94 | Small extension; makes fidelity limitations visible and reproducible |
| Add structured error codes at the Rust service boundary | 0.91 | 0.95 | Before stable agent-facing API; preserve legacy string throws in old exports |
| Publish/test packed npm artifacts with trusted publishing | 0.94 | 0.98 | Before launch; tests consume actual `.tgz`, not workspace paths |
| Add NAPI-RS native Node backend behind the same facade | 0.78 | 0.76 | Conditional: parity and measured end-to-end native benefit justify platform burden |
| Rename existing npm package to an unverified new name | 0.95 | 0.34 | Defer; retain `jubarte-wasm`; new branded scope only after ownership check |
| Replace existing browser WASM with WASI-only bindings | 0.55 | 0.22 | Reject for initial roadmap; loses existing browser deployment simplicity |
| Advertise full ZIP DOCX transforms as streaming | 0.40 | 0.18 | Reject; current engine requires whole-package access, and fake streams hide memory costs |
| Add telemetry to document processing by default | 0.95 | 0.12 | Reject; adoption metrics should use public distribution aggregates and opt-in feedback |

## Public contract: additive facade

Use `jubarte-wasm/api` for the ergonomic full Node API, and `jubarte-wasm/web-api` for the initialized browser client. Keep existing low-level `jubarte-wasm`, `/node`, `/web`, `/slim`, `/node-slim`, `/web-slim` untouched. Explicit entrypoints avoid a bundler accidentally importing Node filesystem code. A later branded wrapper can re-export this API without requiring migration.

New pure byte-oriented facade:

```ts
export type RevisionKind = "Inserted" | "Deleted" | "Moved" | "FormatChanged";
export interface Revision {
  readonly type: RevisionKind;
  readonly author: string;
  readonly date: string;
  readonly part: string;
  readonly moveGroupId: number | null;
  readonly isMoveSource: boolean | null;
  readonly formatChange: { readonly changedProperties: readonly string[] } | null;
  readonly text: string;
}
export interface CompareOptions { readonly author: string }
export type PdfOptions = { readonly compress?: boolean } & (
  | { readonly revisions?: "conventional" | "word"; readonly palette?: never }
  | { readonly revisions: "custom"; readonly palette: string }
);
export interface RedlineApi {
  compare(original: Uint8Array, modified: Uint8Array, options: CompareOptions): Uint8Array;
  revisions(docx: Uint8Array): readonly Revision[];
  accept(docx: Uint8Array): Uint8Array;
  reject(docx: Uint8Array): Uint8Array;
}
export interface FullApi extends RedlineApi {
  toPdf(docx: Uint8Array, options?: PdfOptions): Uint8Array;
  pageCount(pdf: Uint8Array): number;
}
```

Defaults deliberately match Rust/legacy exports: PDF compression false and conventional revision coloring. The quickstart can explicitly opt into compression and Word coloring. Do not silently use wall-clock timestamps: existing comparer default is the epoch and `compare_documents_with_options` already supports caller-specified dates. A new shared `CompareOptions` revision should add an explicit RFC3339 date and named `word`/`powertools` preset after those options are wired through the Rust boundary. Do not expose internal tuning fields such as `in_stamp_residual`.

Node usage executable once the facade patch lands:

```ts
import { readFile, writeFile } from "node:fs/promises";
import { compare, revisions, toPdf } from "jubarte-wasm/api";

const [before, after] = await Promise.all([
  readFile("original.docx"), readFile("revised.docx"),
]);
const redline = compare(before, after, { author: "Legal review" });
await writeFile("redline.docx", redline);
await writeFile("redline.pdf", toPdf(redline, { compress: true, revisions: "word" }));
for (const change of revisions(redline)) {
  console.log(change.type, change.part, change.text);
}
```

Browser usage executable after browser facade lands:

```ts
import { createJubarte } from "jubarte-wasm/web-api";

const jubarte = await createJubarte();
const before = new Uint8Array(await originalFile.arrayBuffer());
const after = new Uint8Array(await revisedFile.arrayBuffer());
const redline = jubarte.compare(before, after, { author: "Legal review" });
const blob = new Blob([redline], {
  type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
});
const url = URL.createObjectURL(blob);
const anchor = document.createElement("a");
anchor.href = url;
anchor.download = "redline.docx";
anchor.click();
setTimeout(() => URL.revokeObjectURL(url), 0);
```

This browser sample is explicitly synchronous and suitable for small documents or a worker; use the asynchronous worker client as the main production quickstart. Synchronous legacy exports remain valuable inside workers and short-lived command-line scripts.

## Async facade: truthful cancellation, ownership, resource limits

Use a dedicated, reusable worker per client first. Add a pool only when concurrency benchmarks establish a need. API:

```ts
export interface RequestOptions { readonly signal?: AbortSignal }
export interface WorkerOptions {
  readonly maxQueued?: number;
  readonly maxInputBytes?: number;
}
export interface AsyncJubarte {
  compare(original: Uint8Array, modified: Uint8Array,
    options: CompareOptions, request?: RequestOptions): Promise<Uint8Array>;
  revisions(docx: Uint8Array, request?: RequestOptions): Promise<readonly Revision[]>;
  accept(docx: Uint8Array, request?: RequestOptions): Promise<Uint8Array>;
  reject(docx: Uint8Array, request?: RequestOptions): Promise<Uint8Array>;
  toPdf(docx: Uint8Array, options?: PdfOptions,
    request?: RequestOptions): Promise<Uint8Array>;
  dispose(): Promise<void>;
}
export function createWorkerClient(options?: WorkerOptions): Promise<AsyncJubarte>;
```

Callers do not accidentally transfer/detach their `Buffer` or pooled backing array. Copy each accepted byte slice synchronously into a fresh `Uint8Array`, preserving `byteOffset` and `byteLength`, before posting/transferring its dedicated `ArrayBuffer`. Reject `SharedArrayBuffer`-backed inputs explicitly in v1; otherwise even snapshot copying can race another thread. The API never mutates input bytes and returns JS-owned output. Document that copying and WASM linear-memory copies are part of the memory budget.

Set an initial conservative queue of 8 waiting requests and compressed input cap of 64 MiB per document as configurable policies, subject to corpus validation. These values are design starting points, not proven universal DOCX limits. Reject excess queued jobs as `QUEUE_FULL` before copying input. Compressed size alone is not a ZIP-bomb limit; Rust must separately bound entry count, expanded bytes, XML depth and image dimensions.

Cancellation contract: queued work is removed/rejected without invocation. For one active worker, cancellation terminates that worker, rejects its active request as `ABORTED`, creates a fresh worker, and then resumes queued work. Every request has an integer ID; unexpected IDs or malformed messages fail the worker instance. Worker crashes reject the active job and create a new worker; jobs not dispatched remain queued. `dispose()` rejects queued/active requests with `CLIENT_CLOSED`, removes listeners and awaits termination. Pre-aborted signals fail before any input copy or worker invocation. `dispose()` is idempotent. Do not claim CPU interruption merely because a Promise rejects; worker termination is the actual interruption mechanism.

```ts
import { createWorkerClient } from "jubarte-wasm/worker";
import { readFile, writeFile } from "node:fs/promises";

const client = await createWorkerClient({ maxQueued: 8 });
try {
  const before = await readFile("original.docx");
  const after = await readFile("revised.docx");
  const redline = await client.compare(before, after, { author: "Legal review" });
  await writeFile("redline.docx", redline);
} finally {
  await client.dispose();
}
```

Node worker_threads is intended for CPU-heavy work and recommends pooling instead of starting a worker for every operation. The primary implementation can reuse one worker while the measured concurrency policy remains small. [Node worker documentation](https://nodejs.org/api/worker_threads.html)

## Error contract

Immediate facade can safely distinguish validation, JSON-protocol, lifecycle, and coarse engine failures. It cannot classify arbitrary engine strings as precise corruption/unsupported-content conditions. Add `JubarteError extends Error` with `code`, `operation`, and `cause`. Initial codes: `INVALID_ARGUMENT`, `ENGINE_ERROR`, `PROTOCOL_ERROR`, `ABORTED`, `QUEUE_FULL`, `CLIENT_CLOSED`, `WORKER_FAILED`. Later shared Rust service codes may include `INVALID_DOCX`, `LIMIT_EXCEEDED`, `EDIT_CONFLICT`, `UNSUPPORTED_EDIT`, `UNSUPPORTED_CAPABILITY`. Define all codes centrally and map typed Rust variants, not substring matching.

Browser/worker messages carry `{ code, operation, message }`, not an `Error` object's accidental enumerable shape. Reconstruct `JubarteError` in the caller. Do not include document contents or full filesystem paths automatically in errors. Legacy `js_err` currently throws a JS string via `JsValue::from_str`; changing that globally would be observable, so introduce new exports or wrap only new facade calls.

## Native Node backend: valuable second lane, not initial prerequisite

Use NAPI-RS 3 with the same Rust service API and Node-API v8 unless a verified required function demands a higher ABI. Native Node addons can maintain ABI compatibility across Node releases when confined to Node-API, but OS/CPU/libc packaging still matters. [Node-API guarantees](https://nodejs.org/api/n-api.html)

Plan native packages for macOS arm64/x64, Linux glibc arm64/x64, Linux musl x64, and Windows x64 only when each has a running integration lane. Do not promise Windows ARM, musl ARM, Bun, Deno or Electron by inference; add them after artifact-level runtime tests. The generated NAPI-RS optional dependency scheme can select a platform artifact without a compiler or postinstall network download. Multi-package publication is non-atomic; publish every tested platform artifact before the root and recover missing packages from identical artifacts rather than rebuilding. Use `npm pack --dry-run --ignore-scripts` for inspection because generated `prepublishOnly` can have publishing side effects. [NAPI-RS release mechanics](https://napi.rs/docs/deep-dive/release)

An async native function must copy JS input on the JS thread before dispatching compute. A held `Buffer` reference prevents collection but does not prevent JS mutation; cross-thread access can be undefined behavior. Return a Rust `Vec<u8>` through the supported NAPI-RS buffer conversion rather than maintaining raw pointers; this may avoid an output copy on supported Node runtimes, but Electron may require a copy. Do not market the entire API as zero-copy. [NAPI-RS typed arrays](https://napi.rs/docs/concepts/typed-array)

CPU work belongs in `Task::compute`/`AsyncTask`, with resolving JS values on the main thread; an `async fn` declaration alone does not prove CPU offloading. Native `AsyncTask` aborts queued work only. Running native compute needs explicit cooperative cancellation in Rust or process isolation; never claim a hard deadline from its AbortSignal. For v1 expose cancellation only on the worker WASM client, and let native promise functions document their non-cancellable active phase. [NAPI-RS AsyncTask](https://napi.rs/docs/concepts/async-task)

Native acceptance gate: parity with WASM/CLI on identical commit/corpus/fonts; clean offline install after package fetch; immutable typed API equality; no input aliasing data races; >=20% material end-to-end improvement on at least one declared production workload with no unacceptable cold-start/install/RSS cost. The 20% threshold is a project decision, not a predicted result. Measure cold load, first call, steady-state throughput, p95/p99 latency and peak RSS; publish raw data and CI hardware context.

## Exact file map

Stage 1 additive facade (keep source hand-written files away from generated glue):

- Create `jubarte-wasm/js/types.ts`: public types only.
- Create `jubarte-wasm/js/errors.ts`: public error class and codes.
- Create `jubarte-wasm/js/revisions.ts`: validate the known JSON contract and return typed immutable-shaped data.
- Create `jubarte-wasm/js/facade.ts`: dependency-injected, synchronous pure wrapper around the raw exports.
- Create `jubarte-wasm/js/node.ts`: import existing Node full glue and export facade operations.
- Create `jubarte-wasm/js/web.ts`: explicit async initialization then facade construction.
- Create `jubarte-wasm/js/slim.ts`: redline-only facade; no PDF stub or runtime surprise.
- Create `jubarte-wasm/js/node-files.ts`: optional asynchronous file read/write convenience kept out of the byte API.
- Create `jubarte-wasm/js/test/fake-engine.ts`: one deterministic engine fake for wrapper tests; does not pretend to test Rust.
- Create `jubarte-wasm/js/test/facade.test.ts`: call forwarding, option defaults, byte ownership and return contracts.
- Create `jubarte-wasm/js/test/revisions.test.ts`: exact current schema, escaping, null fields, unknown type and malformed output.
- Create `jubarte-wasm/js/test/types.test-d.ts`: options and slim capabilities compile tests.
- Create `jubarte-wasm/package.json`, `jubarte-wasm/package-lock.json`, `jubarte-wasm/tsconfig.json`, `jubarte-wasm/vitest.config.ts`: private build/test tooling; publish package stays `npm/package.json`.
- Modify `jubarte-wasm/npm/package.json`: additive `./api`, `./web-api`, `./slim-api` exports and `api/` in files; do not change existing resolutions.
- Modify `jubarte-wasm/build-npm.sh`: compile TS facade after the existing four builds; keep clean-tree check and engine stamp.
- Modify `jubarte-wasm/npm/README.md`: current full PDF signature and quickstart, optional raw low-level section, options/defaults/errors.
- Modify `jubarte-wasm/npm-smoke.mjs`: use all existing functions and new facade; consume a packed package in integration lane.

Stage 2 worker client:

- Create `jubarte-wasm/js/worker/protocol.ts`: discriminated request/response messages and runtime decoding.
- Create `jubarte-wasm/js/worker/client.ts`: bounded queue/lifecycle and request correlation; injected WorkerPort and worker factory.
- Create `jubarte-wasm/js/worker/node-entry.ts`: Node parentPort worker and raw Node WASM import.
- Create `jubarte-wasm/js/worker/web-entry.ts`: browser worker and explicit web WASM init.
- Create `jubarte-wasm/js/worker/node.ts`, `jubarte-wasm/js/worker/web.ts`: platform factories only.
- Create `jubarte-wasm/js/test/fake-worker.ts`: in-memory controllable WorkerPort fake.
- Create `jubarte-wasm/js/test/worker.test.ts`: deterministic queue, lifecycle, cancellation and crashes.
- Create `jubarte-wasm/js/integration/worker.test.ts`: real worker smoke, no unit-test timing assumptions.
- Modify `jubarte-wasm/npm/package.json`: `./worker` and `./web-worker` explicit entrypoints.

Stage 3 shared boundary enrichment:

- Modify `jubarte-wasm/src/lib.rs`: additive typed service/options/error/report exports; leave original exports intact.
- Create `jubarte-wasm/src/service.rs`: Rust-only adapter DTO conversion to shared core service; avoid serializing internal DOMs.
- Modify `jubarte-wasm/Cargo.toml`: serde/serde-wasm-bindgen only if chosen service representation requires them; no JS-only editing logic.
- Modify new TS public types/facade/error modules with compare date/presets, inspection/edit protocol and font report after Rust tests pass.

Stage 4 native backend (conditional):

- Create `jubarte-node/Cargo.toml`, `jubarte-node/build.rs`, `jubarte-node/src/lib.rs`, `jubarte-node/src/tasks.rs`, `jubarte-node/src/error.rs`.
- Create `jubarte-node/package.json`, generated platform package metadata via NAPI-RS, and `jubarte-node/tests/parity.test.ts`.
- The crate depends on `jubarte-redlines = { path = "..", default-features = false }` and uses the shared service. Keep separate workspace conventions initially rather than rewriting all Cargo workspaces.
- Create `.github/workflows/node-native.yml`; runtime-test artifact on its actual advertised platform.

Release/docs:

- Create `.github/workflows/npm.yml`: packed WASM/facade consumer tests plus explicit release job.
- Create `scripts/verify-npm-package.mjs`: tarball inventory, engine commit/version/license and consumer checks.
- Create `docs/api/javascript.md`: compiled usage guide with file-vs-bytes and runtime support matrix.
- Create `docs/api/compatibility.md`: documented extension policy, supported old exports, capabilities and release contract.
- Modify `scripts/bump-version.mjs`, `VERSIONING.md` as needed to include all package versions/stamps.
- Update stale ownership/branding documentation after reconciling authoritative files.

## Preservation and verification design

Run local Cargo commands sequentially from canonical root in default target, and let every package workflow build from a clean source commit. No source behavior fix in generated JS/WASM or copied benchmark binary.

1. Deterministic TS unit tests use pure facade functions and the single owned fake engine/worker interfaces. No actual filesystem/network/clocks/workers in unit tests. Use Vitest V8 coverage; line >=90%, branch >=85% for option/error/queue correctness. Every test invocation has coverage and prints `Coverage: X% lines, Y% branches`. These are proposed gates, not achieved results.
2. Integration suites exercise the real packed wasm npm tarball with Node 22 and 24 LTS, ESM and CommonJS old entrypoints, TypeScript NodeNext and bundler resolution, full/slim export inventories, and offline invocation. Check current support horizon again at implementation time; retain existing Node 18 compatibility only if intentionally tested/documented rather than casually rewriting `engines`.
3. Browser integration uses real Chromium and Firefox, Vite asset loading, explicit WASM URL override, a strict offline smoke after dependencies are installed, full/slim layouts and worker startup. Use the required `vite-plugin-console-pipe` to capture console errors in terminal/log output. Do not assert that every bundler copies `.wasm` automatically: prove supported bundler recipes.
4. Cross-language semantic DOCX parity compares sorted ZIP entry names, relationship targets/content types and normalized XML with narrowly documented volatile-field exclusions; hash untouched binary parts exactly. Compare revisions text/type/part/moves/format changes and accepted/rejected logical content. Do not treat ZIP byte equality, same length, or revision counts as sufficient.
5. Same font inventory and options for native/WASM PDF oracle tests; assert page count, extracted text, layout/pixel score thresholds and report differences. Font loading can vary by platform; rendering identity is not implied by redline identity.
6. Preserve every known fixture area: tracked deletions/insertions, moves, formatting, tables, comments, footnotes/endnotes, images, relationships, styles, numbering, strict OOXML, Unicode and sections. For new edits add stale-snapshot and unsupported-selector/no-mutation checks to shared Rust core.
7. Worker tests cover pre-aborted requests, abort queued request, terminate active worker, ensure other queued jobs complete, full queue rejection before input copies, job error without cross-job contamination, malformed messages, worker crash, initialization rejection, repeated dispose and post-dispose rejection. Integration smoke verifies no live worker keeps an otherwise-completed Node process running after dispose.
8. npm artifact tests install the exact `.tgz` with lifecycle scripts disabled; no Rust compiler or network download after install. Build content hash and engine commit go into a release manifest. For native matrix, ensure every advertised optional platform package exists and has exactly the expected artifact before publishing root.
9. Continue canonical Rust fmt, clippy `-D warnings`, relevant coverage-enabled tests and CLI `--help` smoke. Native/WASM benchmark `script_redlines` scores must agree per fixture before performance publication.

## Trusted publishing and release operations

Use npm OIDC trusted publishing on a supported GitHub-hosted runner, bound to `jandira-tech/jubarte-redlines` and the exact checked-in workflow filename. Configure `id-token: write` only on publish; build/test jobs remain read-only. Current documented minimum is npm 11.5.1 and Node 22.14.0; pin a tested Node 24/npm 11 release toolchain. npm generates provenance automatically for supported public-source publishing. Repository metadata must match source. Configure every new native platform package separately; owning the root name does not establish all suffix names. Do not infer namespace availability from this source tree. [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/)

Release dependency graph: version/clean commit validation -> Rust fidelity/tests -> four WASM builds -> TS build/unit coverage -> pack -> packed-package Node/browser/worker tests -> source/license/manifest checks -> OIDC publish -> clean installed-registry smoke. Keep exact immutable artifacts for retry. Avoid running auto-publishing npm lifecycle hooks during preview. A release rollback uses a new version or deprecation; it does not overwrite a published tarball.

No npm publish, GitHub push, package rename, license rewrite, or telemetry activation is required to prepare this plan.
