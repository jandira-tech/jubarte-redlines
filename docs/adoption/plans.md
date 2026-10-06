<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# What the adoption pages say jubarte cannot do, checked three times

Each item below was a "you lose" or "pending" line on
[anthropic-docx-skill.md](anthropic-docx-skill.md),
[openai-doc-skill.md](openai-doc-skill.md) or
[install-matrix.md](install-matrix.md) before 2026-10-04. Each was checked
three ways: in the source on `main`, by running the command, and against the
substituted tool in [`examples/adoption/`](../../examples/adoption/). Where
Word decides the answer, the check went through Word
(`neurotic_docx_bench/scripts/word_pdf.py` and `word_redline.py`).

Status words: **done** (on this branch, with a test), **minimum** (a bare
minimum works; the plan lists what is still missing), **plan** (not started),
**not a goal** (we will not do it, and why).

## 1. Legacy `.doc`: minimum

- Before: every entry point refused a `.doc` with `LEGACY_DOC`; the pages
  said "keep `soffice --convert-to docx`".
- Checked: `admission::sniff` refused OLE files on `main`; `jubarte text
  x.doc` exited 1; no reader existed in `src/`.
- Now: `jubarte convert old.doc` writes `old.docx` (`-t md` and PDF/PNG
  through it), from `src/legacy_doc.rs`: OLE compound file, FIB, piece
  table, PAPX (styles, tables, lists, header rows), CHPX and the piece
  table's property modifiers (bold, italic), STSH (Heading 1-9, Title),
  PlfLst/PlfLfo (bullet or number per level). Every other
  command still refuses the file and names the convert step. Example:
  [`18-legacy-doc`](../../examples/adoption/18-legacy-doc/); tests:
  `tests/legacy_doc_convert.rs`, `tests/adoption.rs`.
- Next, in order of what contracts use:
  1. A `.doc` saved by Microsoft Word in the fixtures. The one fixture is
     LibreOffice's Word 97 export. Word must make it through the bench
     scripts (a `word_save_as.py` beside `word_pdf.py`), not a hand-rolled
     AppleScript.
  2. Headers, footers and footnotes: the `ccpHdd` and `ccpFtn` ranges of
     the same piece table, split by `PlcfHdd` and `PlcffndTxt`.
  3. Underline, font size and colour (`sprmCKul`, `sprmCHps`, `sprmCIco`)
     as run properties, which needs a direct `.docx` writer instead of the
     Markdown route.
  4. Page setup from the section table (`PlcfSed` to SEPX: page size,
     margins, orientation); the output is US Letter until then.
  5. Pictures (`PICF` in the Data stream), comments (`PlcfandTxt`) and
     tracked changes (`sprmCFRMark`/`sprmCFRMarkDel` runs).
  6. Python, WASM and MCP entry points (`doc_to_docx`), once 1-3 land.
  7. Not planned: Word 6/95 files (`nFib` below 193) and encrypted `.doc`
     (RC4 CryptoAPI). Both stay `LEGACY_DOC` with the save-as hint.

## 2. XSD (schema) validation: plan

- Before: "Keep `validate.py` if you want the XSD pass."
- Checked: `src/validate.rs` says schema validation stays with
  `tools/validate-docx`, a .NET tool that does not ship. `jubarte validate`
  checks what makes Word refuse or repair a file (Ring 1), which is a
  different question: in [`07-validate`](../../examples/adoption/07-validate/)
  a `w:ins` with no `w:id` breaks the schema, yet Word 16 opens it with no
  repair prompt (probed 2026-10-04), and `jubarte validate` passes it.
- Bare minimum: `jubarte validate --schema FILE`, reporting schema findings
  as `SCHEMA_*` codes with `word_fatal: false` unless a Ring-1 rule also
  fires. `ooxmlsdk` (already a dev-dependency, the Rust port of the Open XML
  SDK that `tools/validate-docx` wraps) has a `validators` feature exposing
  `ValidationContext` and `ValidationSettings { file_format,
  max_number_of_errors }`.
  1. Add `ooxmlsdk = { features = ["parts", "validators"], optional = true }`
     behind a `schema` cargo feature, so the default binary and the WASM
     build do not grow.
  2. Map `ValidationErrorInfo { error_type, description, id, part_uri }`
     onto the validator's `Finding`.
  3. Test against the three `07-validate` files and the Ring-1 probes in
     `tests/m_validity_ring1.rs`: every probe gives the same verdict as
     `tools/validate-docx` (Office 2019 file format).
  4. Ship it in the release binary and the wheel only if the size cost is
     under 2 MB; otherwise as `cargo install jubarte-redlines --features
     schema`.

