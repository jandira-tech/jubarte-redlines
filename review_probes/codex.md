# PR #1680 correctness review at 335e67d2

The requested destination `/Users/arthrod/temp/T/fix013w/rev10/codex.md` is outside the session writable roots; a write attempt was denied with `Operation not permitted`. This file is the report in the authorized clone. Reproduction artifacts are beside it.

Review uses the supplied `jubarte-D` binary (cad208c5; user confirms behavior matches 335e67d2 apart from a test). No Word invocation, source changes, or other checkout builds.

## 1. [P1] Resolve story parts by relationships before settling their final marks

**Responsible code:** `src/document_comparer.rs:8070-8076` (the new name-based eligibility test), also gated by the identical outer sweep at `src/document_comparer.rs:7976-7983`. The comparison itself discovers headers and footers by relationships and writes the redlined content back to A's part name (`src/document_comparer.rs:7838`); the settling pass must use those resolved names too.

**Exact input / reproduction:** `python3 review_probes/story_names.py`. The script and its imported `mkdocx.py` / `probe.py` are beside this report. For the minimal header case, each document has body `Body.` and a header containing `Before.`, a two-cell table (`Fixed.` in the unchanged cell), then `After.`. A's second cell contains `Old.`; B's second cell contains `Old.`, `New.`. The script changes the ZIP member `word/header1.xml` to `word/chrome.xml` and changes both the content-type override and document relationship target to match. There are no tracked changes in either input. The footer variant uses `word/bottom.xml`. Neither case uses content controls or an empty paragraph.

```sh
BIN=/Users/arthrod/temp/T/fix013w/bin/jubarte-D
D=review_probes/story-header-renamed
"$BIN" validate "$D/a.docx" --json
"$BIN" validate "$D/b.docx" --json
"$BIN" compare "$D/a.docx" "$D/b.docx" -o "$D/out.docx" --force
"$BIN" validate "$D/out.docx" --json
# Repeat compare with --mode powertools; same result.
```

**Got:** Both input validations exit 0 without findings. Compare succeeds, but output validation exits 2 with:

```json
{"code":"CELL_FINAL_MARK_REVISION","severity":"error","part":"word/chrome.xml","location":"w:tbl[0]/w:tr[0]/w:tc[1]/w:p[1]"}
```

The final `New.` paragraph still has `w:pPr/w:rPr/w:ins` although its row and cell have no insertion. Footer gives the same finding in `word/bottom.xml`. Full results: `story_names.log` and the per-output `.jsonl` files. The standard-name controls in the same script validate cleanly in both modes.

**Expected:** Settle exactly as for `word/header1.xml` / `word/footer1.xml`: move the insertion to the preceding paragraph mark and leave the cell's final mark unrevised. The output should have no `CELL_FINAL_MARK_REVISION`. This demonstrates the invalid output through the PR's validator; Word was not opened, so an actual hang is not claimed as a fresh measurement.

## 2. [P2] Clearing a final deletion across control boundaries adds an untracked paragraph

**Responsible code:** `src/comparer/final_marks.rs:99-109`, specifically the `None => unrevise(dom, last, kind)` fallback. When a preceding inserted partner is in a different block control, joining is refused; simply removing the final deletion keeps a paragraph which must disappear on Accept All. `unrevise` removes the only deletion carrier at `src/comparer/final_marks.rs:214-216`.

**Exact input script:** `review_probes/sdt_boundary.py` (run with `python3 review_probes/sdt_boundary.py`). Its complete input construction is:

```python
from mkdocx import p, tc, tr, tbl, sdt, write
from pathlib import Path
d = Path('review_probes/sdt-boundary')
d.mkdir(exist_ok=True)
def document(blocks):
    return p('Before.') + tbl(tr(tc(p('Fixed.')), tc(*blocks)), cols=2) + p('After.')
write(d/'a.docx', document([
    sdt(p('Alpha gamma.'), p('Alpha gamma.')), sdt(p('Two.'))]))
write(d/'b.docx', document([
    sdt(p('Alpha gamma.')), sdt(p('Three.'))]))
```

