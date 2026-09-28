<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Assessment of the adoption plan bundle (Python API and CLI focus)

Written 2026-09-26 after reading, in this order: the four hand-rolled session
files (`file_logs.jsonl`, `run_pipeline.sh`, `notes_spacing.py`, `redline.py`),
the Anthropic `docx` SKILL.md they were written against, `RESULTS.md`, then the
master plan, 01, 02, 04, 06 and VALIDATION, then patches 0001, 0002 and 0005,
then the engine source at `892ddbc`.

This file records what the plans get right, what they get wrong, and the
corrections that the implementation on branch `feat/agent-adoption` follows.
Section 6 maps every correction to code.

## 1. What the hand-rolled files prove the agent needed

`redline.py` is 170 lines of string surgery over `word/document.xml`, written
after reading the skill. Every function in it is a missing primitive:

| Hand-rolled code | Need | Failure mode it papered over |
|---|---|---|
| `RPR`, `RPR_B`, `RPR_HL`, `RPR_I` constants copied from the file | Inherit the run formatting of the text being edited | A typo in a copied `w:rPr` silently changes the font of the inserted text |
| `once(old, new)` asserting exactly one anchor match | Unique anchors | Wrong anchor edits the wrong clause without any error |
| `split_run(full, parts)` | Replace a substring inside one run as keep/del/ins segments | Keep segments must reproduce the original byte-for-byte; failed on smart quotes and `&amp;` |
| `para_containing(">Third Parties. <")` | Select a paragraph by its text | Matching against XML serialization, so `&amp;` vs `&` and run splits break it |
| Deleted paragraph mark plus `<w:del>` around every run | Delete a whole paragraph as a tracked change | Forgetting the mark leaves an empty bullet after accept |
| New `<w:p>` with cloned `pPr` and a tracked paragraph mark | Insert a tracked paragraph after an anchor paragraph | Cloning `pPr` blindly also clones section breaks and revision ids |
| `pPrChange` wrapping for indent and line spacing (`notes_spacing.py`) | Tracked formatting change | Regex over `<w:pPr>` blocks; no schema ordering guarantee |
| `subprocess` to `comment.py`, then manual `commentRangeStart/End` placement by string index | Comment anchored to text, including text the same script just inserted | Anchoring by `x.index(escape(anchor))` picks the first occurrence in the file |
| `merge_runs.py` before everything | Text findable across Word's run fragmentation | Without it, no anchor matches |
| `validate.py --author Claude` | Prove every edit is tracked | Untracked edits are invisible in the accepted view |
| `accept_changes.py` (LibreOffice macro, 30 s timeout treated as success) | Clean copy | Times out silently; misjoins deleted paragraph marks next to spacer paragraphs |
| `soffice --convert-to pdf` x3, `pdfinfo`, `pdftoppm -jpeg` | Page counts and page images to look at | Three subprocess tools, one of them 400 MB, whose pagination is not Word's |
| `file_logs.jsonl` (hand-written) | A machine-readable record of what changed, per operation, with revision ids and page deltas | Nothing produced it; the agent invented the format |

The reading path in the skill is `pandoc -t markdown` (`acme_letter.txt` is
that output). It gives prose with bold and highlight marks but no paragraph
identity, so nothing in it can be used as an edit coordinate.

So the need is not "a Python wrapper over compare". It is one engine that:

1. reads a DOCX into paragraphs the agent can address (ids, text, style,
   numbering, formatting spans), with a Markdown view that carries the ids;
2. applies a small set of exact, uniquely anchored operations (replace, insert,
   delete text; insert and delete paragraphs; comments) and produces the
   tracked-changes document, the clean document and a per-operation report;
3. produces the clean copy without LibreOffice;
4. renders pages to PDF and to PNG without LibreOffice or Poppler, and reports
   page count and page-start text;
5. exposes all of that identically from Rust, the CLI and Python.

## 2. RESULTS.md: is the evidence convincing?

Convincing for one claim, absent for the others.

**Convincing.** "docx to pdf, every redlined corpus pooled": jubarte 0.9.2
Jaccard 0.4656 vs soffice 0.2291, same 1,416 documents, same date, same
metric, both scored against Word's own PDF export. That is a clean head-to-head
and it favors jubarte by 2x. The neurotic harness rows (88.67 vs 73.98 for
LibreOffice on 398 clean documents) point the same way, though the dates differ
by three weeks, so harness drift is possible.

