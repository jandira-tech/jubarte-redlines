<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Review rules for jubarte-redlines

jubarte reads, edits, compares and renders Word documents, in Rust, with
Python (PyO3) and WASM/npm bindings. AGENTS.md and CONTRIBUTING.md are the
house rules; where they and this file disagree with generic advice, they win.

Review every changed hunk under all five headings below. Report a finding
with its file and line, why it is wrong, and the fix. Do not report code the
pull request does not change. Static tools already ran (the "Jules, el
conejo" report on the PR lists clippy, cargo fmt, cargo deny, ruff,
shellcheck, zizmor and actionlint); do not repeat what they say, and look
for what they cannot see.

## 1. Correctness and Word fidelity

- Microsoft Word is the reference. A change that alters the docx, PDF or
  Markdown output must say which Word behaviour it matches and how that was
  established (a Word probe, a fixture, Word's own PDF).
- Never make jubarte refuse, strip or rewrite an input that Word opens:
  that is a regression, not hardening. A ZIP part name is an OPC URI read in
  memory and never written to disk, so file-system rules (Windows reserved
  names such as CON or AUX, path length) do not apply to it.
- Every docx jubarte writes must open in Word without a repair prompt.
  A new Word-invalid shape needs a check in `src/validate.rs` (`ring1`) and a
  broken probe in `tests/m_validity_ring1.rs`.
- Headers, footers, footnotes, endnotes and comments count as much as the
  body; check that a change reaches every part it should.
- Hierarchy depth comes from the document's real nesting, never from a
  marker's kind or the numbering string.
- Look for off-by-one, empty input, a single element, Unicode (combining
  marks, surrogate pairs, right-to-left text), and tracked changes inside
  tables, fields and content controls.

## 2. Security

- Untrusted input is every byte of a docx: ZIP structure, XML, relationship
  targets, embedded objects. Check bounds on sizes, counts and nesting
  depth, and that no input can panic, hang or allocate without limit.
- No `unsafe` without a stated invariant. No `unwrap()`/`expect()` on data
  that comes from a document.
- Relationship targets and external links must never be fetched or opened.
- Scripts and workflows: no secrets in logs, no untrusted input
  (`github.event.*` titles, bodies, branch names) interpolated into `run:`,
  actions pinned by commit, least-privilege `permissions:`.
- Dependencies: a new crate, wheel or npm package needs a reason; prefer
  what the tree already has.

## 3. Performance and improvements

- Flag quadratic work over runs, paragraphs or revisions, `.clone()` in hot
  loops, repeated parsing of the same part, and allocation per character.
- A change that claims a speed-up must show before and after numbers; a
  fidelity change must not be traded for speed (AGENTS.md: fidelity gates
  precede speed claims).
- Suggest a simpler or existing abstraction when the PR adds a parallel one.

## 4. Best practices and tests

- Rust: no `#[allow(clippy::…)]` (fix the cause), errors as enum variants,
  not strings; public items documented, and intra-doc links that resolve.
- Every fix carries a regression test that fails without it; tests live
  beside the behaviour they protect and are deterministic. A test that
  asserts Word-mode PDF behaviour sets `RevisionStyle::Word` and also tests
  the default style.
- Python: typed, `ruff.toml`'s rules, Python 3.12 is the floor for scripts.
- Shell: `set -euo pipefail`, quoted expansions, no `ls | grep`.
- Docs: a changed flag, output or API updates the docs and CHANGELOG.md in
  the same PR; generated blocks come from `scripts/gen_docs.sh`.
- New files carry SPDX headers (REUSE).

## 5. Syntax and style

- Names say the one domain thing they are; no `Manager`, `Helper`, `Util`.
- Comments explain why, in plain sentences, at the density of the
  surrounding code.
- Commit messages and the PR body say what changed and why.
