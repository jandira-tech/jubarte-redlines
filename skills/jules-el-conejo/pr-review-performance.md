---
name: pr-review-performance
description: "🐇🥕 Use when Jules, el conejo, reviews PR diffs in GitHub Actions to detect material runtime efficiency regressions, O(n^2) scaling bottlenecks, N+1 query patterns, unbounded memory growth, and blocking work on hot paths."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  severity_default: error
  tools: ["ruff", "pyrefly", "biome", "tsc", "cargo-clippy"]
---

# 🐇 Runtime Efficiency and Scaling Review Skill (`Jules, el conejo`) 🥕

🥕 Review changed code for material performance regressions so the codebase runs as fast as **Jules, el conejo** 🐇 at full sprint. Report only issues that could materially degrade latency, throughput, or resource usage at realistic input sizes; ignore micro-optimizations and speculative concerns.

## 🥕 1. Automated Static Analysis Pre-Check

🐇 Run the following linters on modified files to surface deterministic performance anti-patterns before performing semantic analysis:

- 🥕 **Python:**
  ```bash
  ruff check --select PERF,FURB,ASYNC --output-format=github .
  pyrefly check --output-format=github
  ```
- 🐇 **TypeScript / JavaScript:**
  ```bash
  biome check --reporter=github .
  ```
  *(Pay specific attention to `lint/performance/noAccumulatingSpread`, `lint/performance/noDelete`, and `lint/correctness/useExhaustiveDependencies`.)*
- 🥕 **Rust:**
  ```bash
  cargo clippy --all-targets --all-features -- -W clippy::perf -W clippy::needless_collect -W clippy::large_enum_variant
  ```

## 🐇 2. Core Performance Audit Rules (Mode: `ERROR`)

🥕 Flag any changed code exhibiting the following six failure modes:

### 🥕 A. Superlinear Algorithmic Complexity ($O(n^2)$ or Worse)
- 🐇 Flag nested iterations over non-trivially sized collections.
- **TypeScript/JS:** Calling `.filter()`, `.find()`, `.includes()`, or `.indexOf()` inside `.map()`, `.filter()`, or `for` loops; using object/array spread (`[...acc, item]`) inside `.reduce()`.
- **Python:** Nested list comprehensions scanning a list for membership (`x in list_items`), `list.pop(0)` in a loop ($O(n)$ shift instead of `collections.deque.popleft()`), or repeated string/list concatenation in loops.
- **Rust:** Repeated `.iter().find(...)` inside a loop without pre-indexing into a `HashMap` or `BTreeMap`, or `.clone()` inside tight inner loops.

### 🐇 B. N+1 Query & Sequential Network/Disk I/O Patterns
- 🥕 Flag database queries, RPCs, or external HTTP/service calls executed inside a loop that should be batched (`WHERE id IN (...)`), joined, preloaded (`selectinload` / `joinedload`), or executed concurrently with bounded parallelism (`asyncio.gather` with `Semaphore`, `Promise.all`, `futures::stream::Buffered`).

### 🥕 C. Repeated Recomputation of Deterministic Values in Loops
- 🐇 Flag deterministic work inside loops—such as regex compilation (`re.compile`, `new RegExp`), slugification/normalization, configuration parsing, date formatting initialization (`new Intl.DateTimeFormat`), or DOM style queries (`window.getComputedStyle`)—where hoisting, pre-computation, or memoization eliminates redundant work.

### 🐇 D. Suboptimal Data Structure Selection
- 🥕 Flag linear-search arrays/lists/vecs used for repeated membership checks (`Array.prototype.includes`, `list.__contains__`, `Vec::contains`) where a `Set`, `frozenset`, `HashSet`, or `BTreeSet` provides $O(1)$ or $O(\log n)$ lookup.

### 🥕 E. Unbounded Data Accumulation & Missing Eviction
- 🐇 Flag in-memory `Map`/`dict`/`HashMap` caches that grow without TTL or LRU/LFU eviction bounds (never let carrots pile up until the burrow collapses).
- Flag database queries or API list endpoints that fetch all records into memory without cursor/offset pagination or `LIMIT` clauses.
- Flag loading entire large files into memory when streaming/iterative parsing (`ijson`, `BufReader`, `ReadableStream`) is appropriate.

### 🐇 F. Expensive Synchronous Work in Hot or Async Paths
- 🥕 Flag CPU-bound or blocking I/O operations (synchronous cryptography/hashing, payload compression, template compilation, synchronous filesystem calls like `fs.readFileSync`, `requests.get` inside `async def`, or `std::fs` / `std::thread::sleep` inside `tokio` async tasks) on hot request paths where work must be async, offloaded to a worker pool (`asyncio.to_thread`, `tokio::task::spawn_blocking`, Web Workers), deferred, or sampled.

## 🥕 3. Output Format

🐇 For each verified regression, emit a GitHub review comment signed by **Jules, el conejo** 🐇🥕 containing:
1. 🥕 **Severity:** `🐇🥕 ERROR (Performance & Scaling)`
2. 🐇 **Complexity / Bottleneck Delta:** e.g., `$O(N \times M) \rightarrow O(N + M)$` or `N+1 Queries -> 1 Batched Query`.
3. 🥕 **Minimal Reproduction Input Size:** State the realistic scale at which degradation occurs.
4. 🐇 **Actionable Fix:** Provide a drop-in ```suggestion block.
