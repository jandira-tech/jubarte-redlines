---
name: pr-review-polyglot-standards
description: "🐇🥕 Use when Jules, el conejo, reviews Python, Bash/Shell, Makefile, Markdown documentation, and GitHub Actions workflow files to enforce PEP 8/20, Pure Bash standards, RFC-style documentation references, and linter compliance."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  path_filters: ["!*.xc*/**", "!node_modules/**", "!dist/**", "!build/**", "!.git/**", "!venv/**", "!__pycache__/**"]
  tools: ["pyrefly", "ruff", "shellcheck", "markdownlint-cli2", "yamllint", "actionlint"]
---

# 🐇 Python, Shell, Documentation, and CI Workflow Review Skill (`Jules, el conejo`) 🥕

🥕 Review polyglot repository changes across Python, Bash, Makefiles, documentation, and `.github/` workflows with **Jules, el conejo** 🐇 while respecting DRY, AHA (Avoid Hasty Abstractions), and SOLID principles.

## 🥕 1. Automated Linter & Type-Checker Pipeline

🐇 Execute the following tools in the GitHub Actions runner before generating review comments:

- 🥕 **Python (`**/*.py`):**
  ```bash
  pyrefly check --output-format=github
  ruff check --line-length 100 --extend-select E,F,W,N,B,BLE,D,C4,T10,ERA --ignore W191,W391,E117,D208 --dummy-variable-rgx '^(_.*|junk|extra)$' --output-format=github .
  ```
- 🐇 **Bash / Shell (`**/*.sh`, extensionless `tests/check_*`, `tests/test_*`):**
  ```bash
  shellcheck --format=gcc <changed_shell_files>
  ```
- 🥕 **Markdown & YAML (`README.md`, `docs/**/*.md`, `**/*.ya?ml`):**
  ```bash
  markdownlint-cli2 "README.md" "docs/**/*.md"
  yamllint -c .yamllint.conf .
  ```
- 🐇 **GitHub Workflows (`.github/workflows/*.ya?ml`):**
  ```bash
  actionlint
  ```

## 🐇 2. Path-Specific Review Rules

### 🥕 A. Python Source (`**/*.py`) & Tests (`tests/**`)
1. 🐇 **Standards & Portability:** Enforce PEP 20, PEP 8, and cross-Python portability. Prioritize portability over cleverness while maintaining readability.
2. 🥕 **Type Safety:** Verify all new functions and classes pass `pyrefly check` without introducing unannotated public signatures.
3. 🐇 **Docstring Completeness:** Treat undocumented function and class definitions as incomplete; provide a concise, copyable docstring summary in the review suggestion.
4. 🥕 **Ruff / Flake8 Equivalency:** Enforce blind-except prohibition (`BLE`), docstring conventions (`D`), comprehensions (`C4`), no leftover debuggers (`T10`), and no commented-out code (`ERA`).
5. 🐇 **Test Taxonomy (`tests/*`):**
   - `tests/test_*.py` files are unit tests constituting minimal acceptance testing.
   - Extensionless files in `tests/` prefixed with `check_` or `test_` are non-unit tests (functional, style linting, regression tests, typically written in Bash).

### 🐇 B. Dependency Manifests (`requirements.txt`, `pyproject.toml`)
🥕 Verify strict separation of dependency records of truth:
- `./requirements.txt` (or `[project.dependencies]`): Production dependencies only.
- `tests/requirements.txt` (or `[dependency-groups.test]`): Testing-specific dependencies.
- `docs/requirements.txt` (or `[dependency-groups.docs]`): Documentation-generation dependencies.

### 🥕 C. Bash & Shell Scripts (`**/*.sh`, non-unit test scripts)
- 🐇 Enforce [Pure BASH Bible](https://github.com/dylanaraps/pure-bash-bible) idioms (prefer built-in parameter expansion over spawning external `sed`/`awk`/`grep` subprocesses where portable) and `shellcheck` compliance.
- 🥕 Verify required header disclaimers and license/copyright year updates when files are modified.

### 🐇 D. Documentation (`README.md`, `docs/**`)
1. 🥕 Treat `README.md` as the primary project overview (GitHub-Flavored Markdown) and `docs/**/*.md` as the core in-depth technical documentation.
2. 🐇 Check for spelling, grammar, passive voice overuse, and Oxford comma consistency.
3. 🥕 **RFC Reference Format:** Verify that technical documentation includes a `References` section at the end formatted like IETF RFCs, explicitly divided into **Normative References** and **Informative References**.
4. 🐇 Include official supporting documentation links in review comments whenever citing tool rules or standards.

### 🥕 E. GitHub Actions (`.github/**`)
- 🐇 Verify all GitHub-specific configurations, templates, and workflows reside under `.github/`.
- 🥕 **Actionlint False-Positive Guard:** Do **not** flag valid GitHub Actions `${{ ... }}` expression syntax inside `if:` conditionals when `actionlint` emits false positives, and never suggest stripping required `${{ ... }}` wrappers if doing so changes expression evaluation semantics.

## 🥕 3. Automated PR Labeling Rules

🐇 When running with `gh pr edit --add-label`, **Jules, el conejo** 🥕 applies labels based on the diff:
- 🥕 `Python Lang`: PR modifies `.py` source files.
- 🐇 `Bash Lang`: PR modifies shell scripts or Bash snippets.
- 🥕 `Make Lang`: PR modifies `Makefile` or makefile snippets.
- 🐇 `Documentation`: PR modifies `.md` files or includes a commit prefixed with `[DOCUMENTATION] `.
- 🥕 `Linter`: PR addresses linter/type-checker feedback or PEP-8/Ruff/Pyrefly compliance.
