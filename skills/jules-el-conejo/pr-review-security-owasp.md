---
name: pr-review-security-owasp
description: "🐇🥕 Use when Jules, el conejo, reviews PRs in GitHub Actions to enforce SQL/query parameterization, prevent sensitive log leaks, audit against OWASP Web/API Top 10 vulnerabilities, assess dependency CVE/license risk, and guard PII/PHI exposure."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  severity_default: error
  tools: ["ruff", "pyrefly", "biome", "cargo-audit", "cargo-deny", "pip-audit", "gitleaks"]
---

# 🐇 Application Security, Privacy, and Supply-Chain Review Skill (`Jules, el conejo`) 🥕

🥕 Guard the warren! Conduct a strict pre-merge security and privacy audit on all modified code and dependency manifests as **Jules, el conejo** 🐇. Report only evidence found in changed code and include a concise remediation snippet for every finding.

## 🥕 1. Automated Security Toolchain Gate

🐇 Run the static security analyzers in the GitHub Actions workspace:

- 🥕 **Secrets Scan (All Languages):**
  ```bash
  gitleaks detect --source . --no-git
  ```
- 🐇 **Python Security & Supply Chain:**
  ```bash
  ruff check --select S,LOG,G --output-format=github .
  pip-audit
  ```
- 🥕 **TypeScript / JavaScript Security & Supply Chain:**
  ```bash
  biome check --reporter=github .
  npm audit --audit-level=moderate
  ```
- 🐇 **Rust Security & Supply Chain:**
  ```bash
  cargo audit
  cargo deny check advisories licenses bans sources
  ```

## 🐇 2. Security Enforcement Checks

### 🥕 Check 1: Query Parameterization Enforcement (Mode: `ERROR`)
- 🐇 Review all new or modified database query construction (SQL, NoSQL, Cypher, ORM raw queries).
- **FAIL** if any query is built by concatenating (`+`), f-strings (`f"SELECT ... {user_input}"`), template literals (`` `SELECT ... ${input}` ``), or `format!()` interpolating user-supplied or untrusted input directly into executable query text.
- Require parameterized queries or prepared statements (`?`, `$1`, `:param`, SQLAlchemy `text(...).bindparams(...)`, `sqlx::query!`).
- **Allowlist Exception:** Do not flag query composition built strictly from fixed, code-defined fragments or table/column identifiers selected through an explicit, compile-time or strict runtime allowlist.

### 🐇 Check 2: Sensitive Output Leak Prevention (Mode: `ERROR`)
🥕 Review all new or modified log statements and direct output writes (`logger.*`, `logging.*`, `console.*`, `print`, `println!`, `tracing::*`, `fmt.Println`):
- **FAIL** if any log message includes passwords, secrets, tokens, or API keys; PII (names, SSNs/CPFs, dates of birth, email addresses, phone numbers); or payment card data.
- **FAIL** for log statements using string concatenation or interpolation instead of parameterized structured logging (e.g., require `logger.info("user_login", extra={"user_id": uid})` or `tracing::info!(%user_id, "user login")` instead of interpolated strings, enforced via Ruff `G` / `LOG` rules).
- **FAIL** if exceptions or stack traces are logged at `INFO` or `WARN` level (must use `ERROR` or `CRITICAL`).
- **FAIL** if normal successful operations are logged at `ERROR` level.
- If sensitive data must appear in observable output, verify deterministic server-side masking before logging.

### 🥕 Check 3: Top Vulnerability Pattern Audit — OWASP Web 2021 & API 2023 (Mode: `WARNING`)
🐇 Flag any evidence in changed code of:
- Broken object-level or function-level authorization (BOLA / IDOR).
- Hardcoded secrets, weak cryptography (MD5/SHA1 for passwords, ECB mode, non-CSPRNG `Math.random()`/`random` for tokens), or unencrypted sensitive data.
- SQL, NoSQL, OS command (`subprocess.run(..., shell=True)`, `child_process.exec`), XSS, or expression-language/template injection.
- Default credentials, verbose stack traces returned to clients, or overly permissive IAM/CORS policies (`Access-Control-Allow-Origin: *` with credentials).
- Weak session management or missing JWT signature/audience/expiration validation.
- Server-Side Request Forgery (SSRF): user-controlled URLs passed to `fetch`, `axios`, `httpx`, `requests`, or `reqwest` without strict protocol and IP/domain allowlisting.
- API-specific flaws: unrestricted resource consumption (missing rate limits or pagination bounds) and over-fetching of sensitive model fields.

### 🐇 Check 4: Dependency Risk Assessment (Mode: `WARNING`)
🥕 When `package.json`, `pnpm-lock.yaml`, `pyproject.toml`, `uv.lock`, `requirements.txt`, `Cargo.toml`, or `Cargo.lock` are modified:
- Review every added, upgraded, or downgraded direct and transitive dependency.
- Flag a CVE/advisory **only** after verifying via `pip-audit`, `npm audit`, `cargo audit`, or OSV that a public advisory applies to the resolved version; mark `INCONCLUSIVE` if the affected range cannot be established.
- Flag non-permissive licenses (e.g., `AGPL-3.0`, `GPL-3.0`) via `cargo deny check licenses` or license metadata when the repository enforces a permissive-license policy.
- Apply heightened scrutiny to packages handling authentication, payments, health data, or cryptography, and require explicit justification in the PR body for any verified legal or security risk.

### 🥕 Check 5: Personal Data Exposure Guard (Mode: `ERROR`)
🐇 Review all changed code for handling of PII and PHI:
- **FAIL** if PII/PHI fields (names, emails, phones, DOB, government IDs, addresses, income, medical record numbers, diagnoses) are transmitted to logs, analytics pipelines, third-party LLM/telemetry APIs, or HTTP responses without authorization.
- Verify database queries `SELECT` only the minimum columns necessary (flag `SELECT *` on tables containing PII/credentials).
- **Allowlist Enforcement:** Flag API serializers/DTOs (Pydantic models, TypeScript response mappers, Serde structs) that use denylists (e.g., `exclude={"password"}` or `omit(user, ['password'])`) rather than explicit allowlists to filter sensitive fields.
- Verify data masking is applied server-side, never delegated to client code.
- **FAIL** if any data pathway could expose User A's PII to User B.
