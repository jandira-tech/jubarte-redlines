<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Jubarte Redlines Agent Guide

Before changing this repository, read
`~/T/reconciliation_plan/GET_JUBARTE_RUST.md`. It defines the source, native,
WASM, and benchmark ownership map.

## Canonical checkout

`~/T/jubarte-redlines` is the only canonical local source checkout. Older
copies under `ooxmlsdk`, `ooxmlsdk-redline`, and `jubarte_family` are historical
and must not be edited, built, benchmarked, or treated as current evidence.

The names used at each boundary are intentional:

- local source folder: `jubarte-redlines`
- GitHub repository: `jandira-tech/jubarte-redlines` (crates.io: `jubarte-redlines`; Rust path: `jubarte::`)
- Cargo package and CLI: `jubarte`
- benchmark native vendor/method: `jubarte-rust`
- benchmark WASM adapter: `jubarte-wasm`

Do not call this engine `ooxmlsdk-redline`; that is the retired name for an
older location. The `ooxmlsdk` project is a separate SDK dependency and oracle.

## Priorities and verification

Microsoft Word parity is the primary correctness target. Preserve Word-valid,
Word-faithful output before optimizing or generalizing behavior. Use upstream
Docxodus/Open-Xml-PowerTools behavior and existing fixtures before inventing
new semantics.

- Run commands from this repository root.
- Run Cargo commands sequentially in the default `target/`; never set
  `CARGO_TARGET_DIR` or start a second Cargo process while one is running.
- Do not suppress clippy warnings with `#[allow(...)]`; fix the cause.
- Keep tests beside the behavior they protect and prefer deterministic tests.
- After source changes, run formatting, clippy with `-D warnings`, the relevant
  tests with coverage, and a CLI `--help` smoke test.
- Rebuild native and WASM benchmark consumers from this checkout. Never patch a
  copied binary or generated WASM artifact as a substitute for a source fix.
- Fidelity gates precede speed claims: native/WASM `script_redlines` scores must
  agree for the same source commit before publishing performance results.

## Microsoft Word automation (benchmarks)

Word is the source of truth for every PDF and redline reference; soffice never
is. Drive Word only through the neurotic_docx_bench scripts, run from that
repository's root:

- `scripts/word_pdf.py` converts docx to PDF, `scripts/word_redline.py`
  compares pairs, and `scripts/check_redline_identity.py` rejects any redline
  that is not the pair its filename names. Always use these scripts. Never
  hand-roll osascript or a converter, not even to diagnose.
- Run `word_redline.py` in its default one-redline batch mode first. For the
  pairs the batch drops, run it again with `--no-one-redline-osascript`; it
  skips outputs that already exist. Run `check_redline_identity.py` on every
  set of redlines before scoring them.
- Keep the watchdog (`jubarte-loop/word_watchdog.sh`) pointed at the paths the
  current job actually grows. Batch mode reports progress only in its `--log`,
  not in its output folder. A watchdog on the wrong folder kills a busy Word.
  A Word freeze is normal; the watchdog clears it.
- Closing every Word document, or quitting Word, between jobs is fine.
- When Word fails twice on a reference file or pair, skip it and move on:
  without a reference there is nothing to score. If skips leave the sample
  small, add other fixtures instead of retrying the failures.
- A jubarte failure scores zero, never a skip. That covers no output, and
  output that Word cannot open or convert.
- `word_redline.py` builds its staging names from both stems, so long stems
  fail with "File name too long". Stage long pairs under short ids and keep a
  map back to the real names.

## Comparing jubarte's PDFs with Word's