## 3. Building complex documents from code: minimum

- Before: "docx-js remains the tool for documents built programmatically";
  "a skill that builds a document object by object keeps python-docx".
- Checked: Markdown to `.docx` writes headings, lists, tables, images,
  footnotes, links, CriticMarkup, `--reference-doc` styles and `--page
  letter|a4` ([`14-create-from-markdown`](../../examples/adoption/14-create-from-markdown/));
  `insert_table`, `list`, `insert_image`, `format_run`, `insert_footnote`
  and `page_setup` are edit-plan operations whose `clean.docx` is an
  untracked document ([`15-tables-lists`](../../examples/adoption/15-tables-lists/)).
  So `jubarte convert skeleton.md` then `jubarte edit --plan` builds a
  document with no Node or python-docx.
- Gap found: a plan cannot address a paragraph it inserted itself. A `list`
  operation over paragraphs an earlier `insert_paragraph` added is refused
  with `ANCHOR_NOT_FOUND`, because selectors resolve against the source.
  Building in one plan therefore takes several chained plans.
- Plan: a selector `{"inserted_by": "<operation id>"}` that resolves to the
  paragraphs an earlier operation of the same plan inserted, applied in
  plan order. Tests: a one-plan build of a heading, a list over inserted
  paragraphs and a table after them, Word-valid, and its redline accepted
  equal to its clean copy.

## 4. Rendering identical to LibreOffice: not a goal

jubarte lays documents out as Microsoft Word does; matching LibreOffice
would mean matching its differences from Word. The examples show where the
two disagree (font choices in
[`04-font-substitution`](../../examples/adoption/04-font-substitution/), A4
versus Letter for a page size no one set in
[`14-create-from-markdown`](../../examples/adoption/14-create-from-markdown/)).
A skill that needs LibreOffice's pixels keeps LibreOffice.

## 5. A render timeout (Codex #38313): done

- Before: "There is no timeout flag; wrap the call in your sandbox's own
  timeout."
- Now: `jubarte convert … --timeout SECONDS` exits 124 (`timeout(1)`'s
  status) once the limit passes. An output being written at that moment can
  be left partial. Test: `convert_timeout_exits_124_past_the_deadline`.

## 6. Linux older than glibc 2.28: plan

- Checked: the release builds `x86_64-unknown-linux-gnu` and
  `aarch64-unknown-linux-gnu` binaries and `manylinux_2_28` /
  `musllinux_1_2` wheels. pip on Debian 10, RHEL 7 or Amazon Linux 2 picks
  neither wheel and builds the sdist, which needs Rust.
- Bare minimum: add `x86_64-unknown-linux-musl` and
  `aarch64-unknown-linux-musl` static binaries to the GitHub release
  matrix in `.github/workflows/release.yml`; they run on any Linux kernel
  of the last decade, whatever the libc. Then a `manylinux2014` wheel
  (glibc 2.17) with `maturin --zig --compatibility manylinux2014`, checked
  by `scripts/check_release_artifacts.py`.

## 7. Windows on Arm: plan

- Checked: no `aarch64-pc-windows-msvc` target in the release matrix, no
  Windows arm64 wheel.
- Bare minimum: add the target to the binary matrix on GitHub's
  `windows-11-arm` runner, and `--target aarch64-pc-windows-msvc` to the
  wheel job; extend `check_release_artifacts.py` so a release without them
  fails.

## 8. Smaller gaps the examples found