Both inputs have **two block controls**; both cell-final paragraphs are inside a control. No input has an empty paragraph, and no input adds or removes a control. This is distinct from the listed one-sided final-control case and the empty-input-paragraph cases.

```sh
BIN=/Users/arthrod/temp/T/fix013w/bin/jubarte-D
D=review_probes/sdt-boundary
"$BIN" compare "$D/a.docx" "$D/b.docx" -o "$D/out.docx" --force --datetime 2026-10-23T00:00:00Z
"$BIN" accept "$D/out.docx" -o "$D/accept.docx" --force
"$BIN" reject "$D/out.docx" -o "$D/reject.docx" --force
```

**Got:** The second cell of Accept All contains the paragraph texts `["Alpha gamma.", "Three.", ""]`. B contains `["Alpha gamma.", "Three."]`. The extra final paragraph wraps an empty inline control after the deleted words have gone. It remains as an extra line/cell-height contribution. Reject All reconstructs A's three paragraph texts. Input, redline and both resolved files all pass `jubarte-D validate --json`, so validation does not reveal this reconstruction error. Exact output is recorded in `sdt_boundary.log`; all five DOCX files are in `sdt-boundary/`.

**Expected:** Accept All has exactly B's two paragraphs in the changed cell, with no untracked trailing empty paragraph. Preserve valid final marks without converting a fully deleted paragraph in another block control into surviving content.

## 3. [P1] Settling across a deleted nested table removes a paragraph break from B

**Responsible code:** `src/comparer/final_marks.rs:86-108` and `src/comparer/final_marks.rs:188-210`. The opposite-kind partner case joins an inserted paragraph to the following deleted-mark paragraph. That following paragraph can contain surviving inserted text as well as deleted text. Clearing its deletion at line 190 and physically moving the partner's words into it at lines 197-210 removes a paragraph boundary that B needs. Subsequent settling cannot recover that boundary.

**Exact input script:** `review_probes/deleted_table.py` (run `python3 review_probes/deleted_table.py`). The full input body construction, using the supplied/copied `mkdocx.py`, is:

```python
from mkdocx import p, tc, tr, tbl, write
from pathlib import Path
d = Path('review_probes/deleted-table')
d.mkdir(exist_ok=True)
def document(blocks):
    return p('Before.') + tbl(tr(tc(p('Fixed.')), tc(*blocks)), cols=2) + p('After.')
nested = tbl(tr(tc(p('Inner fixed.')), tc(p('Inner.'))), cols=2)
write(d/'a.docx', document([p('Alpha beta gamma delta.'), nested, p('Two.')]))
write(d/'b.docx', document([p('Epsilon zeta eta theta.'), p('One.')]))
```

The outer row's first cell remains unchanged. Neither input has empty paragraphs or content controls; both outer cells end in paragraphs. The nested table is deleted whole, rather than every cell of a retained row being modified.

```sh
BIN=/Users/arthrod/temp/T/fix013w/bin/jubarte-D
D=review_probes/deleted-table
"$BIN" compare "$D/a.docx" "$D/b.docx" -o "$D/out.docx" --force --datetime 2026-10-23T00:00:00Z
"$BIN" accept "$D/out.docx" -o "$D/accept.docx" --force
"$BIN" reject "$D/out.docx" -o "$D/reject.docx" --force
```

**Got:** Accept All's second cell has one paragraph, `Epsilon zeta eta theta.One.`. The two text runs share a paragraph and there is no break or separator between them. Reject All restores A's text, nested table and final paragraph. A, B, the redline and both resolved outputs validate without findings (exit 0). The same pair in PowerTools mode reconstructs B's two paragraphs, which provides an additional control. Results and commands are in `deleted_table.py` / `deleted_table.log`; all inputs and outputs are in `deleted-table/`.