**Weak.** The clean pooled table compares jubarte over 2,102 documents with
soffice over 1,602: jubarte's row includes `fixtures_500`, soffice's does not.
The file says rows are only comparable inside one table, but inside this table
the document sets differ. Tools that fail on hard documents lose those documents
from their denominator (`office2pdf` covers 204 of 208), which inflates their
means; the file does not report failure counts.

**Noise.** The redline-speed table ranks `jubarte-rust-inproc` at 3.38 ms
(n=2) above the same tool at 25.34 ms (n=5000). Rows with n=2 or n=20 sit
beside rows with n=5000 and no variance. Nothing should be quoted from that
table.

**Absent.** Nothing in RESULTS.md measures: accept-all correctness against
Word (the thing `accept_changes.py` does), text extraction fidelity (the thing
`pandoc` does), page-count agreement with Word (the number the agent actually
logs), or rasterization (the thing `pdftoppm` does). `pdf to docx` has no runs.
The Jaccard metric itself is not defined in the file.

Consequence for the pitch: "toss soffice for rendering" is supported by the
redlined-corpus table; "toss soffice for accept" and "toss pandoc for reading"
rest on the engine being a port of Open-Xml-PowerTools' RevisionProcessor and
on the inspection tests added here, not on RESULTS.md. Say so.

## 3. Plan-by-plan critique

### Master plan

- Correct to keep Rust as the only place that interprets OOXML, to make new
  APIs additive, to bind plans to a source hash, and to refuse ambiguity.
- Wrong emphasis: 18 technical plus 14 growth proposals, a 90-day experiment
  program, launch assets and partner drafts, while the two things the agent
  actually needed (address a paragraph, apply an anchored edit) are milestone
  M2 and M4, weeks 4 to 14. Adoption by agents follows from removing the
  hand-rolling, not from the growth program.
- "Keep the canonical checkout; AGENTS.md overrides the worktree
  recommendation" was overridden by the user for this implementation, which
  runs in a worktree on branch `feat/agent-adoption`.

### 01 core and fidelity (patch 0001)

- The parser progress guard is right and small.
- `validate_xml` is right; its `ends_with(b"/>")` test is unnecessary because
  quick-xml already distinguishes `Event::Start` from `Event::Empty`.
- The body projection makes `mc:AlternateContent`, `w:altChunk`, `w:sym` and
  column breaks fatal. `mc:AlternateContent` wraps every anchored drawing Word
  has written since 2010, and `w:sym` is how Symbol-font bullets and checkboxes
  are encoded in most legal forms. A reader that refuses those documents sends
  the agent back to pandoc. Correction: unsupported constructs become
  per-paragraph `limitations`; text boxes stay excluded; `w:sym` projects to
  U+FFFC so anchors remain deterministic.
- `ParagraphInfo { index, text, page_break }` is too thin to address a
  paragraph. Correction: add `id`, `style`, `numbered`, `in_table` and
  formatting spans (`runs`), and a Markdown projection that prints the id in
  front of each paragraph.
- Task C2 (ZIP admission limits) is correctly specified and correctly marked
  as not implemented by the patch. It remains open work.

### 02 Python API (patch 0002)

- The facade design (immutable `Document`, `from_bytes` vs `read`, typed
  options, legacy functions untouched) is right and is kept.
- It has no editing, no comments, no text extraction, no rasterization and no
  CLI section. For the agent workflow that is the whole gap. Correction: the
  facade gains `inspect()`, `markdown()`, `edit(plan)`, `to_png()` and
  `render()`, and `python -m jubarte_redlines` exposes every operation the
  `jubarte` binary exposes, with the same names and JSON shapes.
- PY2's `Paragraph(index, text, page_break)` inherits the 0001 thinness.

### 04 semantic editing (patch 0005)

- The architecture (apply untracked edits to a copy, let the comparer produce
  the redline) is a good reuse of the engine's strength. Word Compare produces
  redlines the same way, so the output is what reviewers expect.
- E5 treats comments on inserted text as a provenance-mapping problem. It is
  not: `tests/m35_comments.rs` documents that the comparer carries the modified
  side's comments through compare, re-anchoring them at the equivalent text
  positions and surviving del/ins wrapping. Correction: author comments in the
  clean copy before compare, including comments on text an earlier operation
  inserted. The implementation verifies this on a fixture.
