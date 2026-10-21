---
name: pr-review-static-triage
description: "🐇🥕 Use when Jules, el conejo, runs autonomous PR triage or repository sweeps in GitHub Actions to detect high-confidence defects across security, async I/O, loop/cache optimization, type safety, and test coverage, and emit prioritized structured findings."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  categories: ["security", "performance", "type-safety", "code-hygiene", "test-coverage"]
  tools: ["pyrefly", "ruff", "biome", "tsc", "cargo-clippy"]
---

# 🐇 Multi-Language Static & Semantic Triage Skill (`Jules, el conejo`) 🥕

🥕 Synthesizes empirical defect patterns across TypeScript/React/Astro, Python, and Rust into a high-precision triage engine for **Jules, el conejo** 🐇, calibrated by **Confidence (1–3)** and **Priority (1–3)**.

## 🥕 1. Deterministic Toolchain Execution

🐇 Run the relevant static analyzers first and map their diagnostics directly into Confidence-1 findings:

- 🥕 **Python (`pyrefly` + `ruff`):**
  ```bash
  pyrefly check --output-format=github
  ruff check --select E,F,B,S,PERF,ASYNC,ERA,TID,UP,RUF --output-format=github .
  ```
- 🐇 **TypeScript / Astro / React (`biome` + `tsc`):**
  ```bash
  biome ci --reporter=github .
  tsc --noEmit
  ```
- 🥕 **Rust (`cargo clippy`):**
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

## 🐇 2. High-Signal Pattern Catalog (Grounded in Empirical Triage)

🥕 Scan the PR diff for the following five defect categories:

### 🥕 Category A: Security & Configuration (`secrets`, `injection`, `input-validation`, `insecure-config`, `authentication`)
1. 🐇 **Client-Side Secret Exposure (Priority 1):** Flag API keys or secrets injected into frontend bundles via bundler config (`vite.config.ts` `define: { 'process.env.API_KEY': ... }`) or instantiated directly in client-side components.
2. 🥕 **DOM XSS & Unsanitized HTML (Priority 1–2):** Flag direct assignment of untrusted strings to `element.innerHTML` (require `DOMParser` or `DOMPurify`) and audit `dangerouslySetInnerHTML` even when wrapped in custom stringifiers.
3. 🐇 **SSRF & Incomplete IP Validation (Priority 1):** Flag hand-rolled `isPrivateOrLocalHost` hostname split checks that miss decimal/octal/hex IP encodings (e.g., `2130706433`) or IPv6-mapped loopback addresses.
4. 🥕 **Path Traversal in File/Folder Normalization (Priority 2):** Flag filename/path sanitizers that allow `.` characters without rejecting `..` traversal segments before `os.path.join` / `path.join`.
5. 🐇 **Disabled TLS Verification (Priority 2):** Flag `verify=False` in Python HTTP clients (`requests`, `httpx`, `WebBaseLoader`) or `NODE_TLS_REJECT_UNAUTHORIZED=0`.
6. 🥕 **Placeholder Auth & Unauthenticated Sockets (Priority 1–2):** Flag fallback tokens (`getattr(payload, "token", "dummy_token")`), stubbed `isAdmin()` checks, or WebSocket upgrade handlers lacking authentication.
7. 🐇 **Web & Markup Hardening (Priority 2–3):** Flag `<form>` tags missing explicit `action` and `method="POST"` attributes, `<a target="_blank">` missing `rel="noopener noreferrer"`, external CDN `<script>` tags missing Subresource Integrity (`integrity`), open redirects via unvalidated `window.location.href = item.href`, ReDoS-prone dynamic `new RegExp(...)` alternations, and `Math.random()` used for IDs instead of `crypto.randomUUID()`.

