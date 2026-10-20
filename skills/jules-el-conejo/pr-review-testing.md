---
name: pr-review-testing
description: "🐇🥕 Use when Jules, el conejo, reviews PRs in GitHub Actions to verify unit and regression test adequacy for new logic, enforce test isolation and hygiene, and require empirical validation evidence for impactful changes."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  severity_default: warning
  tools: ["pytest", "vitest", "cargo-nextest", "pyrefly", "tsc", "biome", "ruff"]
---

# 🐇 Test Adequacy, Hygiene, and Validation Evidence Skill (`Jules, el conejo`) 🥕

🥕 Evaluate both production code changes (for missing coverage) and test code changes (for structural quality and burrow isolation) as **Jules, el conejo** 🐇.

## 🥕 1. Automated Test & Type Verification Gate

🐇 Execute the project's test and type-checking suite in CI to establish baseline coverage and catch broken test fixtures:

- 🥕 **Python:**
  ```bash
  pyrefly check --output-format=github
  # 🥕 Use the repository's own test and source roots (pyproject.toml, setup.cfg).
  ruff check . --select PT,B,ASYNC --output-format=github
  pytest --cov --cov-report=term-missing --cov-branch
  ```
- 🐇 **TypeScript / JavaScript:**
  ```bash
  biome check --reporter=github
  tsc --noEmit
  vitest run --coverage
  ```
- 🥕 **Rust:**
  ```bash
  cargo clippy --tests -- -D warnings
  cargo test --all-targets
  ```

## 🐇 2. Review Checks (Mode: `WARNING`)

### 🥕 Check 1: New Logic Test Adequacy
🐇 Review the PR for new or modified source code that lacks corresponding test coverage:
- **Pure Functions & Utilities:** Flag new functions, methods, and classes/structs that have no unit tests—especially pure functions where deterministic testing requires zero mocks.
- **Bug Fixes:** Flag any bug fix PR that lacks a dedicated regression test which fails without the fix and passes with it.
- **Error-Throwing Paths:** Flag new exception/error-raising paths (`raise`, `throw new ...`, `Err(...)` / `?`) that have no test asserting the exact error type/variant is raised (`pytest.raises`, `expect(...).toThrowError`, `assert!(matches!(res, Err(...)))`).
- **Stale Test Factories & Fixtures:** Flag test factories, builders, or fixtures that were not updated when the corresponding domain entity or Pydantic/TypeScript/Rust model gained new required fields (verified via `pyrefly`, `tsc`, and `cargo check --tests`).
- **Exclusions (Do Not Flag):** Do not flag trivial changes (renaming, formatting, comments), auto-generated code (e.g., `worker-configuration.d.ts`), or refactors already fully covered by existing tests. Pass when new logic has proportionate coverage across happy paths, error paths, and edge cases (empty inputs, `null`/`None`, boundary numbers).

### 🐇 Check 2: Test Hygiene and Isolation
🥕 Review new and modified test files for structural quality (every testconejo stays in its own clean burrow):
- **Single-Behavior Focus:** Flag kitchen-sink tests that assert multiple unrelated behaviors in a single test case; each test must validate one specific behavior.
- **Resource Teardown & Global State:** Flag tests that mutate global state (e.g., `global.fetch`, `os.environ`, singleton caches, temp directories, DB connections) or allocate resources without deterministic teardown (`try/finally`, `afterEach`, `yield` pytest fixtures, `Drop` / `tempfile::TempDir`).
- **Bounded Async Operations:** Flag async tests or external/worker interactions that lack explicit timeout bounds.
- **Self-Explanatory Assertions:** Flag bare assertions (`assert x`, `expect(x).toBe(true)`, `assert!(x)`) where a failure produces an opaque message; require descriptive failure messages or structural equality matchers so failures are self-explanatory without reading source code.
- **Anti-Duplication Across Test Layers:** Flag high-level integration/E2E tests that re-implement low-level algorithmic assertions already covered by unit tests; high-level tests should verify wiring, calls, and outcomes.

### 🥕 Check 3: Validation Evidence for Impactful PRs
🐇 When a PR introduces significant changes (new features, breaking changes, substantial refactoring, or algorithmic modifications), verify that the PR description includes empirical carrot-solid evidence:
- **Numerical Accuracy / Convergence Changes:** Require before-and-after evaluation outputs demonstrating zero regression.
- **Performance-Affecting Changes:** Require before-and-after benchmark numbers (`pytest-benchmark`, `vitest bench`, `criterion` / `divan`) along with the hardware/runtime configuration and dataset context in which they were measured.
- Flag when material changes are introduced with no validation evidence in the PR description.