| Gap | Found in | Plan |
|---|---|---|
| The pages called `occurrence` (S1), `--fail-on-substitution` (S9), `--page` (S7) and `LEGACY_DOC` (S18) pending; all four shipped in 0.11.2. | 04, 13, 14 | **done**: pages corrected. |
| The OpenAI page wrote the `list` value as "bulleted"; the plan takes `bullet`. | 15 | **done**: page corrected. |
| The pages named MCP tools `text`, `edit`, ...; the server's tools are `docx_text`, `docx_edit`, ... | 17 | **done**: pages corrected. |
| `jubarte-mcp` answered `initialize` with an empty `serverInfo.version`. | 17 | **done**: the package version; asserted in `jubarte-python/tests/test_mcp_server.py`. |
| The font report called a missing family placed by its name's class (`Fake Serif Pro` on Times, `Georgia Pro` on Georgia, `Helvetica` on Arial) an explicit match, so `--fail-on-substitution` passed. | 04 | **done**: reported `generic`, counted as substituted; layout unchanged. |
| `convert -t md` drops headers and footers (as pandoc does); `jubarte text` keeps them. | 05 | plan: write each header and footer story after the body under a `<!-- header: default -->` comment, as `jubarte text` orders them. |
| `validate --json` on a refused package prints a plain `error:` line. | 07 | plan: print `{"refused": {"code": ..., "message": ...}}` and keep exit 1. |
| `jubarte changes` gives a revision without `w:id` the id `body:rev:`; id-less revisions share it, so `--id` cannot select one of them alone. | 07 | plan: a stable synthetic id (`body:rev:n3`, by document order) when `w:id` is missing. |
| An edit with one replace and one paragraph delete lists six revisions, two of them run-property changes on empty runs. | 09 | plan: compare with Word's own revisions for the same edit (bench `word_redline.py` on before/after) before changing anything. |
| A page that starts inside a table or a long paragraph gets no page marker of its own. | 06 | minimum: Markdown cannot split a table. Plan: `<!-- page N continues -->` after the block that crossed the break. |
| A `.doc` run whose bold or italic sprm says "as the style" (`0x80`) or "the opposite of the style" (`0x81`) is read as off or on, without the style. A run set to `0x81` inside a bold heading comes out bold where Word draws it plain. | 01 | plan: read each style's character properties from the STSH (`grpprl` of its UPX, following `istdBase`) and resolve `0x80`/`0x81` against them; a fixture with an un-bolded word in a bold style. The same reader gives a run its character style (`sprmCIstd`: Strong, Emphasis), whose bold and italic are lost today (Codex #354 4190597426); one Word-made fixture covers both. |
| `.doc` Title and Heading 7, 8 and 9 came out of `.docx` conversion as Heading 1 and Heading 6. | 01 | **done**: the `.docx` keeps Title and Heading 7-9 (`tests/fixtures/legacy/styles.doc`, opened in Word). Markdown output still writes `#` and `######`: CommonMark has no Title and stops at six levels. |
| `paginate` now ignores fence-like text inside a paragraph, a title quoted in the paragraph before it, and the gaps of a loose list (adversarial review, R2). | 06 | **done**: unit tests in `src/markdown/pages.rs`. |
| A `.doc` to `.docx` run ignored `--report`, `--font-report` and `--fail-on-substitution`. | 01 | **done**: refused for Word and Markdown output, as `--pages` without `--png` is. |
| A `.doc` lost the bold or italic a piece's property modifier (`Prm0`, or a `Prm1` naming a `Prc`) sets over its CHPX. | 01 | **done**: applied after the CHPX ([MS-DOC] 2.4.6.2); unit tests in `src/legacy_doc.rs`. No fixture carries a `Prc` yet: LibreOffice never writes one, and Word writes them only on fast save. |
| A `.doc`'s empty paragraphs (spacers, a paragraph holding only a page break) were dropped, so the `.docx` came out tighter than Word's. | 01 | **done** for `.docx` output: kept, with their style. Markdown output still leaves them out (it cannot hold one). Ring-1 valid; not yet opened in Word. |
| Every table of a `.doc` came out with its first row repeating as a header (`w:tblHeader`), whatever the source said. | 01 | **done** for `.docx` output: only the leading rows marked with `sprmTTableHeader` repeat. Markdown output still makes the first row a header: a GitHub table must have one. Ring-1 valid; not yet opened in Word. |
| A `.doc` whose style sheet, property bin tables, FKP pages, list tables or section table point outside their stream, or whose piece table runs backwards, converted without them (or empty). | 01 | **done**: refused with `LEGACY_DOC`, as a piece past the stream already was. |
| A `.doc` heading that is also a numbered list item (`sprmPIlfo`/`sprmPIlvl` on a Heading style, as contracts number their articles) comes out without its number, and every numbered list comes out decimal whatever its `nfc` (`a.`, `(i)`, `I.` become `1.`; Codex #354 4190597422). | 01 | plan: carry the list item on the heading and, for `.docx` output, write `w:numPr` on it with numbering definitions built from the document's own LVLs (`nfc`, `lvlText`, start and restart), so Word computes the number and draws the source's format. Markdown output keeps the heading unnumbered and lists decimal: CommonMark has neither. A fixture with numbered Heading 1-2, opened in Word. |