jubarte renders a PDF in more than one way. `PdfOptions::revisions`
(`RevisionStyle`, the CLI's `--revisions`) picks how tracked changes and
comments are drawn: `conventional` (the default: red, blue and green marks),
`word`, or `custom`.

- Whenever a jubarte PDF is compared with one Word made (bench scores, scorer
  runs, side-by-side checks, tests that quote a Word PDF), render it with
  `--revisions word` (`RevisionStyle::Word`). The bench's own convert command
  already passes it. A default-mode PDF measured against Word measures the
  wrong renderer.
- A Word behaviour that makes the output worse (an unmarked revised PAGE
  number, a comment Word draws no balloon for) is copied only under
  `RevisionStyle::Word` and recorded in `docs/WORD_DIFFERENCES.md`. The other
  styles keep the sensible output.
- A test that asserts Word-mode behaviour sets `RevisionStyle::Word`
  explicitly. Test the default mode alongside it, so a Word quirk cannot leak
  into it.
- Word mode can be worse than our convention. Say so whenever it is, record
  the case in `docs/WORD_DIFFERENCES.md`, and offer the user both behaviours.

## Diagnosing a low score against Word

Find the cause in the files before changing layout code.

- Unpack the docx and read the XML behind each difference, especially what
  Word chose to render or to leave out (a dead comment range, a hidden or
  vanished run, an `mc:Fallback`, a zero-size frame). Start from the
  `jubarte debug FILE --check …` views.
- Run the code-driven checks on every low scorer. Tables and fills are the
  usual losers.
  - `jubarte debug FILE --check render` lists what should reach the page,
    part by part:
    - tables with their size, style, float position, shading and first
      cells;
    - highlighted and shaded text, text colours and paragraph shading;
    - the fonts asked for and what jubarte loads for each (installed,
      embedded, or an open-source substitute);
    - fields, including page numbers;
    - the order of inserted and deleted runs;
    - frames and sections.
  - `jubarte debug A.docx B.docx --check render` prints only the lines that
    differ.
  - Compare the result with the two PDFs (Word's and ours): the page count,
    the embedded fonts, and the fill and text colours on each page.
- Do not assume where the content lives. `word/document.xml` is one part.
  Headers, footers, footnotes, endnotes, comments, text boxes, and the style,
  numbering and theme parts all count.
- Headers, footers, footnotes, endnotes and page numbers matter as much as
  the body: their style, their position, and the order of revised content
  (whether the deletion comes before the insertion or after it).

## When Word cannot open a jubarte file

Each time a docx file generated by jubarte fails opening at Word, including
when Word indicates it can repair it, i.e., anything but a plain normal
opening of the file, use the ooxml validator to determine if it fails therein
as well. If it does, add a test to jubarte that prevents the issue to happen
again. Fix the code. Try again, if the generated file passes after the fix or
otherwise passed ooxml validation previously, then TDD/red/green, create a
clone of the failing docx with the fix Word requires. If it passes and content
is as expected, then implement the required change in our code that would fix
the issue, plus a test that prevents that from happening again. Don't assume
the error is what you initially thought because we get this wrong routinely.

- First confirm the failure is the file's: convert it alone, under a fresh
  name. In a batch, one document that hangs Word fails its neighbours too.
- `word_pdf.py` answers Word's repair prompt with No, so a repairable file
  already fails there ("document loaded empty") and scores zero.
- The ooxml validator is `tools/validate-docx` (OpenXmlValidator, Office
  2019); run its build, `tools/validate-docx/bin/Release/net8.0/validate-docx
  FILE`. Its silence is a pass only if it exits 0.
- `jubarte debug FILE` triages what the validator misses; `jubarte debug
  OLD NEW` prints only what changed between two builds, and `jubarte debug
  diff A B [C …]` compares packages element by element (styles by name,
  paragraphs by text, headers by role; three-way lines tagged with the
  files that hold them), e.g. A against our reject and Word's. Build
  clones and variants from its findings instead of ad-hoc scripts.
- Word-validity rules the validator misses become Ring-1 invariants in
  `tests/common/validity.rs`, with a broken probe in
  `tests/m_validity_ring1.rs`.

## Licensing and provenance

The repository's only project license is AGPL-3.0-only (`LICENSE`), and
Jandira Technologies, LLC owns its contributions. File-level licensing is
tracked with REUSE/SPDX: commentable project files carry SPDX headers, while
`REUSE.toml` covers binary fixtures and records the preserved upstream MIT
attribution texts under `LICENSES/`.

- Run `uv tool run --from 'reuse[charset-normalizer]' reuse lint` before
  changing licensing or adding non-trivial assets.
- Do not overwrite an upstream copyright notice or license identifier. Add the
  accurate provenance instead and update `REUSE.toml` when a file cannot carry
  a comment header.
- `LICENSES/` is attribution/provenance only, not an alternative licensing
  choice for this repository.
