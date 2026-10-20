---
name: pr-review-architecture-quality
description: "🐇🥕 Use when Jules, el conejo, reviews PRs in GitHub Actions to enforce Single Responsibility, public API call-site consistency, direct module imports (no barrel files), failure-path error robustness, and Conventional Commits."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  severity_default: warning
  tools: ["pyrefly", "ruff", "biome", "tsc", "cargo-clippy"]
---

# 🐇 Architecture, API Contract, and Error Robustness Skill (`Jules, el conejo`) 🥕

🥕 Audit structural code quality, module burrow boundaries, error handling, and commit hygiene across Python, TypeScript/JavaScript, and Rust with the sharp ears of **Jules, el conejo** 🐇.

## 🥕 1. Automated Static Analysis Gate

🐇 Run these deterministic checks in CI to enforce import structure, type contracts, and error handling:

- 🥕 **TypeScript / JavaScript:**
  ```bash
  biome check --reporter=github .
  tsc --noEmit
  ```
  *(Specifically enforce Biome's `lint/performance/noBarrelFile` and `lint/performance/noReExportAll`.)*
- 🐇 **Python:**
  ```bash
  pyrefly check --output-format=github
  ruff check --select BLE,TRY,PLR,RET,SIM,TID --output-format=github .
  ```
  *(Specifically enforce `PLR0913` for too-many-arguments, `BLE001` for blind excepts, and `TRY` for exception handling.)*
- 🥕 **Rust:**
  ```bash
  cargo clippy --all-targets --all-features -- -W clippy::too_many_arguments -W clippy::cognitive_complexity -W clippy::let_underscore_must_use
  ```

## 🐇 2. Structural & Quality Checks

### 🥕 Check 1: Module Cohesion and Naming Clarity (Mode: `WARNING`)
🐇 Review packages, modules, classes/structs, and functions for adherence to the Single Responsibility Principle (SRP)—every burrow in the warren has one clear purpose:
- Flag modules or packages that accumulate unrelated domain concerns.
- **Field Threshold:** Flag classes, dataclasses, interfaces, or Rust structs with more than **~7 top-level fields** as candidates for decomposition into cohesive sub-structures.
- **Abstraction Mixing:** Flag functions that mix high-level orchestration policy with low-level I/O or byte-manipulation details.
- **Ban Context-Free Names:** Flag generic, vague identifiers such as `Manager`, `Handler`, `Util`, `Helper`, `Process`, or `Do`; require names that state a single specific domain concept.
- **Parameter Threshold:** Flag functions or methods with more than **4–5 parameters** (`PLR0913` / `clippy::too_many_arguments`); recommend a typed parameter object/struct or builder.

### 🐇 Check 2: Public API Contract Consistency (Mode: `WARNING`)
🥕 Review any changes to exported or public method, function, class, or trait signatures:
- Flag additions, removals, or reordering of parameters; type changes (including adding/removing optionality `Optional[T]`, `T | undefined`, `Option<T>`); return type changes; and visibility reductions (`pub` -> `pub(crate)` or private).
- Verify via `pyrefly`, `tsc --noEmit`, `cargo check --workspace`, and `git grep` that **all call sites** across the repository have been updated in the same PR.
- Flag file renames or directory moves that alter import paths without updating all consumer imports.

### 🥕 Check 3: Direct Module Import Enforcement — No Barrel Files (Mode: `ERROR`)
🐇 When TypeScript, JavaScript, or Python files are modified, inspect all new import and re-export statements (a conejo hops directly to the source module, never through tangled barrel tunnels):
- **FAIL** if any local import pulls from a barrel re-export file (`index.ts`, `index.js`, or bloated `__init__.py`) that aggregates multiple unrelated modules, components, or hooks.
- Third-party package imports (`node_modules`, PyPI packages, crates) are always allowed.
- For local imports, flag multi-named imports from an aggregating local path and require direct module paths (e.g., `import { Button } from './ui/Button'` instead of `import { Button, Modal, useAuth } from './ui'`).
- **FAIL** if any new `index.ts` or `__init__.py` re-exports symbols from multiple unrelated sibling folders.

### 🐇 Check 4: Failure Path Robustness Review (Mode: `WARNING`)
🥕 Review error handling patterns in new and modified code:
- **No Silent Error Discards:** Flag returned errors or promises silently discarded (`let _ = ...` in Rust without justification, `except Exception: pass` in Python, `.catch(() => {})` or floating promises in TypeScript).
- **No Fragile String Matching on Errors:** Flag error branching based on substring comparison of `error.message` or `str(e)`; require stable error codes, discriminated unions, `instanceof` custom error classes, or Rust error enum variants.
- **Safe Error Property Access:** Flag `error.message` accesses in TypeScript/JavaScript `catch (error: unknown)` blocks without `error instanceof Error` narrowing or optional-chaining protection.
- **Full Context Logging:** Flag `catch` / `except` blocks that log only `str(e)` or `error.message` instead of passing the full error object/stack trace (`logger.exception(...)` / `exc_info=True` in Python, `{ err: error }` in Pino/structured TS loggers).
- **Top-Level Async Entrypoints:** Flag CLI bootstrap scripts, worker entrypoints, or job runners (`main()`) that lack a top-level `try/catch` or `.catch()` logging the fatal error and exiting with a non-zero exit code (`process.exit(1)` / `sys.exit(1)`).

### 🥕 Check 5: Conventional Commits & PR Hygiene (Mode: `WARNING`)
🐇 Review commit messages and PR metadata:
- Enforce [Conventional Commits](https://www.conventionalcommits.org/) format (`feat:`, `fix:`, `refactor:`, `perf:`, `test:`, `docs:`, `ci:`, `chore:`, `build:`) with optional scope (`feat(api): ...`) and breaking change indicator (`!`).
- Flag vague commit subjects (`wip`, `fix bug`, `updates`, `misc`) and single-commit mega-PRs that mix unrelated refactoring with behavioral changes.
- Verify that breaking public API changes include a `BREAKING CHANGE:` footer or migration notes in the PR body.