- Patch 0005's "simple paragraph" rule refuses any paragraph with
  `w:proofErr`, `w:bookmarkStart`/`End`, `w:tab`, `w:br` or a hyperlink. The
  `_GoBack` bookmark and proofing marks are in nearly every Word file, so
  most real paragraphs would be refused. Correction: project paragraph text
  from the runs (zero-width markers ignored, `w:tab` as `\t`, `w:br` as `\n`)
  and refuse only when the matched range crosses a run that is not plain
  text or sits inside a field, hyperlink, content control or revision.
- `TextEdit.expected_text` (echo the whole paragraph) is redundant for a
  plan that carries `source_sha256`, which guards the snapshot (the field is
  optional; an unguarded plan relies on unique anchors alone and its report
  says so); agents will get it wrong on smart quotes
  and pay tokens for it. Correction: drop it; `paragraph` accepts an id, an
  index, or a unique `starts_with`/`contains` selector.
- Replacement text is inserted into the first affected `w:t`, so it inherits
  that run's formatting. That is the right default and it removes the `RPR`
  constants from `redline.py`. Insert-paragraph runs need explicit bold and
  highlight flags (Acme #5, #10); the implementation provides `runs` with
  `bold`, `italic`, `highlight`.
- `existing_revisions` policy: refuse by default is right. The refusal must
  count revision carriers in stories the edit touches, not `w:numberingChange`
  inside `numbering.xml`.
- The plan does not name where the per-operation report is written. The agent
  invented `file_logs.jsonl`. Correction: `jubarte edit` writes
  `report.jsonl` with `load`, one `op` line per operation, `compare`, `render`
  and `summary` events; Python returns the same records as objects.

### 06 agent adoption (patch 0006)

- The skill teaches compare, render, accept and reject only, so it cannot
  replace the `docx` skill's editing path, which is the path the agent used.
  Correction: `skills/jubarte-documents/SKILL.md` covers read (inspect,
  markdown), edit (plan), verify (render to PNG, page count, report) and
  clean copy, and lists the remaining gotchas that survive the redesign.
- `capabilities --json` derived from the built feature set is right and is
  implemented.
- The six-task evaluation protocol is good and is not implemented here.

### VALIDATION

- Honest. Its main gap is structural: the Rust patches were never compiled,
  and the Python facade was tested against a wheel that did not contain the
  Rust changes. Everything in section 6 below was compiled and tested in the
  worktree; numbers are in section 7.

## 4. What "toss pandoc, soffice, pdftoppm" concretely requires

| Tool | Skill use | Replacement | Status after this branch |
|---|---|---|---|
| pandoc | `pandoc -t markdown file.docx` to read | `jubarte text file.docx` (Markdown with paragraph ids); `jubarte inspect --json` | implemented; body story only, headers/footers/notes reported in `summary` |
| pandoc | `--track-changes=accept` | `jubarte accept` (RevisionProcessor port) | already existed |
| soffice | `--convert-to pdf` | `jubarte convert --pdf` | already existed; page count and page starts added to the report |
| soffice | `accept_changes.py` macro | `jubarte accept` | already existed |
| soffice | `.doc` to `.docx` | none | not replaced; the skill says so |
| pdftoppm | PDF pages to JPEG for the agent to look at | `jubarte convert --png --dpi N` from the same layout, no PDF round-trip | implemented |
| pdfinfo | page count | `render` report `page_count` | implemented |

## 5. Corrections applied to the patch set

- 0001: kept the parser guard and `validate_xml` (simplified); replaced the
  body projection with the richer, non-refusing one; `summary` kept.
- 0002: kept the facade; extended it; tests re-run against the rebuilt
  binding, not the PyPI wheel.
- 0005: superseded by `src/edit.rs` (plan JSON v1, selectors, comments,
  paragraph operations, report). Its refusal codes are kept where they still
  apply (`STALE_SOURCE`, `ANCHOR_NOT_FOUND`, `AMBIGUOUS_ANCHOR`,
  `OVERLAPPING_EDITS`, `EXISTING_REVISIONS`, `UNSUPPORTED_STRUCTURE`).
- 0006: superseded by the rewritten skill.
- 0003, 0004, 0007 (TypeScript, recipes, playground): out of scope here and
  untouched.

## 6. Correction to code map

| Correction | Code | Tests |
|---|---|---|
| Parser progress, checked XML | `src/xmllinq/parse.rs` | `checked_xml_tests` |
| Rich paragraph projection, limitations instead of errors, Markdown | `src/inspect.rs` | unit tests in module |
| Plan JSON v1, selectors, text ops, paragraph ops, comments, compare, report | `src/edit.rs` | unit tests in module, `tests/edit_plan.rs` |
| PNG pages from the layout display list | `src/convert/raster.rs` | unit tests in module, `tests/convert_docx_to_png.rs` |
| CLI parity | `src/bin/jubarte.rs` | `tests/m_cli_agent.rs` |
| Python facade, native exports, module CLI | `jubarte-python/` | `jubarte-python/tests/` |
| Skill | `skills/jubarte-documents/SKILL.md` | manual routing check |

## 7. Verification record

Filled in at the end of the implementation session; see the bottom of this
file.

### Verification record (2026-09-26, worktree `feat/agent-adoption` on `892ddbc`)

Environment: Rust 1.95.0, Python 3.11.15, maturin 1.15.0, 2 CPUs, 8 GB memory
cgroup; Cargo serialized (`-j1`); `ooxmlsdk` (dev-dependency) built without
debuginfo through an uncommitted `.cargo/config.toml` because it exceeds the
cgroup with debuginfo. No agents were used.

| Gate | Result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean (after fixing three pre-existing sites the 1.95 lint set flagged: `altchunk.rs` end(), `convert/mod.rs` stroke boolean, `tests/m10_deleted_drawing.rs`) |
| `cargo test --lib` | 568 passed, 1 ignored (perf timing) |
| `cargo test --bin jubarte` | 25 passed |
| `tests/inspect_paragraphs.rs` | 13 passed |
| `tests/edit_plan.rs` | 18 passed (includes comments on inserted text surviving compare, accept(redline) == clean for every operation kind) |
| `tests/convert_docx_to_png.rs` | 4 passed |
| `tests/m_cli_agent.rs` | 7 passed |
| Existing `m7_cli`, `m_cli_no_panic` | 15 + 10 passed |
| Regression around the render refactor: `convert_docx_to_pdf` | 1234 passed, 3 ignored |
| `convert_revision_palette`, `m10_deleted_drawing`, `m11_f1_roundtrip`, `m22_alternate_content_resolve`, `m35_comments`, `m4h_parts` | 7 + 1 + 2 + 6 + 5 + 8 passed |
| `jubarte --help` | exit 0 |
| Python `pytest --cov=jubarte_redlines --cov-branch` against the rebuilt binding | 48 passed; 98% lines (660 statements, 9 missed); 160 of 166 branches |
| Acme walkthrough (`examples/agents/acme-letter/`) | 10 of 10 operations ok, 4 comments, 15 revision records, PDF and PNG rendered, page inspected visually |

Not measured: Rust line/branch coverage. `cargo llvm-cov` needs an
instrumented rebuild of every crate, and the instrumented `ooxmlsdk` does not
fit this container; run it on the canonical machine
(`cargo +nightly llvm-cov --branch --lib inspect:: edit:: capabilities:: convert::raster::`).
The full `cargo test` over all 150+ suites was likewise not run here; the
suites above were chosen to cover every file the branch touches.

Known gaps found while verifying, left open on purpose:

- The PDF/PNG layout paints comment balloons for comments on inserted
  paragraphs but not for comments anchored inside inserted runs (the
  comparer places `commentRangeStart` inside `w:ins`, which Word accepts).
  The comments are in the DOCX; `jubarte inspect` counts them. Renderer work,
  not edit work.
- Compare-based redlines show a long replacement as a word-level diff, as
  Word Compare does. (Closed on release/0.10: `replace` takes
  `"whole": true`, which shows the whole old clause deleted and then the
  whole new clause inserted; a diff that cannot be regrouped stays
  word-level and the report says why.)
- Headers/footers/notes/text boxes as editable stories. (`merge_paragraphs`,
  `format_paragraph`, formatting on inline `insert`/`replace` and the ZIP
  admission limits of 01 C2 landed on release/0.10.)
- `RESULTS.md`'s `accepted_changes` tables score compare output after
  acceptance; they are not a test of the accept operation on arbitrary
  redlines. The claim "toss LibreOffice for accept" rests on the
  RevisionProcessor port and on `accept(redline) == clean` in
  `tests/edit_plan.rs`, not on RESULTS.md.