**Expected:** Accept All must reproduce B's two paragraphs `["Epsilon zeta eta theta.", "One."]`. Joining a paragraph with an opposite-kind mark must account for live inserted content in the next paragraph; that mark cannot simply be removed when it separates two paragraphs of B.

## 4. [P2] Rejecting a move can expose an uncovered inserted cell-final mark

**Responsible code:** `src/changes.rs:1372-1379` / `src/changes.rs:1431-1441`: dependency handling only takes final marks along when a row/cell insertion or deletion resolves. A selected move can also remove a cell's last paragraph and expose a previously non-final revision. `src/validate.rs:1252` considers only `ins` / `del`, so the input's final `moveTo` mark is not flagged and the comparer settle rule would not handle it either. There is no post-resolution settling before serialization at `src/changes.rs:1407`.

**Exact input script:** `review_probes/selective_move.py`, executed as `python3 review_probes/selective_move.py`; it creates `selective-move/input.docx`. This is an explicitly constructed tracked document, not claimed to be emitted by Compare or freshly verified in Word. Its body contains:

- A `Moved paragraph.` paragraph with `pPr/rPr/moveFrom` id 3 and a `moveFrom` text carrier id 4 inside a `moveFromRangeStart/End` pair id 5, name `MoveA`.
- A two-cell table with an unchanged `Fixed.` first cell. The second cell has an inserted `Inserted.` paragraph (mark id 1, text id 2), followed by `Moved paragraph.` with a `moveTo` mark id 7 and `moveTo` text id 8 inside the matching `moveToRangeStart/End` pair id 6, name `MoveA`.
- An unchanged `After.` body paragraph.

All revision carriers are authored by Ann at `2026-10-23T00:00:00Z`. Both move sides use `w:t`, including moved-from text. The script gives the exact XML for every carrier and packages it using `mkdocx.write`.

```sh
BIN=/Users/arthrod/temp/T/fix013w/bin/jubarte-D
D=review_probes/selective-move
"$BIN" validate "$D/input.docx" --json
"$BIN" reject "$D/input.docx" --id r8 -o "$D/reject.docx" --force
"$BIN" validate "$D/reject.docx" --json
```

**Got:** Input validation exits 0 without findings. The single-ID reject succeeds, moves the moved paragraph back out of the cell, and removes its destination paragraph and mark. The cell now ends with the unselected inserted paragraph, retaining `w:pPr/w:rPr/w:ins w:id="1"`. Output validation exits 2 with `CELL_FINAL_MARK_REVISION` at `w:body[0]/w:tbl[0]/w:tr[0]/w:tc[1]/w:p[0]`. Selecting the move for acceptance is a clean control. `selective_move.log` records the change list, command results and full output XML.

**Expected:** A successful selective resolution from a validator-clean document must not introduce `CELL_FINAL_MARK_REVISION`; preserve the remaining insertion with a valid final-mark carrier/empty cell paragraph, or refuse a selection that cannot preserve the invariant. This is a remaining move/selection gap in the new invariant, not a claim that the PR introduced move processing. No Word hang was measured.

## 5. [P2] The new allocator mints schema-invalid IDs above a valid maximum comment ID

**Responsible code:** `src/comparer/final_marks.rs:426-435`. `Stamp::record` parses every `w:id` as `u32`, takes the maximum across all annotation kinds, and adds one. The resulting `u32` can exceed the ID values accepted by Office2019's schema. A large comment ID is preserved by the comparer, so it reaches this late pass even though revision carriers were previously renumbered.

**Exact input script:** `review_probes/id_limits.py` (run `python3 review_probes/id_limits.py`). It creates `id-comment-2147483647/a.docx` and `b.docx` using the copied `mkdocx.py`:

- An unchanged `Anchor.` paragraph with `commentRangeStart`, `commentRangeEnd` and `commentReference`, all `w:id="2147483647"`.
- A matching comment definition of the same ID in `word/comments.xml`, authored by Ann, text `Comment.`; the script adds its content-type override and document relationship.
- A three-cell table. Cell 1 is unchanged `Fixed.`. A's cell 2 is a centered, bold-mark paragraph `Old0.`; B keeps that paragraph and appends right-aligned `New0.` with no bold mark. Cell 3 does the same for `Old1.` / `New1.`. The body ends with `After.`.

Both source packages pass the repository's **OpenXmlValidator Office2019**, with exit 0 and no findings; this input ID is not merely accepted by Jubarte's validator.

```sh
BIN=/Users/arthrod/temp/T/fix013w/bin/jubarte-D
D=review_probes/id-comment-2147483647
SDK=tools/validate-docx/bin/Release/net8.0/validate-docx
"$SDK" "$D/a.docx"
"$SDK" "$D/b.docx"
"$BIN" compare "$D/a.docx" "$D/b.docx" -o "$D/out.docx" --force --datetime 2026-10-23T00:00:00Z
"$SDK" "$D/out.docx"
"$BIN" validate "$D/out.docx" --json
```

**Got:** Compare succeeds. Its new `pPrChange` / mark `rPrChange` records receive IDs `2147483648`, `2147483649`, `2147483650`, `2147483651`. OpenXmlValidator exits 1 and reports four `Sch_AttributeUnionFailedEx` findings, one for each new ID, e.g.:

```text
The 'http://schemas.openxmlformats.org/wordprocessingml/2006/main:id' attribute is invalid - The value '2147483648' is not valid according to any of the memberTypes of the union.
/word/document.xml
/w:document[1]/w:body[1]/w:tbl[1]/w:tr[1]/w:tc[2]/w:p[2]/w:pPr[1]/w:pPrChange[1]
```

`jubarte-D validate --json` still exits 0 without findings. The SDK results, including separate exit codes for both valid inputs, are in `openxml-confirmed.jsonl`. This is a schema-valid-input to schema-invalid-output reproduction; Word was not opened.

**Expected:** Allocate unused revision IDs within the schema's accepted range (there are ample small free IDs here), rather than using the maximum unrelated annotation ID plus one.

## Verification and evidence bounds

All five findings use saved inputs and outputs in this folder, with executable reproduction scripts named above. `manifest.json` records the reviewed commit, comparison base, supplied binary hash and SHA-256 hashes of the final reproduction files. Embedded input-construction fragments use `mkdocx.py` from this folder; the listed `python3 review_probes/NAME.py` invocations work directly from the clone root.

The repository's `tools/validate-docx` was built in this clone against its declared OpenXml SDK 3.2.0, with Office2019 validation. Build succeeded with zero warnings/errors. Sixteen final package checks (separate recorded exit codes in `openxml-confirmed.jsonl`) corroborate the valid inputs and the exact schema boundary of finding 5. Its regenerated, timestamp-pinned artifacts were rechecked in `id-openxml.jsonl`: A and B each exit 0; the output exits 1 with four invalid-ID findings. The content-control, nested-table and move fixtures and relevant resolved outputs are schema-clean; their failures are reconstruction or Ring-1 invariant failures rather than SDK schema failures. `jubarte-D --help` also exits 0 (`smoke-help.txt`).

Only reproduction artifacts and validator build outputs were written inside this clone. No engine source was changed, no Cargo test suite was run, no Word invocation was made, and nothing was pushed or posted. Coverage: N/A lines, N/A branches; these are black-box reproductions using a prebuilt Rust binary, not an instrumented unit-test run. Broader exploratory outputs are retained, but only the five confirmed findings above are reported. In particular, the duplicate-ID experiment with an out-of-range source comment ID is not a finding because independent schema validation rejected that input; finding 5 uses the schema-valid source ID 2147483647 instead.

The requested external report path could not be written under this session's filesystem policy. This report is located at `/Users/arthrod/temp/T/rev013/codex/review_probes/codex.md`.

DONE