### 🐇 Category B: Performance, Async I/O & Caching (`n+1-query`, `async-io`, `loop-optimization`, `caching`, `data-structure`)
1. 🥕 **Sequential `await` in Loops (Priority 1–2):** Flag sequential `await` calls inside `for` loops for independent I/O (e.g., IMAP email processing, external case API lookups, attachment reads, or SQL migration checks in loops). Recommend `asyncio.gather` (Python), `Promise.all` (TS), or `WHERE ... IN (...)` batching.
2. 🐇 **Blocking Sync I/O Inside `async` Functions (Priority 2):** Flag `fs.readFileSync` inside `async function` (replace with `await fs.promises.readFile`), blocking HTTP/search calls in Python `async def` without `asyncio.to_thread`, or synchronous file parsing of entire JSONL files when single-record streaming (`ijson`) suffices.
3. 🥕 **DOM Layout Thrashing in Observers/Effects (Priority 1–2):** Flag synchronous `window.getComputedStyle(el)` or `document.querySelectorAll(...)` inside `MutationObserver`, `ResizeObserver`, or `IntersectionObserver` callbacks without caching or declarative data attributes.
4. 🐇 **Redundant Collection Reads & Missing Memoization (Priority 1–2):** Flag repeated un-cached calls to content collection loaders (`getCollection`, CMS readers) or expensive regex normalization inside build/render loops.
5. 🥕 **Loop & Allocation Anti-Patterns (Priority 2–3):** Flag $O(N \times M)$ nested `.map(... => .filter(...))` grouping (use a single-pass `Map`), repeated `slugify(tag)` inside `.filter()`, allocating array literals (`['ul', 'ol'].includes(...)`) or `Object.keys()`/`Object.entries()` inside tight parser loops, static constants recreated inside React components on every render, and Python `list.pop(0)` ($O(N)$) instead of `deque.popleft()` or `.pop()`.

### 🥕 Category C: Type Safety & Static Verification (`complexity`, `other`)
1. 🐇 **Explicit `any` & Unsafe Casts (Priority 2–3):** Flag `any` in component props, DOM event helpers, `useRef<any>`, or dynamic dictionary indexing (`(Icons as any)[name]` -> use `keyof typeof Icons`).
2. 🥕 **Linter & Type Suppressor Abuse (Priority 2):** Flag `@ts-ignore` (require proper typing or `@ts-expect-error` with description), blanket `/* biome-ignore-all */`, `biome-ignore lint/a11y/...`, or `# type: ignore` without error codes.
3. 🐇 **Inferred `{}` Options Objects (Priority 1):** Flag default parameter objects `options = {}` lacking an explicit interface/type annotation (caught by `tsc` TS2339 and `pyrefly`).

### 🐇 Category D: Code Hygiene & Maintainability (`duplication`, `dead-code`, `deprecated`, `todo-fixme`, `unused-import`)
1. 🥕 **Cross-Package / Cross-Locale Duplication (Priority 1–2):** Flag duplicated components, hooks (`useContrastColor`, `useMenuContrast`), legal/privacy pages, or inline `<script>` blocks across monorepo apps or locale routes.
2. 🐇 **Swallowed Errors & Leftover Debug Logs (Priority 1–3):** Flag empty `.catch(() => {})` on worker/async initialization, leftover `console.log`/`console.error` in auth flows, commented-out code blocks, orphaned `@deprecated` files/constants, and hardcoded local developer paths (`/Users/...`).

### 🥕 Category E: Test Coverage Gaps (`untested-function`, `missing-test-file`, `missing-edge-case`, `missing-error-test`)
- 🐇 Prioritize Confidence-1 pure functions (pagination math, URL builders, slugifiers, JSON escapers, contrast calculators, binary/XML header parsers) and critical entrypoints (`compareDocx`, `redlineDocx`, `ErrorBoundary`) lacking unit, edge-case, or error-path tests.

## 🥕 3. Structured JSON / Markdown Output Schema

🐇 When **Jules, el conejo** 🥕 runs in batch triage mode or PR review mode, score each finding by:
- 🥕 `confidence`: `1` (Deterministic / Self-contained fix < 20 LOC), `2` (Moderate context / test fixture required), `3` (Architectural refactor > 50 LOC).
- 🐇 `priority`: `1` (Critical security, N+1 bottleneck, core API untested), `2` (Moderate defect/hygiene), `3` (Minor polish).
