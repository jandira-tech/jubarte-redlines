<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

See [docs/VERSIONING.md](docs/VERSIONING.md) for the release codemod and cross-repo steps.

## [Unreleased]

### Added

- Regression coverage for a caption followed by a 60×30pt inline picture in a narrow fixed or autofit table cell, with a narrow body paragraph as control (#1734). The run-order fix in #1980 already keeps the caption above the wrapped picture; both Word and conventional revision styles are covered.

- **A non-finite table measure is a Ring-1 finding** (#1737). `w:w` on `gridCol`, `tcW` and `tblW` that `f32` accepts and that is not finite (`NaN`, `inf`, `infinity`, or an overflow such as `1e40`) is `NON_FINITE_MEASURE`. `word_fatal` is true for `gridCol` and false for `tcW` and `tblW`. Word 16.115 (2026-10-31) refused `gridcol-NaN`, `gridcol-inf` and `gridcol-1e40` (unreadable content) and opened `tcw-*`, `tblw-*` and `valid-control` cleanly. `repairable` stays false. A token that does not parse (`abc`, `12pt`) is not this finding.

- **`read FILE --find TEXT` finds paragraphs in the Markdown view** (#1889). Matches keep the same Markdown and id lines as `read -p`, using the original final logical text searched by `--for-edit`. `--regex PATTERN` requires its pattern and conflicts with `--find`; both support `--ignore-case`. Existing `--for-edit --find` combinations with `--json` or `--tables` remain accepted; no match exits successfully with empty stdout and a stderr notice. Matches outside the selection name up to three paragraph ids and the remaining count. With `--track-changes reject`, a note explains that search uses accepted text while the view shows the original. Like `read -p`, view search selects body paragraphs only; use `--for-edit` to search header/footer/note paragraphs. `edit FILE --anchor TEXT` and `add FILE --anchor TEXT` without `-p` now direct the caller to `-p WHERE` and `read --find`, keeping usage exit code 2.
- **`--in-place` replaces the input through the same atomic write as `-o`** (#1888). `edit`, `add`, `accept` and `reject` take `--in-place`. It conflicts with `-o` and, on `edit` and `add`, with `--out-dir` (exit 2). The input is replaced only after the edit or the resolution succeeds; a refused edit (exit 3) or a failed check leaves it byte-identical. The line is `wrote contract.docx in place (replaced sha256 <first 8 hex>…)`, and an `edit` or `add` that adds markup keeps `; N change(s) by "X"` and the no `-a` warning. With no `-o`, the default `<stem>_modified.docx` write adds `; contract.docx is unchanged (--in-place updates it)`. That note is absent when `-o` or `--out-dir` names the output, and when the input's stem already ends in `_modified` (that default already updates the file). Typed `edit replace`, `rewrite`, `insert`, `delete` and `apply` take `--in-place` too (not with `-o`, `--out-dir`, `--dry-run` or `--save-plan`). Their receipt is `edit replace: changed; wrote FILE in place (replaced sha256 <first 8 hex>…); 1 change by "GiantCo"`, and a refused edit leaves FILE byte-identical. `python -m jubarte_redlines` writes that same `wrote` line for `edit`, `add`, `accept` and `reject`, including the markup detail and the no `-a` warning.

- **`--keep-others` refuses to erase another author's markup** (#1892, #1890). One opt-in covers tracked changes and comments. `--keep-others` on `edit`, `add`, `edit replace`, `edit rewrite`, `edit insert`, `edit delete`, `edit apply` and `edit template`, the environment variable `JUBARTE_KEEP_OTHERS=1` (read only by the CLI, and only the exact value `1`), or the plan field `"others": "keep"` (v1 and v2) turns it on. Another author is anyone other than the plan's author, which is `Jubarte` when `--author` is left out; a missing or empty author is not yours. With the guard on, accepting or rejecting their change is `CHANGE_NOT_YOURS` (exit 3), whether the selection is a flag, `resolve_revisions`, or `existing_revisions` set to `accept` or `reject` on a file that holds their changes. Deleting, resolving, reopening or rewording their comment, and deleting your own thread that holds their reply, is `COMMENT_NOT_YOURS`. Resolving or reopening your own thread still marks the replies in it done or open again, as Word does; that erases nothing, so the guard allows it. The message and the next actions offer only the keeping moves: restore a deletion, strike an insertion, a comment, or a reply. Your own markup still settles. A style's recorded formatting is not one of these changes. Python and the npm CLI pass the same field. `EditPlan` and `PlanV2` gain `others` (a Rust struct literal names it; Python's field defaults to omitted).

- **`--keep-others` covers nested markup, comment bodies and typed edits** (#1892, #1890). `edit replace`, `rewrite`, `insert`, `delete`, `apply` and `edit template` take the flag, and `JUBARTE_KEEP_OTHERS=1` sets the same guard there; the environment variable is still read only by the CLI. A generated template carries `"others": "keep"`. Settling your own change is refused when that decision would remove another author's nested revision (`CHANGE_NOT_YOURS`, naming that revision) or delete their comment or a reply in its thread (`COMMENT_NOT_YOURS`). `existing_revisions` of `accept` or `reject` counts revisions inside comment bodies (`comments.rN` since #1706). A suggested restore or strike carries `--author` and, when the guard was on, `--keep-others`. With the guard off, a hint for another author's insertion offers strike before accept and reject, where the syntax has strike. A v2 plan whose `existing_revisions` is `refuse` names `"existing_revisions": "keep"` in either guard state. Resolving or reopening your own thread still marks the whole thread, replies included.

### Changed

- **Breaking: a one-shot `edit` or `add` that adds markup requires an author** (#1997, #1891). A write that signs tracked changes or comments needs `-a NAME` spelled or `JUBARTE_AUTHOR` set and non-empty; without either it is the typed usage refusal `error: AUTHOR_REQUIRED: sign the new markup with -a NAME (this document has markup by "X")` (exit 2, the parenthetical only when the document names authors), and nothing is written. The typed `edit replace`, `rewrite`, `insert`, `delete` and `apply` refuse the same way, and the Python CLI and the npm runner share the code, message and exit. A write that adds no markup (`--accept`/`--reject` only, `--editing-mode` with nothing left to sign, `accept`/`reject`) needs no author, and a plan still carries its own. The `; N change(s) by "X"` receipt detail stays; the `: no -a given` suffix and its warning are gone — there is no unsigned write left to warn about. `compare` and Markdown→DOCX keep their defaults.

- **Breaking: `Command::Accept` and `Command::Reject` store `output` as `Option<PathBuf>`** (#1888). `--in-place` leaves the struct field `None`. A Rust `match` that read `output` as a `PathBuf` now reads an `Option`. The CLI spelling `accept`/`reject -o INPUT --force` is unchanged: it still replaces the input. `parse_json` fills `args.output` with the input path and `args.output_format` with `docx` for `accept`/`reject --in-place`, including when the file is not named `.docx`.

- **`edit` and `add` name who signed the new markup** (#1891). A one-shot write, an `--out-dir` bundle, and a typed `edit replace`, `rewrite`, `insert`, `delete` or `apply` that adds tracked changes or comments says so: `wrote out.docx (docx, N bytes; 1 change by "GiantCo")`, `wrote DIR (N files: …; 1 change by "GiantCo")`, or `edit replace: changed; wrote out.docx; 1 change by "GiantCo"`. A comment is `1 comment by …`, and both are `2 changes, 1 comment by "GiantCo"`. Without `-a`, the line adds `: no -a given` and stderr warns `warning: no -a given; the new markup is signed "Jubarte"`. That warning stays on stderr for `--json` and `--jsonl`; the receipt on stdout is unchanged. `--editing-mode` publishes the clean copy, so the line counts no tracked change and counts a comment only when that copy keeps it. Recording a plan without `-a` warns `warning: no -a given; the plan signs its markup "Jubarte"`; applying the plan still treats its author as explicit. A write that adds no markup (`--accept`/`--reject` only, or editing mode with nothing left to sign) keeps the previous line and prints no warning. `-q` suppresses the warning with the rest of the status.

- **An `ANCHOR_NOT_FOUND` names the paragraphs the anchor does occur in** (#1889). The released message stays word for word as the prefix (#1013): `"Liability Cap" does not occur in the paragraph; it occurs in p53 ("10.2. Liability Cap. To the maximum extent…")`, and `paragraph index 182 does not exist in body (169 paragraphs)` gets the same addition when the operation carries an anchor. At most three paragraphs are named, then `and N more`; a candidate is exactly a paragraph the same anchor would match (Markdown marks and read-as normalization included), by the id the views print (`p53`, `header1.p1`). An anchor that occurs nowhere leaves the message byte for byte, and the JSON `op` event gains `"candidates": [{"at": "p53", "ctx": "…"}]`, absent when there are none. The edit is still refused — nothing is ever retargeted (#957). The legacy flags (`edit FILE -p WHERE --anchor …`) and v1 plans say it; the typed commands already had their own suggestions, and a redaction never names where its text lives.

- **`validate --baseline` names who signed the new markup, counts findings per change and ends with a verdict** (#1893). After the findings the report prints `new markup by: GiantCo (17 changes), G: GiantCo (1 change), Jubarte (1 comment)`: one entry per signing name, in first-appearance order, counting the logical changes `changes` lists and the comments separately. The summary needs no `--author` — only the `none of it is signed "Reviewing Counsel (GiantCo)"` line does — and it counts the markup the edit gained over the baseline, whichever `--original` names. More than one name adds `3 names sign the new markup; one session signs with one -a`; a revision that names no author counts as `<no author>`. A `FOREIGN_AUTHOR` finding is one per logical change, naming it as `changes` prints it (`r10+11 by GiantCo is new`), even when another author's text sits between the change's marks, and the `UNTRACKED_EDIT` of a paragraph whose only difference is that change is no longer reported beside it, so one mis-signed edit is one finding; that comparison is per story, so a tracked paragraph mark elsewhere in the document does not silence it. The input's resolved findings come before `new:`, and the last line is the verdict: `PASS: every difference from the baseline is yours and tracked` when the run had an `--author` to check the authorship against, `PASS: no new finding (authorship not checked: pass --author NAME)` when it had none, or `FAIL: 4 findings (3 changes, 1 comment); not ready to deliver`. The counts are objects, not findings: the distinct logical changes and the distinct comments the findings name, so one comment reworded and resolved is one comment, and a validity finding is no change at all. `--json` gains a last `{"authors": [...], "verdict": {...}}` line; the finding rows and the `resolved` line keep their shape, and the exit codes do not change. Breaking for the library: `validate::baselined_text` and `validate::baselined_json` take the run's `strict` flag, `validate::Baselined` gains `authors`, `author` and `verdict`, and `validate::Verdict` gains `authored`.

- **Breaking: deleting, resolving, reopening or rewording another author's comment is allowed unless `--keep-others` is set** (#1892, #1890). This undoes the #1853 default `COMMENT_NOT_YOURS` refusal. The report still notes each such change (`c0 is by "Bob"; their words were changed under their name`). Accepting or rejecting another author's tracked change stays allowed too. `--rewrite-others-comments` and `"rewrite_others_comments": true` stay accepted and are ignored; they do not turn the guard off. With the guard off, a refusal that names their change keeps the four routes and says what accepting or rejecting erases (`let it go (erases "Trainer 1 - AgentCo"'s deletion):`); `next_actions` lists restore or strike first and marks accept and reject `"destructive": true`. `EXISTING_REVISIONS`, in either guard state, and `--existing-revisions` beside `--plan`, say to add `"existing_revisions": "keep"` to the plan. Top-level `accept` and `reject` are unchanged. The comment default is breaking, so it ships after 0.16.0 with a minor bump.

### Removed

- **Breaking: `accept`/`reject -o INPUT` refuses; use `--in-place`** (#1998, #1888). The spelling deprecated in 0.16.0 is gone: `-o` naming the input is now a typed usage refusal `error: OUTPUT_ALIAS: the output 'FILE' is the same file as the input 'FILE'; use --in-place` (exit 2), with or without `--force`, and it fires before the input is read. `-o` naming a different file is unchanged. The Python CLI and the npm runner refuse the same way, with the same code and exit.

### Fixed

- **An orphan `w:vMerge` continuation is a plain cell** (#1993). A cell whose `tcPr` holds `<w:vMerge/>` (continue) with no `w:vMerge w:val="restart"` above it in the same grid column laid out as a swallowed continuation, hiding its text and leaving its row empty; Word 16 paints its text and keeps the row (`orphan_second_row`, `orphan_first_row`, checked 2026-10-31). `convert` now treats such a continuation as a restart of its own: its cell and text are laid out, and later continuations in that column merge into it, while a continuation under a live restart still hides its text (`proper_merge`). python-docx, PHPWord and HTML converters write these. Top-level and nested tables behave alike.

- **`jubarte changes` lists revisions inside comment bodies** (#1706). A `w:ins`, `w:del` or formatting change in `word/comments.xml` is `comments.r9`, the same story prefix as `footnotes.r1` and `header1.r1`. The text listing, the JSON records, Python `list_changes` and WASM `listChanges` / `changesText` all print it: they share `list_logical_changes` and `changes_text`. `accept` and `reject --id comments.r9` settle that change. `r9` does not name it when only the comment holds `w:id` 9, and `c0.r9` is not a change id (`c0` already names the comment). The same `w:id` in the body stays `r9`. A selection that leaves some says they are `comments.r…`. `commentsExtended` is not a revision part. Footnotes were already listed.

- **A cell reads only its own `tcPr`** (#1729). Every converter reader of a cell's property block (`parse_tc_border_spec`, `table_block`'s `hideMark` and explicit-shading reads, `cell_pad_h`, `cell_pad_tb`, `cell_span`, `cell_pref_width`, `cell_without_width`, `cell_is_deleted`, `cell_valign_is`, `cell_vertical`, `cell_nowrap`, `cell_fill`) now takes the `tcPr` as a direct child (`cell_pr`), as Docxodus and Open-Xml-PowerTools do. A cell without a `tcPr` that holds a nested table no longer borrows the nested first cell's gridSpan, vMerge, tcBorders, shading (including the explicit-fill suppression of a table-style band), tcMar, noWrap, textDirection, vAlign, hideMark or cellDel. Word writes a `tcPr` on every cell it saves, so its own files never hit this; python-docx, PHPWord and HTML converter output can. Lookups inside the cell's own `tcPr` keep their `tcPrChange` (prior-state) filters.
- The wasm32 build no longer warns that `edit::record::plan_target` is never used: only the native publish path compiles it. CI's wasm32 clippy now also denies the engine's own warnings (#1700).
- **Accept, reject and identical self-compare repair inherited schema-invalid shapes** (#1717). Standalone `accept_revisions` / `reject_revisions` (CLI `jubarte accept` / `jubarte reject`, Python and WASM) and identical-input `compare_documents` now run the same package validity sweep comparison uses since #1712: empty `w:shd/@w:fill`, `w:gridCol/@w:type` and out-of-order `w:tblCellMar` / `w:tcMar` children are normalized in every story part the resolution writes, including `tcPrChange`, `tblGridChange` and `tblPrChange` snapshots Reject All restores from an older redline. Parts a resolution never touches (the glossary, #1996) stay byte for byte; a clean source stays byte-identical on self-compare.

- **Reject All removes a cell whose saved properties say it was inserted** (#1724). A `w:cellIns` on the live `w:tcPr`, or on the `w:tcPr` a `w:tcPrChange` restores, drops the cell. `CT_TcPrInner` can hold that mark, so the check reads both copies; reading only the rebuilt `w:tcPr` would keep a live insertion whose saved copy has none (`gb2`). A live `w:cellDel` still keeps the cell and the saved shading (`c2`, `gb1`). A `w:cellMerge` with `w:vMerge="rest"` and `w:vMergeOrig="cont"` keeps the cell, restores the saved properties and drops the cell's paragraphs and nested tables, leaving an empty paragraph and no `w:vMerge` (`c3`). Word 16's grid autofit is left alone.

- **Reject All keeps a row's conflict mark as an ordinary revision** (#1723). Restoring a `trPrChange` carries the live `w14:conflictIns` and `w14:conflictDel`, which the saved `CT_TrPrBase` cannot hold. After accept, a mark whose parent is that `w:trPr` becomes a plain `w:ins` or `w:del`, keeping its own id, author and date. Word 16 keeps `r0` and `r1` tracked that way. A conflict mark on a run or a paragraph mark, and a row mark with no `w:` revision beside it, stays as it was.

- **Reject All joins a paragraph whose inserted mark also changed formatting** (#1722). Restoring an `rPrChange` on a paragraph mark (`w:pPr/w:rPr`) keeps the live `w:ins`, `w:del`, `w:moveFrom` and `w:moveTo`, in that order, and drops a saved copy of the same mark. Word 16 joins `ga1`, `ga3` and `h1` into one paragraph, and `n0` into one paragraph and one section. A deleted mark (`ga2`) still leaves the two paragraphs apart. A run's `rPrChange` is unchanged.

- **A package with no styles part takes Word 16's built-in defaults** (#1725). Conversion seeded a styles-less package with Calibri 11 / line 276 / after 200 twips; Word 16 lays it out on the same stock defaults it gives a styles part without `pPrDefault`/`rPrDefault` — Aptos 12pt, kern 1pt, after 160, line 278 — so a `word/document.xml`-only package now renders exactly like the same package with an empty `<w:styles/>`.

- Header and footer comparisons neutralize unresolved relationship attributes on both inputs before diffing, preserving content deletions and accept/reject round trips in Word and PowerTools modes (#1709). A story that cannot be read, carried or redlined now returns a typed `StoryDiffError` naming the part, instead of silently retaining the original without revision marks.

- **A pure accept or reject no longer hides an error from its resolved lines** (#1702). The Python CLI computes that text before printing and catches only `JubarteError`, the engine refusing the text. That refusal still prints the changed view once and exits 0. Any other error reaches the CLI's handler: `error: …` on stderr and exit 1, with nothing from the resolved view on stdout.

- Accept All and mirrored Reject All no longer panic while joining paragraphs containing nested smart tags, custom XML, content controls, fields, hyperlinks or direction wrappers (#1704). The paragraph-content probe walks through those wrappers and preserves unknown children, including malformed paragraph-level ruby (#1711). `validate` reports direct `w:p > w:ruby` as `RUBY_OUTSIDE_RUN`, a schema error with no automatic repair; Word behavior remains unverified.

- **A non-finite measure reads as no measure** (#1737). `NaN`, `inf`, `infinity` and a value that overflows `f32` (`1e40`) on a document length or width are read the same way as a token that does not parse. `tcW`, `tblW` and a header table's `gridCol` then keep the text on the page where an unparsable width puts it. A finite number, including a negative and a finite exponent (`1e2`), is unchanged. Integer attributes are unchanged.

- **A patch cannot add a public item cargo-semver-checks does not see** (#1758). `scripts/api_snapshot.py --added OLD NEW` prints each public item the new listing adds, as `namespace path` (Rust's type, value and macro namespaces, so a new `something!` stays distinct from `fn something`), and exits 1 when there is one. A public glob re-export counts as the concrete items it makes nameable (`pub use b::*` that newly exposes `b::second` as `jubarte::second`). An impl's generic parameter names are positional (`Deserialize<'0>`, `From<T0>`), so renaming `'de` to `'input` keeps the same path; a new trait impl is still an addition. For a patch, `bump_gate` runs that check after cargo-semver-checks. The baseline is always a listing built from tag `v<prev>` in a throwaway worktree with this checkout's script. A committed `docs/api` file, four-column or otherwise, is never that baseline. The gate compares the tag listing with a listing of HEAD. The worktree and the HEAD listing are removed on the way out, including when the patch is refused or the build fails (bash 3.2 and `set -e` included). Each added path is named, and the gate says to release the next minor. A minor or a major skips the check. Equal listings pass: a method body does not change one.

- **PDF text boxes no longer invent an outline** (#1979). A WPS text box without a line or a nonzero style line reference stays unoutlined. Explicit no-fill lines suppress outlines, style references retain their line colour and width, and VML keeps its existing default stroke.

- **The release matrix test rejects an x86-64 build in the ARM64 binary row** (#1748). `scripts/test_release_workflow.py` checks each binaries-matrix label (`linux-x86_64`, `linux-aarch64`, `macos-x86_64`, `macos-aarch64`, `windows-x86_64`) against its Rust target triple, GitHub runner and wheel platforms. A copy of `.github/workflows/release.yml` whose `linux-aarch64` target is `x86_64-unknown-linux-gnu` fails that check.

- Inline DrawingML pictures in table cells, headers, footers and horizontal text-box paragraphs now occupy atomic boxes in the text line, so adjacent text wraps around their width and height. Header/footer splices preserve breaks, field state and unbound picture height; cell alignment uses picture widths. Text boxes size to pictures, clip overflowing picture lines and restore the host paragraph's first line and picture row (#1975).

- **Split deleted tables preserve stored grids for unprobed geometry** (#1913). Skip preferred-width resolution for row width or indent exceptions, tracked cell or row properties, horizontal or vertical merges, and any nonzero cell margins under percentage table widths. These are conservative guards, not new claims about Word behavior. Keep resolved totals wider than the section text area, as demonstrated by Word’s redline of `510bcf9432` (9,571 twips in a 9,358-twip text area).

- **Split deleted tables resolve supported preferred cell widths** (#1913). Reconcile agreeing positive `dxa` cell widths in both autofit and fixed tables, including fitting stored grids; honor exact `dxa` expansion and percentage widths with explicit zero margins. Preserve live grids and cell preferences. Keep grids requiring content measurement, unknown margins, spacing, conflicting preferences or fractional allocation unchanged; full Word layout resolution remains open. Twelve constructed Word 16.115 probes establish the supported cases.

- The Ring-2 release proof stages legacy source names from the bench pool, verifies renamed aliases against the rename ledger, and reports missing originals without re-blessing. Sweeps refuse baselined stems that were not run and never report their findings as fixed (#1791).

- The release formatting gate checks all four Rust workspaces, including WASM, Python and in-process bindings; existing WASM formatting drift is corrected (#1760).

- PowerTools-faithful comparisons, `compare --no-paragraph-merge`, and custom settings with `merge_replaced_paragraphs=false` no longer keep A-only headers and footers live on the final section (#1837). A-only detection uses both kind and default/first/even slot; stories referenced solely by the final section are deleted too. Accept removes their text and Reject restores it. The existing PAGE-field workaround moves an eligible footer off deleted breaks to avoid the Word crash; Reject restores that footer on the last section only. Stripping a final reference cannot be undone on Reject: a different story on an earlier live section can be inherited instead. Docxodus e605d62 accepts s548 to one section with no header/footer text; our accepted empty footer reference deliberately differs. Shared slots and stories on live earlier sections retain faithful behavior.

- Identical-input comparisons, including inputs that become identical after preparation, run validity repairs only (#1796). After the existing Strict translation and optional source-revision acceptance, they supply missing `cnfStyle` bitmasks and required shading/colour values; remove invalid unqualified attributes, empty shading fills, `gridCol/@w:type`, numbering-symbol highlights and invalid paragraph/text ids; repair Strict shape namespaces and wp14 percentage values, fractional integer measures, illegal hyperlink/field revision nesting, schema-required property order, missing compatibility-prefix bindings and invalid/duplicate DrawingML ids. Valid unit-bearing measures, Transitional decimal twips and table widths, text kinds inside deletions, alignment, DrawingML percentages, VML ids, empty paragraph properties, comments and untouched part bytes survive. Identical exits neither strip explicit `nextPage` nor add `tblLook`, comparison defaults or final-mark cleanup. Full-comparison order and output are unchanged. The maintainer confirmed that both Strict self-compare outputs open cleanly in Word: missing `cnfStyle/@w:val` is a schema defect, not a Word-fatal shape.

- Comparisons preserve unique VML ids and spids, repair collisions in a shared per-part space, and keep type, control, regroup, fragment and OLE references linked to their source definitions. AlternateContent Fallback ids stay untouched. `validate` warns only when a shape type names its own id or no shapetype in the part, excluding Fallback copies; repeated ids alone are allowed. The maintainer confirmed all six before/after bench probes open cleanly in Word, including the colliding pre-fix ids (#1800).

- **An inline picture is a box in the line** (#1840). A body flow picture (a simple blip, not a group, a float, VML, a text box, a cell, a header or footer, a picture bullet, or a deleted drawing) enters the run stream as one unbreakable box, its width the picture's and its height standing on the baseline. Breaking, justification, alignment, list markers, deleted text, page breaks, shading, borders and the revision bar then follow that line. A picture-only line still shares its row with a following inline group, past a trailing tab. Groups, VML, floats and the other shapes above stay on the old path.

- **Split-off deleted tables use their own rows' grid** (#1913, step 1). In Word-mode comparisons, the deleted runs split out by #1912 coalesce unused column edges and rewrite cell spans and row omissions, preserving retained cell widths. Trailing columns every row explicitly omits are dropped, as in Word's saved redlines. Layout-resolved widths (step 2) remain open pending a Word probe.

- Anchor-not-found candidates use retryable source paragraph IDs after revision settlement, show the matched text in grapheme-safe excerpts, and stay out of comment-text scope errors. Repeated failures reuse candidate scans, and literal/plain anchor successes skip normalized searches (#1889).

- **`--in-place` replaces only the input, and only as a DOCX** (#1888). `accept` and `reject` of a DOCX named `contract.json` or `notes.md` no longer overwrite it with a JSON snapshot or Markdown. `edit --in-place --pdf` (and `--png`, and a failed-op plan) no longer overwrites an existing companion unless `--force` is also given. Companions are committed before the input, so a companion that cannot be committed leaves the input byte-identical. `accept` and `reject` recheck the source at the commit boundary and refuse with `SOURCE_CHANGED` when it changed; the receipt hash is the hash of the bytes replaced. A symlinked input is replaced through the link: the link stays, and the target keeps its mode. Recording `--plan` with `--in-place` is refused, the same way as `--output`. `parse_json` reports the resolved input path and `output_format` `docx` for `accept`/`reject --in-place`. The Python CLI and the Node runner (`jubarte-wasm/cli`) follow the same rules: companions still need `--force` and are written before the input, `accept`/`reject` recheck the source, and a symlink is followed. The checked-in `jubarte-wasm/npm` package is unchanged; `npx jubarte-redlines --in-place` keeps refusing the flag until `jubarte-wasm/build-npm.sh` rebuilds it from a clean tree.

- Incomplete-change refusals name runnable whole-change retries for edit flags (`-p r10+11 --accept` / `--reject`), typed edits (`--accept-change r10+11` / `--reject-change r10+11`), and top-level `accept` / `reject` (`--id r10+11`). One-shot selection errors omit the plan-only prefix, including when recording flags into a plan with visible rejects. JSON plans keep their released wording, and `--strike` still allows a single mark (#1895).

- One-shot `edit` and `add` accept `--occurrence N` after `--anchor` in each `-p` group. Ambiguous anchors show numbered context windows and flag-based guidance; plans retain their released wording. `edit replace --find` lists matching paragraphs with their match counts and suggests `--at` or `--occurrence` (#1894).

- `scripts/release.sh` step 12 accepts a `--version` that carries the build commit (`jubarte 0.16.0+00b32d2`, #1924), and still refuses a `.dirty` build (#1954).

- `scripts/release.sh` compiles the app before the tag (#1816). A new app gate between steps 3 and 4 lays out the engine at HEAD with the app at `jubarte-app/` inside it — `git archive HEAD` of both checkouts in a fresh `mktemp -d`, so `src-tauri`'s `path = "../.."` resolves to the engine — and runs the app's own `cargo test`. A compile error or a failing test stops the release before the tag and every publish; 0.14.0 shipped before anything compiled the app against the released engine. The gate never passes `--locked`/`--frozen`: the app's lock on main records the previous release's versions and the engine's version moves every release, so a stale lock is normal and cargo updates only what the path dependency's new version requires, inside the throwaway copy only. `--skip-gates` is the only skip — an existing tag proves no app-gate pass. A failed gate prints its cargo log tail and removes the layout (`JUBARTE_APP_GATE_KEEP=1` keeps it; an interrupt always removes it). The checklist row that was a `before|you|` reminder is now a `3|auto|` row, and `docs/VERSIONING.md` documents the gate.

- The release's quality gate measures the previous release, run on the same corpus and machine, instead of stored baselines that measure nothing (#1797). `tools/parity_ladder.py sweep` gains `--against BIN`: it sweeps the previous release's binary on the same pairs and reports only findings the candidate adds — the blessed TSV stays the day-to-day ratchet. New `scripts/criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE]` benches the previous release's tag and the candidate (a directory or revision — the release's own tag when resuming, HEAD before it) interleaved A B A B into one `CRITERION_HOME` under the release's state dir (never `/tmp`), keeps the six `--load-baseline` comparisons as printed context, and reads each case's verdict from criterion's saved estimates (`<group>/<bench>/<run>/estimates.json`, mean point estimate — never the `change:` text, whose sign is a Unicode minus): per case delta = mean(b1,b2)/mean(a1,a2) − 1 against self-noise max(|a2/a1−1|, |b2/b1−1|) — REGRESSION past both the >5% bar and that noise, UNDECIDABLE (rerun on a quieter machine) when a self-noise above 5% swallows it, the gate failing on either as well as on a failed cargo call, a missing estimates file, or a case benched on one side only. A dirty or wrong-rev reused worktree is discarded and recreated. The checklist's quality-gate row names the corpus (`--corpus` a staged ladder_corpus, the bench's `corpus/word` via `uv run bench fixtures download` or the previous release's own `gates/ladder_corpus`), finds the previous release from `v<ver>^` when the tag exists and HEAD before it, and `docs/VERSIONING.md` follows.

- `scripts/gen_docs.sh` rebuilds the Python package whenever the venv's interpreter cannot find `jubarte_redlines` (`importlib.util.find_spec`, which does not execute the package, so an installed copy whose import raises is left for the runner to fail on, not rebuilt), not only when the interpreter itself is missing. Its `python -m jubarte_redlines` fallback runner now reaches `gen_cli_docs.py`: `--runner` repeats once per argv word (`--runner=WORD`) instead of an argv list that argparse's `nargs="+"` ended at `-m`, and each word reaches the subprocess verbatim — paths with spaces, tabs or backslashes need no quoting (#1810).

- `scripts/lane_validity.py` compares validator errors by signature — (error id, part family, description with revision element names folded — `w:ins`/`w:del`, and a bare `ins`/`del` that names an element — while a quoted `'ins'`/`'del'` attribute name or value stays itself, element at the xpath's end) — so a renamed part (`redlineB_header3.xml`) still meets its source's `header3.xml`. It validates the sample's Word redline (the `docx` column) and reports shared signatures as `WORD` with the count on both sides (`ours N / word M`), since the exemption is per kind, names a source or Word redline the validator cannot open as `UNREADABLE SOURCE` — references are validated even when the redline is clean, so the unreadable count is true — and a redline the validator cannot produce or open (`ABSENT`, `OPEN_FAILED`, `EXIT`) is always `OWN` and exits 1, whatever the references say (#1798).

- **An emptied content control shows Word's placeholder** (#1703). When Accept All or Reject All empties a `w:sdt` (its words go, the control stays), the resolver marks it as Word 16 does: `<w:showingPlcHdr/>` goes in the control's `w:sdtPr` and a run `<w:r><w:t xml:space="preserve">     </w:t></w:r>` (five spaces) fills it — inside the control's paragraph for a block control, directly in `w:sdtContent` for an inline one. The stamp distinguishes a control this resolution emptied from an authored empty one, so resolving twice does not duplicate the placeholder, and a control whose author left it empty keeps the authored shape. Controls in every story (body, cells, headers, footers, notes) are covered; building-block gallery controls (`docPartObj`/`docPartList`) are not marked, matching Word.
- **A content control whose whole content changes stays one control** (#2004). When no word inside a `w:sdt` survives (a `fill_control` edit, or a compare that replaces all of a control's text), the redline wrote two `w:sdt` with one `w:id`: the deletion in one, the insertion in the other. Accept All and Reject All then each left an emptied duplicate. The deletion now joins the insertion in one control, as when Word types into it: inline or block-level, matched by exact `w:id` (tag or alias only where a side has no id) at the same nesting, and only when the two are adjacent. A `fill_control` redline rejects back to the source again.
- **Reject All keeps a block control's paragraphs inside it** (#2006). When a paragraph mark joined two paragraphs inside a block `w:sdt`, a paragraph with no `w:r` run (an empty paragraph, or one holding only an equation) was rebuilt outside the control. Word 16 keeps both inside, and so does jubarte now. Our own Word-mode compare writes this shape, so before the fix, Reject All of a compare could move A's equation out of its control.

## [0.16.0] - 2026-10-31

> **Summary.** Breaking: `changes --json` and `comments --json` emit records only on stdout; their `input_validity` status moved to one JSON line on stderr, which `-q` suppresses (#1925). New: `--version` and the `--help` header carry the build's short Git commit (#1924). `validate` resolves relationship attributes by namespace (#1937), checks every relationships part for missing targets (`MISSING_REL_TARGET`, #1938) and reports a nameless move range (`MOVE_RANGE_NAME_MISSING`, #1939). A PDF table whose rows reach past its grid takes Word's column widths (#1914). A keep-path `rewrite` marks one change per phrase (#1923). An edit can delete a paragraph's first words and insert at its start, with no panic (#1922). Inserted-only words stay ahead of one deleted table, as Word writes them (#1926). On the release's two 600-item samples, scored against Word's own output: redlines mean 74.36 (Docxodus 64.92), counting as 0 the 7 redlines Word 16.115 crashes opening; PDFs mean 79.19 (LibreOffice 54.61) with no failed document. LibreOffice aborts on 2 of the PDF sample's documents, which score 0 in its mean, and Word's own redline crashes Word on all 7 of those redline pairs. The Miri full suite was not run for this release.
>
> **Docs.** CHANGELOG.md gained the 0.16.0 section. docs/MARKDOWN.md and jubarte-wasm/npm/README.fragment.md (and the generated jubarte-wasm/npm/README.md) say that changes/comments --json put input_validity on stderr. docs/WORD_DIFFERENCES.md records the single-table order (difference 14/16) and the past-grid table widths (difference 17). The generated docs/rust.md, docs/python.md and docs/flags_datetime.md follow the CLI help (build version and the new validate codes).

### Changed

- `changes --json` and `comments --json` now emit records only on stdout: the `input_validity` status moved from stdout to one JSON line on stderr, suppressed by `-q` (#1925). The Python CLI and npm CLI's `changes --json` follow the same rule.

### Fixed

- `validate` checks every relationships part, including `_rels/.rels`, for missing internal targets (`MISSING_REL_TARGET`, #1938). An unused relationship produces a non-fatal warning, which `--repair` does not fix: Word has not been probed for this case. A relationship used by markup retains its Word-fatal error and is reported once even when used repeatedly.

- **A PDF table whose rows reach past its grid takes Word's column widths** (#1914). Word lays out again the columns of a table whose cells run past its saved grid. An autofit table with a declared width now shares it by its cells' content, a cell's own width first, in twips or as a percentage. Without a declared width, each cell keeps its width in twips, and a column without one takes its line. Cells keep their own widths over a grid Word did not save, too. When the declared width is too narrow for the cells' widths and their longest words, columns without a width get their longest word and the cells with one share the rest; when it is narrower than every longest word, each column gets its longest word and the table outgrows its declared width, as Word's does. A column only `gridBefore` or `gridAfter` holds takes no width. A fixed table starts each column past the grid at 18pt, then fits them all to its declared width. Before, each extra column took 80pt, or an even share of a one-column grid. On Word 16.115 probes, the autofit columns and the fixed ones a table widens fall within 0.3pt of Word's; a fixed table narrower than its columns comes within 5pt. Word's own redline of one bench pair, a 14-column grid under 18-cell rows, now converts to 14 pages instead of 16 (Word prints 13), and its score goes from 49.25 to 52.24. Of jubarte's 72 redlines of the bench pairs with two or more tables, one renders differently, at the same score and page count. Word still differs where a fixed table is narrower than its columns and where an autofit table keeps a grid Word did not save (docs/WORD_DIFFERENCES.md, difference 17).

- `validate` checks every attribute in the officeDocument relationships namespace, regardless of prefix, quoting, or local name, and no longer mistakes raw XML text for relationship attributes (#1937). Dangling `r:id`, `r:embed` and `r:link` references keep their Word-fatal finding; the other relationship names are repairable warnings until Word is probed. Existing repair drops the dangling attributes.

- **A `rewrite` under `existing_revisions: keep` marks one change per phrase** (#1923). It writes the same marks as a rewrite that refused the existing revisions. A hunk of deleted words and inserted words was written as one substitution per deleted word, paired by position. The word diff now joins a whitespace-separated hunk into one deletion followed by one insertion, and a keep-path rewrite of a paragraph that holds no tracked change of its own emits the comparer's hunks. An anchored `replace` is unchanged. A tracked change already in the paragraph stays on the word diff, so it is not marked again.

- CLI `--version` and the `--help` header identify the build with the short Git commit as SemVer build metadata, plus `.dirty` for a changed tree. Builds without Git or a checkout keep the Cargo version alone; Python and npm package versions stay unchanged (#1924).

- **An edit can delete or replace a paragraph's first words and insert at its start** (#1922). Expected-text checks now compose touching ranges in the same order as the writers and refuse invalid ranges instead of panicking. A `delete` that starts at the paragraph's first character and an insertion at `position: "start"` (or one that ends at its last character and an insertion at `position: "end"`) are written as one whole substitution, so deleting every run keeps the inserted text in Accept All; a `replace` beside an edge insert keeps writing only the words it changed. Two differently formatted insertions merged into one such delete are refused as `OVERLAPPING_EDITS`.

- **`jubarte validate` reports a move range start with no `w:name`** (#1939). `w:moveFromRangeStart` and `w:moveToRangeStart` require `w:name`. A missing or empty name never entered the per-name counts, so a nameless source range with no destination passed with no finding. It is now `MOVE_RANGE_NAME_MISSING`, a warning. Word's reaction to a nameless range is unverified, so the finding is not Word-fatal. A named range whose source and destination counts differ is still `MOVE_RANGE_COUNT_MISMATCH`.

- Inserted words that are a paragraph's only content stay ahead of a single whole-deleted table when the paragraph after it keeps no words, as Word writes them (`employee_directory_table` → `employee_review_john_smith_suggesting_insertions`, `meeting_agenda_table` → `meeting_minutes_id_paraid_overflow`). The keep-in-place rule of #1826 now applies only to a paragraph between two deleted tables. Words before two deleted tables, words beside the original's deleted text, and words whose following paragraph already keeps text, still move past the tables (#1926).

<!-- changelog-skip: #1955 tooling-only: release.sh step 12 reads a --version with the build commit -->

## [0.15.0] - 2026-10-30

> **Summary.** Breaking: `jubarte::debug::Check` gains `Tables` (#1817). New: `validate --original --author` checks comments (`FOREIGN_COMMENT`, #1846); `strike` and `restore` take paragraph marks and multi-mark changes (#1848, #1849); `validate` reports `DELETED_SECTION_PAGE_FIELD` and `INSERTED_TEXT_BEFORE_DELETED_TABLE`. Word mode pairs, splits and meshes the tables of a replaced region as Word Compare does (#1817); `edit` refuses to delete, resolve or reopen another author's comment; comments and edits no longer renumber lists or restyle what they did not touch (#1850, #1861, #1868); Word opens the redlines whose deleted section breaks carried a page-numbered footer (#1833). jubarte builds against oxml-opc 0.14.4, which 0.13.0 and 0.14.0 cannot. On the release's two 600-item samples, scored against Word's own output: redlines mean 74.19 (Docxodus 64.92), counting as 0 the 7 redlines Word 16.115 crashes opening; PDFs mean 79.19 (LibreOffice 54.61) with no failed document. LibreOffice aborts on 2 of the PDF sample's documents, which score 0 in its mean, and Word's own redline crashes Word on all 7 of those redline pairs. The keep-in-place rule of #1826 also reorders two single-table regions of the parity ladder away from Word's order, though both still accept and reject exactly (#1926). The Miri full suite was not run for this release.
>
> **Docs.** README.md (and the generated README.crates.md, jubarte-python/README.md, jubarte-wasm/npm/README.md, jubarte-wasm/cli/README.md) says validate --original also checks comments (FOREIGN_COMMENT). CHANGELOG.md gained the 0.15.0 section; docs/WORD_DIFFERENCES.md records the table pairing, split, mesh, footer-move and paragraph-between-tables differences; docs/MARKDOWN.md, docs/python.md, docs/rust.md and docs/flags_datetime.md cover the new strike/restore and validate behaviour; KNOWN_ISSUES.md, skills/jubarte-documents/SKILL.md and examples/adoption/12-comments were updated.

### Breaking

- **`jubarte::debug::Check` gains `Tables`** (#1817). A Rust `match` on `Check` without a wildcard arm needs one for it.

### Added

- **`validate --original --author NAME` checks comments too** (#1846). It read only revisions, so a counterparty comment you deleted or resolved, or one `add` signed "Jubarte" because `--author` was left out, passed with "no findings". Each is now a `FOREIGN_COMMENT` error in the `edits` set: another author's comment (or reply) that is gone, reworded, resolved or reopened, and a new comment or reply not signed NAME. Every message says what to do instead (`c0 by "Counterparty Counsel" was resolved: reply to their comment instead of resolving it`). Comments pair by id, then by author, date and text, so a file whose comment ids another tool renumbered still pairs. NAME's own comments change freely, and resolving NAME's own thread does not count against the replies in it. The Python (`Document.audit_tracked`) and WASM (`auditTracked`) audits report the same findings.
- **`strike` and `restore` take a paragraph mark** (#1848). When another author split a paragraph, striking their inserted mark puts your tracked deletion beside their insertion in the mark, as Word writes it, and Accept All joins the paragraphs. When they joined two paragraphs, restoring their deleted mark puts an empty paragraph of yours right after theirs, with its properties and mark formatting, and Accept All splits them again. Reject All gives the original either way. A comment (`--content`) sits on the last word before the break. Both used to be refused, with advice that answered the change without a trace (`reject rNN`) or struck the paragraph's own words too (`delete paragraph pN`).
- **`strike` and `restore` take a change of several marks** (#1849). `jubarte changes` lists one author's touching marks under one id (`r10+11`, a replacement's `r11+12`), but a plan's `strike`/`restore` refused that id (`strike takes one revision`) and `-p r11+12 --strike --content` refused the comment (`--content comments one revision`). A strike now strikes every insertion mark of the change and a restore restores every deletion mark, each mark checked as it would be alone, under one comment from the first answered word to the last. A replacement's other half stays as it is, and the edit says so (`note: op-1: struck r12; r11 is its deletion: restore it too to reject the whole change`). A change with no mark of the kind is refused (`r12+13 holds no insertion: restore it instead`), and so is one holding your own mark, naming it. `-p r11+12 --reject --visible --content TEXT` gives a replacement its restore and its strike, the comment on the first in reading order. A strike or restore's note (`already struck` included) now prints as a `note:` line.
- **`jubarte debug FILE -c tables` lists every table's grid, rows and cells** (#1817). Each cell shows its width, span and vertical merge, its revision marks and the old properties its `tcPrChange` records, then its text; nested tables are indented under their cell, and each table shows its old grid and `tblPrChange`. `debug` is the maintainer tool hidden from `jubarte --help`.
- **`jubarte validate` reports `DELETED_SECTION_PAGE_FIELD`** (#1833). This is a warning on a header or footer that a deleted section break references, whose closing paragraph holds a PAGE or NUMPAGES field. Word 16.115 crashed opening 9 of the 11 release-sample redlines with that shape, whatever the story's marks said. It is a warning rather than an error, because Word opened the other 2.
- **`jubarte validate` reports `INSERTED_TEXT_BEFORE_DELETED_TABLE`** (#1851). This is a Ring-1 warning lead on a body paragraph whose mark is deleted and whose own `w:ins` runs hold every word Accept All keeps of it, followed right away by a whole-deleted table, then the paragraph those words join. Word 16.115 crashed opening 6 of jubarte's 16 release-sample redlines with that shape. The compare pass below shares the rule, so the two cannot disagree.

### Changed

- **`strike` and `restore` refusals offer a comment, never a silent reject** (#1848). A move, a formatting change, a field, a section-break mark and the other shapes they cannot answer visibly now end in `comment on it instead`. They used to advise `reject … instead`, which the reviewer must not do to another author's change. A deletion with no text says `nothing to restore`. Your own revision still says `reject it instead`.
- **`edit` refuses to delete, resolve or reopen another author's comment** (#1846). Rewording one was already refused (`COMMENT_NOT_YOURS`), but `-p c5 --delete` / `--resolve` (`delete_comment`, `resolve_comment`) went through on anyone's comment. Deleting your own comment also deleted another author's replies in its thread without a word. All of these are now refused with `COMMENT_NOT_YOURS`. The refusal says what to do instead (reply to their comment, or resolve your own thread), and `--rewrite-others-comments` (`"rewrite_others_comments": true`) still lets the operation through, with one report note per comment it changed. A plan that resolved or deleted another author's comment on purpose now needs that opt-in. `capabilities` names it on the `delete_comment` and `resolve_comment` rows.
- **Word mode pairs the tables of a replaced region in order, as Word does** (#1817). Where the revision replaces a stretch of the original and both stretches hold tables, Word's Compare pairs the original's first table with the revision's first, the second with the second, and so on, and diffs each pair row by row whatever their text; around each pair it writes the revision's paragraphs before the original's. The default mode wrote the revision's tables inserted whole and the original's deleted whole, and Word 16.115 crashed opening one such bench redline; it opens now. On the 72 bench pairs whose documents both hold two or more tables, jubarte's row marks per table now match Word's in 61, against 34 before. One redline Word opened before now crashes Word 16.115 the way Word's own redline of that pair does.
- **Word mode splits a paired table where Word does** (#1817). Word keeps a table's properties on each row: kept and inserted rows carry the revision's, deleted rows the original's, and Word starts a new table wherever adjacent rows differ in table style (matched by name) or layout type. Where the two sides of a paired table differ that way, each run of the original's deleted rows is now a table of its own with the original's properties and grid, as in Word's redlines. Before, those rows sat in the revision's grid: in one bench pair, one-cell rows in a four-column grid ran 70 pages against Word's 40. On the 72 bench pairs, every table's row marks now match Word's in 36, against 15 before. A style without a name counts only on the original's side: Word drops the revision's unnamed styles from its redline. One more redline Word opened before now crashes Word 16.115 the way Word's own redline of that pair does.
- **Word mode meshes a row whose cells were repartitioned, as Word does** (#1817). When a cell is split or merged, a column is added or deleted, or a cell boundary moves, the default mode now pairs the two rows instead of writing the original as a deleted row and the revision as an inserted one. The row keeps one cell count: the revision's extra cell holds inserted text and the original's deleted text. Every cell records its properties, and no span overflows its side's grid. Word 16.115 would not open the deleted rows of the old shape in two fixtures. Accepting a merge, or rejecting a split, now keeps an empty extra cell, as Word's own redline does; `--mode powertools` keeps complete rows and restores both tables exactly (docs/WORD_DIFFERENCES.md).

### Fixed

- **An edit dry run's `next:` line names the file it writes** (#1847). A dry run that named no output printed a `next:` command without one, so the command run as printed wrote `<stem>_modified.docx` beside the input, while the file the caller meant to update stayed as it was. The line now spells the default as `-o <stem>_modified.docx`, after a note that the input stays as it is. An `X_modified.docx` input, updated in place, gets no `-o` (naming the input as the output is refused), and its note says it is updated in place.
- **A comment no longer renumbers the document's lists** (#1850). When a document held several identical list definitions, as Word writes them, a redline (`compare`, and the redline of `add` and `edit`) matched each of the revised side's definitions to the first identical one. Every list then looked like a clash and moved to a fresh `numId`, each of its paragraphs carrying a formatting change back to the old one. Adding one comment to `tests/fixtures/redline/original.docx` listed 9 such changes. The lists that moved also shared one definition, so their numbering ran on from one list into the next. In Word 16.115, the fixture's one-item lists, each "1.", showed "1." struck and "2." to "10." inserted, and Accept All kept those numbers. A definition now matches its own id first, and a list drawn the same on both sides keeps its `numId`.
- **A comment no longer lists changes it did not make** (#1861). On a file with no tracked changes, `add --comment` and an `edit` plan that only comments, replies, resolves, rewords or deletes comments went through the comparer, and its normalizations came back as changes: a heading's spacing with its `lineRule` written out, the stylesheet merged (Normal took another font and size), a short paragraph before a page break deleted and inserted again. After #1850, 18 of 245 clean bench documents still listed such changes. These plans are now written straight into the document, as `--existing-revisions keep` writes them, and list none.
- **An edit no longer restyles what it did not touch** (#1868). On a file with no tracked changes, `edit` and `add` build the redline by comparing the source with the edited copy, and the comparer applied Word Compare's normalizations to the whole document. It merged the stylesheet and renamed its ids (`style20` became `TableContents`, and Normal went from Times New Roman 10pt to Liberation Serif 12pt), and turned a table of contents' 133 hyperlinks into styled text. It stripped `spacing line=276` from every paragraph and dropped `jc=left`, `rtl=0`, undefined `pStyle`s, the page background and the section's vertical alignment, page borders and line numbering. It wrote `lineRule`, 10-twip table cell margins, a table look, a page size and a numbering part the source never had, and added empty notes, settings and theme parts. It redrew the grid of a table whose rows hold more cells than the grid declares. So Accept All of the redline was not the document the plan asked for: its run formatting differed from the clean copy on 258 of 299 clean bench documents, and its rendered pages on 21. The comparer now leaves what the plan did not touch as the source has it, and keeps the plan's own content as written. 6 documents still differ in their runs: empty runs and an empty comments part are dropped, `mc:AlternateContent` is reduced to its Choice, and rowless tables are removed. 4 differ in their pages. Three come from comparer behaviour `compare` shares: row exceptions dropped after the inputs' pre-accept joined two tables (#1869, #1870), a footnote mark's run properties dropped (#1871), and saved table grids re-fitted by the renderer (#1873). In the fourth, jubarte's renderer draws the clean copy's text `mc:AlternateContent` wrongly, while Accept All, resolved to each Choice, draws what Word does (#1876). `compare` is unchanged.
- **A note's mark keeps its run properties in a redline** (#1871). The comparer leaves a footnote's or endnote's own mark (`w:footnoteRef`, `w:endnoteRef`) out of the diff and puts it back afterwards, as Open-Xml-PowerTools does. It put back a bare run styled `FootnoteReference`, in `compare` (both modes) and in the redline of `add` and `edit`. A mark set in 10pt Arial took the style's size, a mark with no style took one, and the note set differently: page 18 of `25e6d4b508`'s 50 changed after an `add` on its first paragraph. The mark now keeps the run properties of the note it comes from: the revised note's for a note that stays or is inserted, the original's for a deleted one.
- **A table row keeps its exceptions in a redline** (#1869). The comparer rebuilt each row from its `w:trPr` alone, as Open-Xml-PowerTools does, so a row's `w:tblPrEx` (the borders, margins or width one row sets apart from its table) was dropped in `compare` (both modes) and in the redline of `add` and `edit`. After an `add` on `60dfd517a9`'s first paragraph, the calendar's rows lost the borders they set, and both of its pages differed from the clean copy. Rows now keep their exceptions. A paired row whose exceptions changed records the original's in a `w:tblPrExChange`, as Word's compare does, so Reject All gives them back.
- **A VML shape keeps its shapetype in a redline** (#1872). Word writes a shapetype once, beside the first shape that uses it, and later shapes in other pictures name it by id. The redline (`compare` in both modes, and the redline of `add` and `edit`) renumbered every shapetype but rewrote only the `type` of the shape beside it, so later shapes named an id that no longer existed. Word 16 then drew such a shape without its shapetype's path: an oval built from a custom path came out as a plain oval. Pictures were unaffected, since they draw from their image. Every shape now names its shapetype's new id. An edit's redline also leaves a VML or drawing id the document holds once as written, renaming only an id that repeats, so Accept All gives back the clean copy's ids.
- **`convert` draws the branch of `mc:AlternateContent` Word draws** (#1876). Word picks one branch of a wrapper: the first `mc:Choice` whose `Requires` it understands, else the `mc:Fallback`. The renderer picked a branch only for pictures, so text in a wrapper was drawn twice or not at all: a run-level wrapper drew both its Choice and its Fallback, and a paragraph wrapped in a table cell or in the body was not drawn. Each wrapper that holds no drawing now gives way to the branch Word draws, in the body, headers, footers, notes and comments. A picture's wrapper also skips a Choice Word would not take. A Choice counts as understood when each prefix it requires is bound to a namespace Word knows (OOXML's, Office's or VML's). Word 16 passes over a Choice on a namespace the document declares but Word does not know, and `compare` now does the same; before, any declared namespace counted. The renderer also passes over a Choice it cannot draw: Word writes an emoji as a `w16se:symEx` symbol with the same character as text in the Fallback, so the renderer draws the Fallback. That gives `a1cfa10c32`'s ☺ in Apple Color Emoji, as Word draws it, where Arial drew it before.
- **A Word-mode compare leaves inputs without revisions as they are** (#1870). Before it diffs, `compare` in Word mode accepts the revisions its inputs already hold, and that accept ends in the Word-save tidy. The check that decides it matched `<w:ins` as a prefix, so a field's `w:instrText` and a table's `w:insideH` or `w:insideV` counted as revisions. 262 of the 600 pairs of the 0.14.0 redline sample were tidied though neither side held one: adjacent tables were joined (R8), and headers, footers and notes lost their rsids. The redline of `add` and `edit` went the same way. The check now matches whole element names, from the set Accept All resolves, so a table, cell, section or numbering revision counts too; before, those counted only when a field happened to be there. Accept All and Reject All of a redline still join adjacent tables, as Word does. Of those 262 redlines, 251 change; jubarte draws 250 of them as before. Word's PDFs of six score the same, but for `60dfd517a9`'s calendar, whose title table now stays apart from its day table (+2.06).
- **An edit takes a document whose only tracked changes are a style's** (#1885). `edit` and `add` skip a style's recorded formatting (a `w:rPrChange` in `styles.xml`), which does not make the text tracked, but they matched its id against `styles:`, while `changes` lists it as `styles.r0`. So under `existing_revisions: refuse`, the plan default in the API, Python and WASM, such a document was refused with `EXISTING_REVISIONS`, and `--editing-mode` refused it under `keep`, which the CLI's `auto` picks for it ("needs a document without tracked changes"). 14 of the bench's Word-saved documents with comments have this shape. Past the check, the redline went on to accept the source's revisions before the diff, and that accept ends in the Word-save tidy: adjacent tables were joined, and a tracked change inside a comment, which no story lists, was accepted unasked. An edit's redline now leaves the source's revisions as they are, whether or not the plan changed the text. All 14 now edit, with no new `validate` finding; the style's record stays in the clean copy and the redline, and Word 16.115 opens the three redlines probed without a word.
- **A bookmark that starts a paragraph with no text starts there in a redline** (#1880). The comparer drops every bookmark and puts each back by its text offset, and a paragraph with no text (a TOC heading's tab, a paragraph holding only pictures) has none of its own: its bookmark started on the next text instead, in `compare` (both modes; an edit's redline already kept such a paragraph as the source has it, #1872). When that text was inserted, the start sat inside the insertion. Accept All and Reject All then started the bookmark in another paragraph, so a TOC entry or a `PAGEREF` to it pointed at the wrong one. The start now goes back to its own paragraph, counted on its side's view of the redline. On the 0.14.0 redline sample, 29 pairs hold such a bookmark, and 19 of their redlines change. Of the 66 bookmarks in those, Reject All (for the original's) and Accept All (for the revised's) now start every one in the paragraph its source does, where none did before. Some of them now start in an unchanged paragraph and end inside an insertion, which `validate` reports as `BOOKMARK_CROSSES_REVISION` (62 findings on those redlines become 65); Word's own redlines of these pairs cross the same way.
- **A table row keeps Word's save marks in a redline and in Accept All** (#1873). Every accept and reject strips the `w:rsid*` marks across the document, and the comparer accepts both inputs before it diffs, so the rows lost the `w:rsidR` and `w:rsidTr` Word writes on them: in `compare` (both modes), in the redline of `add` and `edit`, and in Accept All and Reject All of any document. The renderer keeps a table's saved grid only when its rows or its own paragraphs carry Word's marks (#1716), so it re-fitted the auto-width tables of a Word-saved document's redline and of its Accept All, though nothing in them changed: Accept All of an `add` on `b6dd0b6ae8`'s first paragraph differed from the clean copy on 3 of its 5 pages. A row now keeps its own rsids, as Word keeps them; its paragraphs and runs still lose theirs. That Accept All now matches the clean copy on every page. Of the 0.14.0 redline sample's source documents with a table Word marked, 152 take an `add`, and its Accept All re-fitted 326 of their tables in 44 documents; now it re-fits none. On the sample itself, 268 of the 600 redlines change; jubarte draws 263 of them as before, and its PDFs of the other five come a little closer to Word's (+0.01 to +0.03).
- **A comment is kept when another one says the same** (#1898). The comparer took two comments with the same author, date and text, one on an empty range and one on text, for copies of one comment, and dropped the empty one: in `compare` (both modes) and in the redline of `add` and `edit`. `1672057675` holds a comment and its reply at the start of a paragraph, and two later comments that say the same over other text; its redline lost the first two with their markers, though the clean copy kept them, so Accept All was missing them. Only a comment of the original that the revision holds on the same range is now taken for a copy; comments that say the same are otherwise kept, each with its own range. Of the 117 bench documents with comments, `1672057675` was the one whose redline lost a comment Word draws, and it now keeps all six; the 600 redlines of the 0.14.0 sample keep the comments they had. Redlines still drop a comment that has no `w:commentReference` anywhere (8 of the 117), which Word does not draw.
- **A comment's reference keeps its place and its formatting in a redline** (#1897). The comparer drops every comment marker and puts each back by its text, and it wrote each `w:commentReference` as a new run with only the `CommentReference` style, right after its range's end: in `compare` (both modes) and in the redline of `add` and `edit`. A reference with its own size (`r5_01`'s is 10.5pt) lost it, and one written apart from its range's end, such as at the start of the next paragraph, moved to that end, so Accept All of an `add` was not its clean copy. A reference now keeps its run's properties (the revised document's for a comment the revision holds, the original's for one it removed) and goes back to its own place when the text between its range's end and it is unchanged. Of the 111 bench documents with comment references, Accept All of `add -p p0` gave a reference other properties than the clean copy in 77 (`auto`) and 89 (`refuse`), and moved one in 3 and 7; it now does so in 2 and moves one in 2, in each mode, and none of those four comes from the reference (pretty-printed source whitespace, a paragraph Accept All loses, a comment over a drawing whose text box text is projected). In the 0.14.0 sample, 79 of the 600 redlines change, only in their reference runs, with as many references and the same `validate` findings as before, and jubarte's PDFs of them do not change; of the 285 references in Word's own redlines of those pairs, 206 now match Word's in place and properties, where none did. Most of the rest differ because Word sizes a reference from the text around it, not from its source run. `compare` in Word mode still gives a reference run that sets no size `w:sz` and `w:szCs` 20. Word opens the seven files probed, six redlines and one Accept All, without a word.
- **A row that takes in the original's extra cell keeps the revision's table width, as in Word's redline** (#1914). When the revision's row holds fewer cells than the original's row it pairs with, Word mode keeps the original's cell and rebuilds the grid for the extra column. That rebuild wrote `tblW 0 auto` and split the page's text width. Word keeps the width the revision's table declares, in twips or as a percentage, and splits it. The original's cell also kept its own width. Word gives it the properties of the revision's cell before it, all but the span, and records the original's in its `tcPrChange`. In one bench pair, a second column joined one-column boxes 9809 twips wide, and the text of their second cells printed past the page edge; it now prints where Word prints it. On the 72 bench pairs whose documents both hold two or more tables, the table width now equals Word's in 294 of 297 live tables (278 before) and the cell widths in 4767 of 5249 cells (4445 before). Word still lays such a grid out again where we keep it (docs/WORD_DIFFERENCES.md).
- **`jubarte validate` no longer reports a text box's fallback copy of a picture as a duplicate drawing** (#1817). Word saves a text box twice, as a DrawingML choice and a VML fallback, and a picture inside it twice under one `wp:docPr` id. `DUPLICATE_DOCPR_ID` counted the fallback copy as a second drawing. Word reads one of the two branches and opens such a file clean (the header of a bench fixture, and Word's own redline of it). A picture outside the fallback that repeats an id is still reported.
- **Word opens redlines whose new words sat right before deleted tables** (#1851). When a revision's words sat in a paragraph whose mark is deleted, right before a whole-deleted table, Word 16.115 crashed opening 6 of the 16 such release-sample redlines. Word's log named its table cache. Compare now moves those inserted runs to the start of the paragraph Accept All joins them to, after the tables. The accepted and rejected documents do not change. On the release sample, the move changes 15 redlines, and all 15 were opened alone in Word 16.115: 13 are CLEAN, including 4 that crashed before, and the other 2 crashed before too and still do. The cost is that the redline shows the new words after the deleted tables, where Word's own redline shows them before: the 8 others scored lose 14 bench points each on average, and the 4 that open now gain 180.5 in all (net +68.5 on 600 pairs). With deleted paragraphs between the words and the table, the words keep Word's place: that shape crashed Word no more often than any redline. A selective `accept` or `reject` (by id, author or kind, or an edit plan's) that leaves the words right before the table moves them too. Accepting every other mark of Word's own redline of one bench pair joined the words' paragraph to the empty one before the table. Word 16.115 crashed opening that result, and it opens now. Recorded in docs/WORD_DIFFERENCES.md (difference 16).
- **Word opens the redlines whose deleted section breaks carried a page-numbered footer** (#1833). Word 16.115 crashed opening 9 of the 23 release-sample redlines (#1814), and Word's own redlines of those pairs too. In each, the revision drops a header or footer that only section breaks the redline deletes reference, and the footer holds a PAGE or NUMPAGES field. Word mode now deletes such a story whole, its closing paragraph mark included, and the paragraphs keep their own properties. A deleted break that references a page-number footer crashed Word whatever its marks, so the footer the final section inherits now moves to the final section, and its header stays on the break. It moves only when the redline deletes a table row (Word's crash is in its table cache) and no earlier section sets that footer. So a rejected section shows no footer, never another section's. Word opens all 9, and all 53 full-corpus redlines with the shape (16 crashed in 0.14.0; the move applies to 18). Accepting shows neither; rejecting gives the original's text, but a moved page-number footer shows on the last section only. A story a live section also references keeps Word's shape (docs/WORD_DIFFERENCES.md).
- **A paragraph between two deleted tables stays between them in Word mode** (#1826). When a revision replaced a body of `table, paragraph, table` with new text, the default mode moved the original's middle paragraph, carrying the new text, ahead of the first table. That left the two deleted tables adjacent, and `jubarte reject` fused them into one table. The paragraph now keeps its place between the tables, as in Word's redline, and rejecting gives the original's two tables back. The new text stays on that paragraph, where Word puts it in a paragraph of its own ahead of the first table (docs/WORD_DIFFERENCES.md).
- **jubarte builds against oxml-opc 0.14.4.** oxml-opc 0.14.4 made `OpcPackage::parts` hold shared bytes (`Arc<Vec<u8>>`), so jubarte 0.13.0 and 0.14.0 fail to compile wherever Cargo picks that version: `cargo install jubarte-redlines` without `--locked`, or a crate that depends on jubarte. jubarte now requires 0.14.4 and stores its parts that way. Its output does not change: the 600 release-sample redlines are byte for byte those of the build on 0.14.2.
- **Review follow-ups to the fixes above.** These are from the bots' reviews of their PRs.
  - A bookmark that starts a paragraph with no text after the document's last text, such as a final picture, now starts there too. It used to start after the last text before it.
  - A paragraph the other side inserted or deleted no longer takes a textless bookmark's start. The count stays on this side's paragraphs, in both modes, so Accept All and Reject All still start it on the picture or the tab.
  - A text box drawn before such a start no longer takes the start into the box (#1880).
  - The footer move leaves a footer part alone when one of its slots must stay: a part shared with a slot an earlier section sets moves whole or not at all.
  - A first-page footer moves only to a final section that shows first pages (`w:titlePg`) (#1833).
  - The footer move and `DELETED_SECTION_PAGE_FIELD` read a field code split across runs (`PA` + `GE`) as one code, so such a PAGE field counts, and a `PAGEREF` split as `PAGE` + `REF` no longer does.
  - `strike` and `restore` refuse a change that names a mark twice (`r12+12`).
  - A replacement half that is already struck still names its other half in its note (#1849).
  - A paired row takes the revision's `w:tblPrEx` with format-change tracking off too (#1869).
  - An endnote's mark is no longer doubled by a footnote-styled one (#1871).
  - A drawing id repeated at `u32::MAX` gets a fresh id instead of `u32::MAX` again (#1872).
  - `debug -c tables` lists tables in a cell's text boxes, and no DrawingML `a:tbl` (#1817).
  - `scrub --rsids` strips a table row's `w:rsidR`/`w:rsidTr` too. Accept and reject still keep them for the table layout.
  - When several tables are replaced together, a repeated row counts once in matching old tables to new ones, so the closest table pairs.
  - The ids of a split table's copied change records wrap past `u32::MAX` to unused ones instead of overflowing.
  - `gridSpan`s whose sum passes `u32::MAX` are trimmed to the grid instead of overflowing the check.

<!-- changelog-skip: #1933 tooling-only: convert_sweep.py's 76 set is the baseline's stems -->

## [0.14.0] - 2026-10-24

> **Summary.** Breaking: the Rust CLI's `ReadArgs::paragraphs` is a `Vec<String>`, one entry per `-p`, and `Command::Changes` gains `author` (#1690). New: `read -p` and `--select` repeat, `read --tables -o FILE` writes the grids, `changes --author NAME` filters, `validate` reports `CELL_FINAL_MARK_REVISION`. Tracked edits keep each author's formatting history apart and refuse a new duplicate revision id before writing; table redlines avoid the cell final-mark revision that hangs Word; `accept` and `reject` resolve rows, cells and section breaks as Word does. No build carries `block-buffer` 0.10.4 (GHSA-qwgh-2vcv-g2f7). On the release's two 600-item samples, scored against Word's own output: redlines mean 71.62 (Docxodus 64.92), counting as 0 the 23 redlines Word 16.115 crashes opening; PDFs mean 79.19 (LibreOffice 54.61) with no failed document. Word's own redline of the pair crashes Word on 20 of those 23 pairs; 3 crash only with jubarte's (#1802), which the owner waived for this release. One pair regresses (26ce1d6300, -15.06, #1759). The full Miri suite did not run for this release, and the speed comparison with 0.13.0 could not resolve a 5% change on the release machine, where one binary varied by up to 9.7% against itself (#1797). 0.13.0 never reached npm; this is the first npm release since 0.12.0.
>
> **Docs.** No README changed. CHANGELOG.md gained the 0.14.0 section; the Rust, Python and JavaScript guides (docs/rust.md, docs/python.md, docs/javascript.md) and docs/locations.md, docs/flags_datetime.md cover the repeatable read -p/--select, read --tables -o and changes --author; docs/SELF_UPDATE.md covers signature verification; docs/VERSIONING.md and docs/WORD_DRIVER_AUDIT.md describe the Ring-3 Word probe through the bench; SECURITY.md and docs/OPENSSF.md the provenance and advisories; examples/cli/help-cli.md and skills/jubarte-documents/SKILL.md were regenerated from the CLI's help; LICENSES.md updated.

### Release

- **`scripts/release.sh` proves what a person used to sign off** (#1742). Step 0 refuses a patch bump that adds a feature (a `### Added` heading) or breaks the public API (`cargo semver-checks` against the previous tag), runs `scripts/test_release_workflow.py`, and checks the `release` environment and, once `keys/` holds a public key, the `ZIPSIGN_PRIVATE_KEY` secret. Step 2 refuses a CHANGELOG section that misses a fix/feat/perf/security pull request merged since the previous release, or that still holds placeholder text. The script tests run on Python 3.12, the crate's keywords are checked against crates.io's limits, queued CI is cancelled before the tag goes up, and step 12 verifies one archive's build provenance. Rustdoc no longer opens in a browser unless `--open-docs` asks. When npm refuses a trusted publish, release.yml's log names the two causes, a trusted publisher that does not allow `npm publish` or one that expired, and the command that fixes it.

- **The 0.13.0 release audit, closed in the script** (#1742, #1744-#1790, #1792, #1793). Step 0 reads GitHub's security findings with an admin's token and stops on a report in triage or an unpatched draft above low, and the patch gate denies every cargo-semver-checks lint, including one the manifest would lower. Step 3 refuses evidence scored on another engine and a measured sentence its results do not hold. Step 6 builds its API snapshot under the mbx build cache again, and names a previous release that left no snapshot. Step 8 checks the tag's shape. Step 12 installs each channel the way a user does: this machine's archive against `SHA256SUMS.txt`, a PyPI wheel, `npx` and `npm i` with `require()`. 0.13.0 never reached npm, and no step said so; 0.14.0 is the first npm release since 0.12.0. Every checklist proof names a tool a repository holds: the parity check (`tools/parity`), the lane-validity check and the two-binary identity run are now committed.

- **Build provenance for every release asset** (#1742). When release.yml's release job publishes the GitHub release, its archives, wheels and sdist carry a SLSA build-provenance attestation, Sigstore-signed by the release workflow: `gh attestation verify jubarte-X.Y.Z-linux-x86_64.tar.gz -R jandira-tech/jubarte-redlines` ties a download to the run that built it. A release made from a run's artifacts instead says in its notes that the release job did not publish it, and how to check whether an attestation covers it (#1749, #1768).

- **The audit's fixes, one by one.** The patch gate denies every lint cargo-semver-checks skips (#1744), and `scripts/semver_lints.py` refuses a missing cargo-semver-checks (#1771). Step 2 reads changelog completeness from PR titles and refuses a failed discovery (#1745). A prerelease tag is no previous release (#1746). Every engine `gh` call names `jandira-tech/jubarte-redlines` (#1747). Each matrix row builds what its label names (#1748), and provenance is bound to the tag and its commit (#1749). Step 0 stops on open security findings (#1750), refuses the canonical checkout and shows HEAD's red CI (#1751). Step 3 proves the evidence's engine (#1752) and holds the measured sentence to the evidence (#1753). Adoption rows and headerless files stop the release (#1754). Every tag step 8 keeps or writes has release.yml's shape (#1755). Step 12 installs each channel the way a user does (#1756). Step 6's API snapshot builds under the mbx build cache (#1769). The Ring-3 Word probe opens Word through the bench's `word_pdf.py` under its watchdog, opens each failure again alone, then opens Word's own redline of that pair (#1794). The WASM parity driver resolves `--module` to an absolute path (#1799).

<!-- changelog-skip: #1804 test-only: CodeQL #54 in jubarte-wasm/cli/test/cli.test.mjs -->
<!-- changelog-skip: #1812 #1811 test-only: npm-smoke.mjs takes the pair ref r1+2 -->

- `scripts/release.sh --checklist` is now the whole release, not only the
  script's own steps. New lines cover what a release owes outside them:
  before — the full Miri suite and the docs/VERSIONING.md quality gate,
  the Ring-3 Word open probe on the release samples, the bump decision,
  reading the engine CHANGELOG section that becomes the release notes,
  drafting the six one-line notes, a release.yml review (wheel and binary
  matrices, SHA-pinned actions, the ZIPSIGN secret vs `keys/`, and the
  `release` environment the OIDC publish jobs run under), and the App
  Store path's credentials and unused version number; during — the
  NUMBERS sentence against the checker's aggregates, the security scan,
  the GitHub release page's assets and notes, the site's own release
  ladder, and reading every jubarte.pro page the deploy rebuilt
  (/download, /benchmark, /demo, home, /pro, /terms); after — merging the
  app's `release/vX.Y.Z` pull request, `jubarte self-update --version`,
  installing each library the way a user does, reading docs.rs and the
  registries' pages, and the App Store item now carries the in-app
  purchase and the metadata checks of `MAC_APP_STORE_RELEASE.md`. The
  dangling empty "step 14" phase is gone, the one proof that carried jq
  pipes (the checklist printer splits on `|`) is written pipe-free, and
  the checklist test derives its phases from the script instead of
  counting to 14.

### Breaking

- **Rust CLI argument types change** (#1690). `jubarte::cli::ReadArgs::paragraphs` is a `Vec<String>`, one entry per `-p`, instead of `Option<String>`. `paragraph_spec()` returns the joined comma list. `Command::Changes` also gains `author: Vec<String>`. The read facade JSON still carries one string or `null`.

### Added

- **`read -p` repeats** (#1690). `read FILE -p p3 -p p8` reads as `-p p3,p8`, in the view, `--json`, `--tables` and `--for-edit`. `--select` repeats the same way.

- **`read --tables -o FILE` writes the printed grids to a file** (#1690). This works in the binary, Python CLI and npm CLI. A `.json` name is refused. An existing file needs `--force`.

- **`changes --author NAME` lists only that author's changes** (#1690). The match is exact and the flag repeats. It filters text and JSON output in the binary, Python CLI and npm CLI. The Python native bindings and WASM `listChanges` / `changesText` also take an optional author list. Revisions inside comment bodies remain outside the listing.

- **`edit --help` shows a complete plan** (#1690). The binary and npm help also show the separate PlanV2 workflow, `edit template` then `edit apply`. Python help leaves out those routes because its CLI cannot run them.

- **`validate` reports `CELL_FINAL_MARK_REVISION`** (#1669). It identifies a cell's final paragraph mark with an insertion or deletion that no matching row or cell revision covers. Word opens that shape, but Accept All or Reject All can hang. The finding blocks an edit; `--repair` does not repair it.

- **Rust can check package admission before writing** (#1665). `jubarte::cli::refused_package(path, bytes)` returns a refusal message when the bytes are not an admitted Word package, or `None` when they are.

- **Rust exposes release-archive signature verification** (#1705). `jubarte::update::verify_signature` takes an archive path and a slice of `ReleaseKey` public keys. It checks zipsign `.tar.gz`, `.tgz` and `.zip` archives.

- **Rust exposes border and cell-margin order tables** (#1712). `jubarte::comparer::order_tables` adds `TCBORDERS_ORDER`, `TBLBORDERS_ORDER`, `PBDR_ORDER` and `CELL_MARGINS_ORDER`.

- **Rust has more cached XML-name helpers** (#1669, #1694, #1712, #1716, #1720). `jubarte::namespaces` adds `W::custom_xml`, `M::r`, `M::r_pr`, `M::o_math`; `W::proof_err`, `W::comment_range_start`, `W::comment_range_end`, `W::comment_reference`, `W::perm_start`, `W::perm_end`; `W::grid_col`, `W::type_`, `W::shd`, `W::fill`; `W::rsid_r`, `W::rsid_tr`, `W14::para_id`; and `W::w`. Each returns a cached `XName`.

### Changed

- **`validate --repair` says what it checked** (#1665). The binary and Python CLI print `structure only`, followed by either `history not checked (--original, --author)` or notice that the history check follows. Add `--original` and `--author` to check history. A history failure still exits 2 after repair.

### Fixed

- **Edits split formatting history without repeating its id** (#1656). Under `existing_revisions: "keep"`, splitting a run gives copied formatting records fresh ids. The original fragment keeps its id, author, date and old properties.

- **Formatting-history checks recognize split records** (#1659). `validate --original ORIGINAL --author NAME` and typed edit plans compare the recorded properties and the text they cover. A legitimate split no longer reports new foreign history or refuses the edit merely because its copies have new ids.

- **New words do not acquire another author's formatting history** (#1658). Inserted and replacement text gets separate runs without the earlier formatting record. This applies to the clean copy and to tracked replay inside the editor's own insertion.

- **A formatting range stops between text elements in one run** (#1658). Formatting that ends at the end of one `w:t` no longer spills into the next `w:t` in that run.

- **Typed replacements keep the words they keep inside existing insertions** (#1658). The clean copy preserves the earlier insertion record on unchanged words. A replacement's comment covers its new edge words in the redline as well as the clean copy.

- **`edit` refuses new duplicate revision ids before writing files** (#1665). A clean copy or redline that adds a same-kind duplicate fails with `CANDIDATE_INVALID`. Existing duplicates pass unless the edit adds another occurrence. The refusal names the output, part, record type and id.

- **`validate` finds XML parts by content type too** (#1665). Word XML stored under a name such as `word/main.dat` is checked. Custom XML data is excluded from revision-id and paragraph-id checks and their repairs.

- **Revision-id checks and repairs handle range markers** (#1665). Duplicate range starts and the range ends that require unique ids are reported. Case variants of `customXmlDelRangeStart`, `customXmlDelRangeEnd` and `moveToRangeStart` ids count as the same id. Repair renumbers a duplicate start with its matching end; it does not rename an end on its own.

- **Strict DOCX translation reads XML parts without an `.xml` name** (#1665). It leaves custom XML data and parts that are not UTF-8 untouched.

- **An invalid `--original` stops repair before it writes** (#1665). In the binary and Python CLI, `validate --repair OUT --original ORIGINAL --author NAME` checks that ORIGINAL can be read as a Word package before repairing the input.

- **One plan can replace a cell's only paragraph** (#1668). `edit` allows paragraph insertion beside a paragraph the plan deletes. Deleting every paragraph of a cell or story leaves an empty final paragraph with its closing mark kept. Both legacy and typed plans use this rule.

- **Tracked paragraph deletion refuses equations and subdocuments it cannot track** (#1668). Under `existing_revisions: "keep"`, deleting a paragraph containing an equation or subdocument is refused. Apply that operation without `keep` to use the comparer path.

- **Tracked final-paragraph edits protect existing records** (#1668, #1669). Under `existing_revisions: "keep"`, a replacement or append is refused if its formatting would undo a retained formatting record. Appending after a final paragraph whose mark is already tracked is also refused.

- **Markdown paragraph replacements stay in their cell or story** (#1668). Replacing a paragraph whole anchors the new paragraph on the one it replaces. A paragraph added after it follows the replacement.

- **Table redlines avoid the final-mark revision that hangs Word** (#1669, #1684). In both comparison modes and tracked edit replay, a cell's final paragraph mark carries no insertion or deletion without a matching row or cell revision. This also covers cells in nested tables, notes, headers, footers and text boxes.

- **Resolving a row or cell also resolves its final paragraph marks** (#1679). Selective `accept` and `reject` take the matching insertion or deletion off those marks, including nested cells covered by the same revision. A nested row revised on its own stays separate.

- **Candidate refusals name the operation that causes them** (#1681). `CANDIDATE_INVALID` messages name the operation and target, with any earlier operations it needs. Typed plans use `/operations/N`. A refusal that also occurs without operations stays at plan level; codes and exit statuses stay the same.

- **`edit --skip-failed` can skip an operation that fails the candidate checks** (#1681). It leaves out the named operation and prepares the remaining plan instead of refusing the whole plan for lack of an operation id.

- **`accept` and `reject` keep block content controls where Word keeps them** (#1694). When paragraph marks resolve, a control stays around the joined paragraph if the other paragraph contributes no surviving content. It can become inline when both sides contribute content. Multi-paragraph boundaries keep their marks where Word does.

- **Comparison removes inherited shading and table schema errors** (#1712). Redlines drop empty shading fills and `gridCol/@w:type`, keeping other shading attributes and column widths. Cell and table-cell margins are ordered `top, left, start, bottom, right, end`. This includes saved property snapshots and tables in comments. Identical-input comparisons also apply these validity repairs after preparation, but preserve valid source markup rather than applying comparison cleanup (#1796).

- **Comparison keeps cell borders in schema order** (#1712). A valid border set containing both `left` and `start` no longer becomes invalid in the redline.

- **Rejecting paragraph formatting keeps the live section break** (#1708). `reject` no longer merges two sections when the section's last paragraph has a formatting revision. Saved paragraph properties cannot add a stale section break or duplicate the live paragraph-mark properties.

- **Rejecting an inserted row removes it even when its properties changed** (#1713). `reject` also removes a wholly inserted table whose rows carry property revisions. Selective rejection by author follows the same rule.

- **PDF and PNG conversion fits more tables without saved Word widths** (#1716). An autofit table with an automatic table width can fit its content when every grid column has a cell without its own width. Missing cell widths, automatic widths and zero dxa widths qualify. The new rule keeps the grid for tables carrying Word save marks and for columns whose cells all specify widths.

- **Fitted table columns account for pictures, nested tables and spans** (#1716). PDF and PNG conversion includes picture widths and nested-table grids, and shares a spanning cell's content across its columns. Overlong words no longer push the fitted table past the available width. A cell without its own width no longer takes the first nested cell's width.

- **A grid column's `w:type` no longer forces content fitting** (#1720). PDF and PNG conversion keeps a numeric grid when the cells specify their widths, with or without that attribute. A missing or nonnumeric grid width triggers content fitting on its own.

### Security

- **New Python CLI outputs are owner-only** (#1806, #1808). `python -m jubarte_redlines read --tables -o FILE` creates a new FILE with mode 0o600, whatever the umask, so other users of a shared machine can neither read nor write it (CodeQL `py/overly-permissive-file`). The native binary still creates 0o666 before the umask. A file it replaces keeps its own mode.

- **No `block-buffer` 0.10.4 in any build** (#1705, GHSA-qwgh-2vcv-g2f7). `jubarte self-update` checks zipsign signatures itself (`src/update/zipsign.rs`, on ed25519-dalek 3) instead of through `self_update`'s `signatures` feature, whose zipsign-api 0.2 pinned ed25519-dalek 2 and with it `block-buffer` 0.10.4. The archive format and the keys are unchanged; no release key is committed yet, so nothing is checked until one is. Cargo.lock still records the unbuilt chain, because `self_update`'s archive features name `zipsign-api?/…` (rust-lang/cargo#12937). Rust API: `jubarte::update::verify_signature` and `ReleaseKey`.

## [0.13.0] - 2026-10-22

> **Summary.** Breaking: replace writes only the words that differ, so kept words keep their formatting; the marks Word writes for one edit are one logical change with a composite id (r7+8+9+10), and accept/reject take it whole; jubarte prints short ids; the timestamp flag is -d, --datetime everywhere (--date is removed); validate and audit are one command. New: edit --skip-failed applies what passes and writes a plan of the failures; visible rejects (--strike, --restore); Word math reads and writes as LaTeX; charts, fields, images and inline HTML round-trip through Markdown; check FILE --expect; typed v2 edit plans. PDFs lay out math fractions, radicals and delimiters, divide columns as Word does and update header PAGE fields; Strict OOXML packages read and compare as themselves. Dual-header and structurally inconsistent ZIP packages are refused (GHSA-m665-v7pm-7grj); rustls 0.23.46 fixes RUSTSEC-2026-0346 and -0347. Registries now publish from CI through OIDC trusted publishing. On two 600-item samples scored against Word's own output: redlines mean 71.63 (Docxodus 64.92), counting as 0 the 23 redlines Word 16.115 crashes opening (KNOWN_ISSUES #8); PDFs mean 79.19 (LibreOffice 54.61), no failed document.
>
> **Docs.** README.md and the library READMEs follow the new flags and short ids; docs/rust.md, docs/python.md and docs/javascript.md carry the regenerated CLI and API references; docs/DIALECT.md, docs/MARKDOWN.md and docs/flags_datetime.md describe the settled Markdown dialect and the --datetime flag; docs/adoption/ covers MCP and the edit plans; docs/WORD_DIFFERENCES.md and docs/WORD_COMPARE_RULES.md record the Word behaviour this release copies; KNOWN_ISSUES.md is updated.

### Release

- **Registries publish from CI with OIDC trusted publishing** (#1642). `release.yml` publishes crates.io (`rust-lang/crates-io-auth-action`, self-hosted runners), PyPI (`uv publish --trusted-publishing always`, the seven CI wheels plus one shared sdist, self-hosted runners) and npm (`jubarte-wasm`, then `jubarte-redlines`, on a GitHub-hosted runner because npm refuses self-hosted ones). No registry token exists. `scripts/release.sh` steps 9–11 wait for those jobs, and `--local-publish` keeps the old token path as the fallback.

- **Eight CI workflow files are five (#1649).** `ci.yml` takes the
  Markdown round-trip job and hands cargo-deny and REUSE to
  `security.yml`, which keeps its nightly schedule; `docs.yml` takes the
  adoption checks; `jules.yml` joins the two Jules reviews; `release.yml`
  is unchanged. A change to `tests/legacy_doc.rs` now triggers the docs
  workflow, and `scripts/test_docs_workflow.py` keeps every test the
  workflow runs inside its path filter.

- **The Jules reviews are back, and quieter.** Restored after their
  removal, they read only the files a pull request changes, so ruff and
  shellcheck no longer report the same old findings on every PR (ruff is
  pinned to 0.17.2), and the report comment is edited in place on each
  push instead of posted again. The el conejo report runs four checks,
  each with its own status — lint, syntax and format, security, workflows
  — beside the LLM review as a fifth row, and reads its rules from
  `.github/jules-review-rules.md`: correctness and Word fidelity,
  security, performance and improvements, best practices and tests,
  syntax and style. Both prompts state Word parity: never refuse an input
  Word opens; ZIP part names are OPC URIs, never file-system paths.

- **Python 3.12 is the floor for the repository's scripts.** CI runs
  the script tests and the release publish step on 3.12, and `ruff.toml`
  targets py312, so ruff reports syntax newer than the floor. The
  `jubarte-redlines` Python package still supports Python 3.10
  (`requires-python >= 3.10`; `jubarte-python/` is checked at py310).

- **No build cache feeds a shipped artifact.** `release.yml` drops
  rust-cache from the binaries job and turns maturin's sccache off, so a
  release builds from a clean state and a cache entry an earlier run
  wrote cannot reach a published binary or wheel (zizmor
  cache-poisoning); `scripts/test_release_workflow.py` refuses a cache
  step in the workflow.

- **The public and private rustdoc builds run with `-D warnings` in
  CI.** They are the two builds `scripts/release.sh` gates a release on
  (the public docs and the private items), now part of the fmt + clippy
  job, so a broken doc link or a module documented twice fails the
  build.

- **Published artifacts carry no builder's paths.** The WASM/npm
  builds remap the builder's home directory and checkout out of the
  binaries, the wheel's remap matches the npm one, and the CI path gate
  reads every user name and the CLI package too, so no machine's paths
  ship inside a release. (#1480, #1481)

- **The npm build's stamp vouches only as the full commit hash**, and
  a release refuses an npm build the tagged engine did not produce
  (#1088); `wasm-cli` refuses a symlinked `--consume` plan on Windows and
  reads files open-then-fd (#1059).

### Breaking

- **`replace` keeps what it keeps** (plan 12, #1262). A `replace` with
  neither `whole` nor `format` writes only the words that differ from
  `replacement`, so the words it keeps keep their formatting (a bold
  defined term is no longer re-typed plain with a `w:rPrChange`) and the
  document's characters: a curly quote or a non-breaking space stays
  where `replacement` types the plain one. Words that all change are one
  change, as Word Compare shows them. To restyle only a quote, give the
  replace `"whole": true`; a replace that changes nothing reports a
  `message`. The range a replace names still conflicts with other
  operations as a whole. `"whole": true` over runs of mixed formatting is
  one deletion and one insertion, with no leftover `w:rPrChange` (#1375).

- **One logical change per edit** (#1261). The marks Word writes for one
  edit are listed as one change: one author's touching insertions (or
  deletions) in a story, whatever their dates, get a composite id
  (`r7+8+9+10`) and a `members` list; a deletion followed by the same
  author's insertion is one `replacement` (`r3+4`, text `old → new`). A
  change takes its own paragraph's mark (`r5+6`) and ends there. `jubarte
  changes` prints `\t(4 elements)` after such a change's text; `read
  --json` is snapshot schema 3 and lists logical changes; Python
  `Change.members` and `ChangeMember`, and WASM `listChanges`, follow.
  `accept`/`reject --id`, `resolve_revisions` and `-p … --accept|--reject`
  take a composite (`r7+8+9+10`, or `r87+r88`) and refuse part of a
  change, naming the whole (`accept`/`reject` exit 2; an edit plan fails
  with `INCOMPLETE_CHANGE`); one whole side of a replacement may still be
  settled alone. The library's
  `accept_elements`/`reject_elements` settle marks one by one. The edit
  report counts logical changes and gains `revisions.replaced`. The
  agent view tags every piece of a change with its joined id
  (`{++bold++}{r7+8 @AC}{++ words++}{r7+8 @AC}`, `¶{r5+6 @AC}` when it
  ends its paragraph), and the writer gives each piece back its own
  mark; a replacement with no text on either side names each side by its
  own marks on its id line (`rev r4 @D, rev-del r3 @D`); the header counts such a change once (`2 marks shown as 1
  change`), and each other story counts its changes as `jubarte
  changes` lists them, with its marks when they differ (`3 in header1
  (7 marks)`).

- **Markdown images stay inside their folder** (#617). `convert`, `compare` and `diff` read an image only when its decoded path resolves, symlinks followed, inside the Markdown file's folder or the `--resource-path` root. An absolute path, a `..` walk, a symlink out or a percent-encoded scheme (`file%3A…`, `DATA%3A…`) now becomes the image's alt text, and each alt-text warning names its reason (`outside the resource root`, `a URL scheme`, `not found`). Pass `--resource-path DIR` to trust another root.

- Refresh saved edit anchors quoting the old in-code revision spelling (`` `w:del{++        ++}{r76 @AS}` ``): marks inside code are now literal, so those anchors can return `ANCHOR_NOT_FOUND`; quote a fresh view with marks outside the code fence.

- `read --json` snapshot schema 2: `summary.tables` and `tables[]` count
  top-level tables, with view-aligned `index`. Cell tables move into
  recursive `nested` arrays with a `parent` cell path and no `index`.
  `read --tables` uses the same top-level numbers and indents nested grids
  under their cell rows. Python `Table` gains `parent` and `nested`; its
  nested indices are `None`. Legacy edit-plan schemas are unchanged.

- **Markdown revision attribution uses the settled brace dialect.** Update saved
  views and consumers using this migration table. Comments keep `{>>…<<}`.

  | Rule | Old | New |
  | --- | --- | --- |
  | Attribution tag | `{++new++}{>>r1 @AL<<}` | `{++new++}{r1 @AL}` |
  | Parent chain | `{>>r3 @BO on r1 @AL<<}` | `{r3 @BO re r1 @AL}` |
  | Merged ids | `{++a b++}{>>r1+2 @AL<<}` | `{++a++}{r1 @AL}{++ b++}{r2 @AL}`; merge only equal ids |
  | Substitution pairing | Adjacent changes could merge regardless of direction | Only deletion then live insertion by the same author in the rendered stream; different inline dates stay separate; old id precedes new id in `{r1+2 @AL}` |
  | Inserted paragraph mark | `<!-- p0 break-ins r7 @AL -->` on an id line | Inline `¶{r7 @AL}` at the paragraph end |
  | Literal escape | Attribution-shaped text after a closer was unprotected | Escape the opening brace, as in `{++new++}\{r1 @AL}` and `¶\{` |

- **`edit::PrepareOptions` has a `syntax` field** (`PlanSyntax`: `V1`,
  `Flags`, `V2`), the request syntax a refusal quotes. A struct literal adds
  `..PrepareOptions::default()`.

- **`--skip-failed` adds library fields (#1253).** `edit::PrepareOptions`
  has `skip_failed`, `edit::Operation` an optional `group`,
  `edit::EditReport` `skipped` and `output_sha256`, `edit::PreparedEdit`
  `skipped`, and `capabilities::Operations` `edit_skip_failed`. A struct
  literal adds the field (`skip_failed: false`, `group: None`) or
  `..Default::default()`.

- **`inspect::Snapshot` has a `track_changes` field (#1255)**
  (`Option<Settle>`), the choice `inspect::settled_snapshot` read the text
  under. A struct literal adds `track_changes: None`.

- **Comment and bookmark ids survive accept and reject (#1264, plan 14).**
  `jubarte accept`/`reject` (all or some), `convert --track-changes`,
  `accept_revisions`/`reject_revisions` in every host, and an edit plan that
  settles revisions keep the ids of the comments and bookmarks that stay:
  a reply to `c5` after a bare `jubarte accept` lands in c5's thread.
  Before, a document left with no revision was renumbered 0, 1, … as Word's
  save does (`docs/WORD_DIFFERENCES.md`, A17). A comment a plan adds takes
  an id above every id its source held, never one the plan's settling
  removed. An edit's redline still numbers its bookmarks as the comparer
  does; its clean copy keeps them.
  `revision_processor::accept_revisions_package_keeping_ids` is gone
  (`accept_revisions_package` now keeps ids), and so is
  `KeptComment.written_id` (a kept thread's id is its source id).

- **`edit_comment` on another author's comment is refused (#1264, plan
  14):** `COMMENT_NOT_YOURS: c0 is by "Bob"; reply instead
  (reply_comment), or pass "rewrite_others_comments": true to change their
  words under their name`. Only a comment that names the plan's author is
  its own: one that names no author is refused too (`c0 names no author;
  …`). A plan field `rewrite_others_comments` (v1, v2 and Python's
  `EditPlan`) and the `edit --rewrite-others-comments` flag allow it; the
  comment keeps its author and the report notes each rewrite (`op-1
  (edit_comment): c0 is by "Bob"; their words were changed under their
  name`). An `EditPlan` struct literal adds `rewrite_others_comments:
  false`; a `PlanV2` one, `rewrite_others_comments: None`. The discovery
  matrix names the condition on `edit_comment`.

- **`convert FILE.docx -t md` writes the agent view**, the
  Markdown `jubarte read` prints (YAML header, id lines, ids on changes and
  comments), which comes back whole through `convert FILE.md`. Pass
  `--commonmark` for the plain Markdown it wrote before.

- **`edit --accept/--reject` prints the resolved changes, not the view.**
  `edit FILE -p r12 --accept` (or `--reject`, `-p` repeats) prints one line
  per resolved change (`accepted r1+2 in p3: "{~~weekly~>monthly~~}" >
  "monthly"`), nothing else; `-p r1` accepts only `r1`, not its `r1+2` pair.
  A successful `edit` or `add` prints `SUCCESS` and a fenced before/after
  diff of the full agent views, replacing the changed-paragraph listing;
  `-q` hides the whole block, and a context line of tildes cannot close
  the fence (#1140).

- **jubarte prints short ids.** The agent view tags revisions
  `r12` (`header1.r1`, `footnotes.r3` in other stories) and comments `c5`
  (a reply names its root, `re c5`); table lines name rows and cells
  `row1` and `row1.cell2`. `changes` and `comments` list the same ids,
  and `validate`'s messages name paragraphs by them (`p108`, `header1.p1`).
  JSON keeps the long `id` and adds `ref`; machine hints and refusals keep
  the long id.

- **One revision count.** `summary.revisions`, the view's
  `revisions:`, the edit report and `changes` count each listed change once
  across the six parts `accept` and `reject` process (body, headers,
  footers, footnotes, endnotes, styles), formatting changes included. Word's
  own pairs count once: a style's `w:pPrChange` and `w:rPrChange` sharing a
  `w:id`, and a text box's `mc:Fallback` copy. Marks in `comments.xml` and
  `numbering.xml` no longer count.

- **Only the settled short ids parse.** `12`, `#12`, `#c27`,
  `#c27.text` and `t0.r1.c2` exit 2 wherever an id is taken (`--id`,
  `edit -p`, `read -p`, plan JSON, Python and npm alike), naming the
  settled spelling the option takes: `--id 12` → `r12`, `read -p 12` →
  `p12`, `-p 6 --resolve` → `c6`, `t0.r1.c2` → `t0.row1.cell2`. A settled
  id `-p` does not edit yet (`s0`, `p3-p7`, `toc0`) exits 2 as well. The
  long `body:rev:12`, `body:p:12` and `header1:p:0` are refused the same
  way, naming `r12`, `p12` and `header1`; nothing prints them any more.

- **Content controls are `ccN` and `pN.ccK`.** `inspect`'s
  controls and a plan's `fill_control` name a block control `cc0` (the
  body's block controls in order) and one inside paragraph `p12` `p12.cc0`
  (that paragraph's controls in order); a cell's first paragraph also
  spells its controls `t0.row1.cell2.cc0`. The old `body:sdt:3` is refused
  with `ANCHOR_NOT_FOUND`, naming the control's id. Under
  `resolve_revisions` or `existing_revisions` a control id names the
  control of the document the plan's author read, as paragraph ids do.
  `capabilities` lists the spellings under `ids.control`.

- **`read --tables` and `validate` print short ids.** A table
  cell that read `body:p:1=Item\tbody:p:2=Qty` now reads
  `p1=Item\tp2=Qty` (a cell's `p3,p4`, via `ids::short_paragraph`, the id
  the agent view prints for the same paragraph); the snapshot's tables keep
  the long ids. A finding that read `word/document.xml#body:p:5` (or
  `body:p:0` for the audit sets) now reads `word/document.xml#p5` (or
  `p0`); JSON keeps the long `location` and adds `ref`.

- **`compare` (and `jubarte OLD NEW`) dates its revisions now.**
  Without `-d/--datetime` the shared parser fills the current UTC time, to the
  second, so the binary, Python and npm hand the same value to the engine;
  `-d 1970-01-01T00:00:00Z` pins it for reproducible output (the old
  default). `diff`'s patch header already said now; Python and npm no longer
  fill it themselves. MCP `docx_compare` without `date` follows the CLI. The
  library default (`document_comparer::DEFAULT_DATE`, Python
  `Document.compare`, WASM `redlineDocuments`) stays the epoch.
  `cli::CompareArgs::datetime` is `Option<String>`.

- **`convert X.md -o X.docx` signs and dates by the document's
  own metadata.** A change or comment that names no author or date takes,
  in order: its own tag or the `authors:` table; `-a/--author` and
  `-d/--datetime`; the front matter's `author:` and `last_modified:` (else
  `modified:`, `date:`, `created:`; a bare day is its midnight); the
  `--reference-doc`'s `dc:creator` and `dcterms:created`; else `Jubarte`
  and now. Pin `-d 1970-01-01T00:00:00Z` for byte-identical output (the
  old default). `compare`, `edit`, `add` and `diff` change a file, so they
  keep `--author` (default `Jubarte`) and `-d/--datetime` (default now)
  and never read the files' metadata. The library (`DocxOptions::default()`, Python
  `markdown_to_docx`/`from_markdown`, WASM `markdownToDocx`) keeps
  `Jubarte` and the epoch, so library calls stay deterministic, and reads
  no reference document's metadata; a given `author` or `date` beats the
  front matter (Python's `author` default is now `None`). No file owner or
  modification time is read. `cli::MarkdownArgs::author` and `datetime`
  are `Option<String>`.

- **The timestamp flag is `-d, --datetime` everywhere** (`edit`,
  `add`, `compare`, `diff`, `convert`'s Markdown-to-Word date); `--date`
  is gone, and passing it exits 2 naming `--datetime`. `edit` and `add`
  gain the `-d` short flag; `compare`, `diff` and `convert` move it from
  `--date`. The clap
  field is `datetime` on `cli::EditOptions`, `cli::CompareArgs`,
  `cli::Command::Diff` and `cli::MarkdownArgs`, and the shared parser's
  JSON carries `datetime`, so the Python CLI reads `args.datetime` and
  the npm CLI `o.datetime`.

- **The neutral default author is `Jubarte`** (#916), at every site
  that owns one: `edit`, `add`, `compare`, the paragraph patch, the plan
  recorder, the Markdown stamp fallback and `DocxOptions::default` (was
  `Modified User` in 0.12, `Redline` in between), so the agent view's
  handle is `@J`. A plan recorded from operation flags (`--plan X -p …`),
  WASM `flagPlan` and the hosts' record options default to it too, and
  Python's `jubarte.diff()` no longer reads `git config user.name`: its
  owner is `Jubarte` unless `author=` says otherwise. Explicit `--author`
  values and the Python and WASM API defaults are unchanged.

- **`validate` and `audit` are one command.** `validate --rules`
  takes rule sets (`word`, the default; `edits`, which needs `--original`
  and `--author` and is added by them; `a11y`, `style`, `structure`) or
  audit codes, comma-separated; `--strict` fails on warnings too. A word
  or edits finding fails the run (exit 2), and so does any error; an
  a11y/style/structure warning fails only under `--strict`, as `audit` did.
  An unknown rule, or `edits` without `--original`, exits 2 (`audit` exited
  1 on an unknown rule). Every finding has one shape, in the library
  (`validate::Finding`), JSON Lines, Python and WASM: `code`, `set`,
  `severity`, `part`, `location` (was `path`), `message`, `word_fatal`,
  `repairable`; and one text line, `* CODE\tSEVERITY\tPART#LOCATION\tMESSAGE`
  (`  CODE\tSEVERITY\tLOCATION\tMESSAGE` for the audit sets), closed by
  `N finding(s): E error, W warning, I info`. `validate --repair` prints
  `repaired N finding(s)` and then the repaired package's findings; its
  `wrote` line is on stderr. `audit FILE` is now
  `validate FILE --rules a11y,style,structure`. Python's CLI and MCP `docx_validate` take `rules` too, and
  `capabilities` lists `validate_rules`. Library: `validate_with`,
  `findings_text`, `fails`, `RULE_SETS`, `audit::word_layout`,
  `audit::is_rule`, `From<AuditFinding> for Finding`; `ValidateError`
  gains `Usage` and `Layout`.

- **`comparer::finalize` and `comparer::tables` are hidden and outside the
  version promise** (`docs/VERSIONING.md`). They stay public for the tests
  that call single passes, but any release may change them.

- **`PdfOptions` gains `comment_notes`.** A `PdfOptions { … }` literal that
  names every field needs it (or `..PdfOptions::default()`).

- `RenderRequest::pages` counts document pages and narrows the PDF and the
  report as well as the PNGs (an empty list writes no PDF).
  `ConvertError::PageNotKept` names a page `changed_only` dropped;
  `PageOutOfRange` reads "the document has N pages". `PageText` gains
  `number`, and the report JSON's pages carry it. `RenderReport` gains
  `document_pages` (also in the report JSON). `cli::PageOptions` is
  `cli::RenderArgs`, which `convert` and `diff` flatten.

- `cli::Cli` loses `read`: the view's options are `Cli::compare.read`.
  `CompareArgs` gains `reference_doc`, `critic`, `resource_path`,
  `render` and `read`. `cli::top_level_command()` is the model whose
  `--help` lists the tasks only.

- `cli::Command::Edit` and `Add` gain `plan_default`; `Recorded::summary`
  takes `default_plan`.

- `cli::EditOptions` gains `output`; `out_dir` is
  `Option<Option<PathBuf>>` (`--out-dir` with or without DIR), and
  `EditOptions::target(file)` gives the `EditTarget` (`Single` or
  `Bundle`) a host writes. `cli::EditOptions::datetime` gains the `-d`
  short flag; `cli::CompareArgs::date`, `cli::MarkdownArgs::date` and
  `cli::Command::Diff`'s `date` are renamed to `datetime` (`--date` is
  gone).

- `markdown::DocxOptions` gains `author_fixed` and `date_fixed`: `true`
  makes `author` and `date` win over the front matter's; a struct literal
  needs them or `..DocxOptions::default()`. New
  `markdown::docx_stamp(docx)` returns a `.docx`'s first non-empty
  `dc:creator` and its `dcterms:created`.

- `validate::Finding` is `{code, set, severity, part, location, message,
  word_fatal, repairable}` (`path` is `location`); `ValidateError` gains
  `Usage` and `Layout`. `cli::Command::Audit` and `Command::Inspect` are
  gone (`Validate` gains `rules` and `strict`; `audit` and `inspect` parse
  as their aliases).

- `capabilities::Capabilities` gains `validate_rules`, `formats` and `ids`
  (`capabilities::Ids`), so a struct literal needs them; deserializing an
  older manifest still works (`serde(default)`).

### Added

- **Keep nested comment threads during settlement** (#1279). Native
  `accept`, `reject` and `convert --track-changes accept|reject` offer
  `--keep-nested-comment`, off by default. A wholly removed anchor becomes
  an empty untracked anchor, keeping the comment and its replies, with the
  same kept receipt as an edit plan's `"removed_anchor_comments": "keep"`.
  Partial overlaps and flag-off output retain their existing behavior.

- **A plan error teaches (#1258).** An `INVALID_PLAN` for an operation names it
  (its `id`, else `op-N`), its kind, the field, the nearest field the kind takes
  and the list of its fields; an unknown kind lists the kinds; a missing
  `schema_version`, characters after the plan, a bare number for `paragraph` and
  a string for `resolve_revisions.*.ids` each say what to write, as do an `id`
  that is no string and an alias beside the field it is read as. A byte order
  mark before the plan is ignored. Two slips apply as meant, each with a
  `note:` line on stderr and in the `summary` event's new `notes`, present
  when one applies (`EditReport.notes`, `EditPlan::from_json_with_notes`):
  `anchor` for `find` on `replace`, `delete` and `comment`, and `comment` for
  `text` on `comment` and `reply_comment`. `delete` and `rewrite` take a
  `comment`: a deletion's sits on the struck words in the redline only, and a
  rewrite's covers the paragraph. The version is read first: a plan whose
  `schema_version` this build does not read is `UNSUPPORTED_SCHEMA` before
  its fields are checked (a v2 plan given to `EditPlan::from_json` was
  `INVALID_PLAN` for its v2 fields). `edit apply` says a plan that is an
  array, a string or a number is one JSON object, instead of asking for its
  `schema_version`. Under a plan that settles revisions, a deletion's
  comment lands on the paragraph the deletion names as read. When the
  comparer deletes the same words elsewhere, the refusal says so and gives
  `"existing_revisions": "keep"` (`--existing-revisions keep`), which writes
  the deletion as given.

- **`validate OUT --baseline IN` names only the edit's findings (#1256).**
  The findings IN already has print as `already in the input: N (not
  yours)` after the new ones (`new: none`, or `new: N` and their lines),
  then, when the edits fixed some, `resolved: N (in the input, not in this
  file)` and those; only new findings fail the run, `--strict` included. A
  finding is new by identity, never by count: its code, set and part, its
  element chain without indices (`p#` for a paragraph id, in any story),
  its message with element indices and paragraph ids normalised, and a
  place it shares with the input's finding, tried strongest first:
  nothing around it changed; the tracked change it sits in (`jubarte
  edit` keeps the input's `w:id` values); its paragraph's `w14:paraId`;
  its paragraph's text between the same neighbours; that text alone; and
  last the neighbours alone, its paragraph rewritten. Outside a
  paragraph the text is its table's, or its element's with the
  attributes of what holds it (a style's id). So a paragraph added above
  an inherited finding, even one of the same text, leaves it the
  input's; the copy an edit adds beside an inherited finding is the one
  printed as new; and a finding that replaced another, elsewhere, is new
  at the same total. The
  field, table and nesting checks now report the element chain they found
  (`word/document.xml#w:body[0]/w:p[1]/w:r[1]/w:fldChar[0]`) instead of
  an empty place. `--json` prints each finding with `"status": "new"` or
  `"baseline"`, new ones first, then one `{"resolved":[…]}` line. With
  `--author`, IN goes through the same rules, the edits rule included,
  against the same original (IN, unless `--original` names another), so
  the other party's changes IN already had are `already in the input`,
  matched by their id, then by kind, author and text.
  `--baseline` conflicts with `--repair`. Library:
  `validate::{baseline, baseline_keyed, identities, validate_baselined,
  baselined_text, baselined_json, input_notice}`, `Baselined`,
  `FindingKey`, `Place` and `Finding::identity`; Python `validate_run(…,
  baseline=)` and `input_notice`, WASM `inputNotice`.

- **With `--json`, an error is a JSON object on stdout too (#1255).** A
  usage error prints `{"error":{"code":"USAGE","message":"…","usage":"jubarte
  read [OPTIONS] <FILE>"}}`, a runtime error `{"error":{"code":…,"message":…}}`
  (its code the message's leading `CODE:`, else `ERROR`), on one line, beside
  the same text on stderr and the same exit code; `jubarte read x.docx --json
  --bogus | jq .error.code` prints `"USAGE"`. A message clap spreads over
  lines is joined on one, and `usage` keeps each form clap prints, a line
  per form; a removed spelling's line (`--date is removed; use
  --datetime`) carries the usage too. The npm and Python CLIs print the
  same: WASM `wantsJson` and `runtimeJsonError`, Python `wants_json` and
  `runtime_json_error` (a `--json` that is an option's value, as in `edit
  replace … --text --json`, asks for none), and `parseCli` results carry a
  usage error's line as `json`. `--track-changes` also takes `accepted` and
  `rejected`. WASM `tablesText` and `readJson`, and Python `tables_text`
  and `read_json`, take the `trackChanges`/`track_changes` the CLI does;
  WASM `tablesView` and Python `tables_view` print `read --tables` whole,
  `-p` and the view's options included, as `{markdown, warnings}` and a
  `(markdown, warnings)` pair; a pick that reaches no table raises
  `USAGE: …`.

- **`edit --skip-failed` applies what passes (#1253).** Every operation
  that passes is applied and the failed ones are left out: the output is
  the bytes the plan without them writes, exit 0. stderr says `skipped 2 of
  14 operations; the fixes are in PLAN.failed.json`, a plan of the failed
  operations to fix and run on the output (its `source_sha256`, the run's
  `date`, a `why` on each operation that the parser ignores). A
  positional paragraph (`p3`, `{"index": 3}`, `t0.row1.cell2`), or a
  control named through one (`p3.cc0`), that the applied operations moved
  is renamed to its id in the output, and `why` says `p3 is p4 in the
  output` in the plan's own spelling. One with no one-to-one counterpart in
  the output (deleted, merged, split) becomes `source:p3`, which no
  document resolves, so it runs only once given again; a skipped operation
  `existing_revisions` keep refuses
  (`insert_footnote`, `insert_image`, `page_setup`, `fill_control`) says,
  on the skip line and in `why`, to fix it in the original plan. Operations
  sharing a `"group"` stand or fall together (`GROUP_FAILED`). The report's
  `summary` says `"status":"partial"` with `ops.skipped`. `STALE_SOURCE`, a
  failing `resolve_revisions`, a failing strike or restore that
  `--reject --visible` added, and a plan whose every operation fails are
  never skipped; those strikes and restores keep the names the whole plan
  gives them. `--consume` keeps a plan that did not land whole, and says
  so. The manifest's `operations.edit_skip_failed` says whether a runtime
  has it (the CLI and Rust; not yet the Python and WASM bindings), whichever
  runtime asks.

- **Every edit write prints `sha256: <hex>`** of the primary output, also
  the `summary` event's `output_sha256`, and a `--dry-run` that passes
  whole ends with `next:`, the command that writes it: the same arguments
  without the `--dry-run` flag (a FILE named `--dry-run` after `--` stays),
  and `--force` before any `--` when the output, or an `--out-dir` empty
  or not, is already there (#1253). `cli::next_command` builds it.

- **Visible rejects: `strike` and `restore` (#1252).** `edit FILE -p r12
  --strike` deletes another author's insertion with your tracked deletion
  inside it; `-p r12 --restore` leaves their deletion and inserts a copy of
  its words, formatting kept, right after it. Rejecting either way shows
  the decision, where `--reject` leaves no trace. `--content` comments on
  the struck or restored words, and `--reject --visible`
  (`resolve_revisions.reject.visible` in a plan) picks strike or restore by
  each revision's kind. Plans take `{"kind":"strike"|"restore","revision":…,
  "comment":…}` under `existing_revisions: keep`; discovery offers both
  only there. Own revisions, empty ones, paragraph marks, formatting
  changes, a move's source to restore or destination to strike, and a
  split or non-plain deletion to restore are refused with the route that
  takes them.

- **`debug --full-lines`** prints the listing checks' lines (`text`, `xml`,
  `runs`, `changes`, `styledefs`, `numbering`, `render`) whole instead of
  clipping them at 200 characters; `debug::Options` gains `full_lines`. The
  Markdown round-trip gate reads the render check this way: a clipped table
  line kept a window that still held the cell shading the gate drops, so
  `ids-fonts.md` reported a table difference the two packages do not have.

- **`convert --commonmark`** writes Word as plain CommonMark with
  CriticMarkup, for readers outside jubarte's dialect: what `convert … -t md`
  wrote before.

- **A change's note names its number in plain Markdown:**
  `{++new++}{>>r5 Ana Lima (2026-09-29T14:05:00Z)<<}`. The writer reads a
  numbered note as the change's attribution, so an undated author
  (`{>>r75 Di Eng<<}`) and an empty one (`{>>r5 (2026-…)<<}`) come back as
  they were instead of turning into a comment or `Jubarte`. A note with no
  number keeps its old meaning.

- **Plain Markdown keeps Word's cross-reference targets:** every `_Ref…`
  bookmark prints its `<a id>`, since the REF fields that name it print as
  their text.

- **`PdfOptions::comment_notes`** (hidden CLI flag `--comment-notes`)
  attaches each comment as a PDF note at its anchor, for the exceptional
  case that wants one.

- **`convert --scholar`** sets Markdown as LaTeX's article class sets a
  manuscript (Latin Modern Roman 10pt, justified, bold headings, unstretched
  line-break lines) and numbers the pages from page 2, filling in only what
  the front matter leaves out. Python `from_markdown(scholar=…)`, wasm
  `markdownToDocx` `scholar`, and the npm CLI take it too.

- **The Python CLI prints what the binary prints.** `changes`, `comments`,
  `read --tables` and a selected `read --json` come from the engine
  (`_native.changes_text`, `comments_text`, `tables_text`, `read_json`), and
  every `wrote` line from `wrote_line`/`wrote_png_line`, on stderr. Edit's
  report lines are compact with sorted keys, so a bundle is byte-identical
  to the binary's. Python's `Change` gains `ref` and `paragraph`, `Comment`
  gains `ref`, and `Snapshot` gains `changes` and `comments`.

- **The npm CLI prints what the binary prints**, for the commands it has.
  `changes`, `read --tables`, a selected `read --json` and every `wrote`
  line come from the engine (jubarte-wasm adds `changesText`,
  `commentsText`, `tablesText`, `readJson`, `wroteLine` and
  `changeFilter`); the wrote lines move to stderr. A bad change id exits 2
  before anything is read, a failed resolution reads `accept failed: …`,
  I/O errors read as Rust words them, and edit's report lines are compact
  with sorted keys. `applyEditPlan` takes the file's name for its patch,
  which under `existing_revisions: keep` shows only the plan's changes, so
  an edit bundle is byte-identical to the binary's. npm still has no
  `comments`, `validate`, `append`, `scrub`, `visual-diff`, PNG output or
  `--pages`.

- **MCP `docx_text` returns `read`'s agent view** (the header, `<!-- pN -->`
  id lines, revisions and comments with their ids) instead of the body with
  `[body:p:N]` ids; plans take `p12` as written.

- **One id grammar** (`jubarte::ids`), the same in every place an id goes:
  revisions `r12`, `r1+2` (both marks of one change), `header1.r1`,
  `footnotes.r3`, and the long `body:rev:12` the JSON prints; comments
  `c27` (a plan's `comment_id` also takes the number `27`); paragraphs
  `p12`, `header1.p2`, `footnotes.p3`, `t0.row1.cell2` (`.p1` for a cell's
  second paragraph), and the long `body:p:12`. `accept`/`reject --id`,
  `edit`/`add -p`, a plan's `paragraph`, `resolve_revisions` ids and
  `comment_id` take them all; bare numbers and 0.12's `#12`, `#1+2`,
  `#c27` and `t0.r1.c2` exit 2 (see Changed). Every printed id is safe
  unquoted in bash, zsh and csh, and each number has one spelling (`p012`
  is refused).

- `edit FILE -p r12 --accept` (or `--reject`) resolves one tracked change,
  before the command's other operations; `-p` repeats. `--reply TEXT` on
  `edit` and `add` with `-p c5` answers a comment.

- `changes --json` and `comments --json` lines, and the snapshot's
  `changes` and `comments`, carry `ref`, the short id (`r12`, `c5`), beside
  the long `id`; a change also names its `paragraph`, and the `changes`
  text listing gains a paragraph column. `validate --json` lines carry
  `ref`, the short id (`p5`) beside the long `location` (`body:p:5`);
  Python's `Finding` gains `ref`, and MCP `docx_validate` returns it
  (as `docx_changes`/`docx_comments` already return `Change`/`Comment`
  `ref`).

- `c5.text` names the text comment c5 covers, for `edit -p` and a plan's
  `paragraph`. `edit X -p c5.text --content "new"` replaces it as one
  tracked substitution inside the comment (a range over two paragraphs
  that follow each other also deletes the mark between them);
  `--anchor` looks only inside it (`… is not in the text c5 covers`,
  exit 3); `--delete` deletes it. `-p c5` still edits the comment's own
  text. `capabilities` lists the spelling under `ids.comment_text`.

- `read` prints the paragraph id of each footnote and endnote on the
  line before its definition (`<!-- footnotes.p0 -->`,
  `<!-- endnotes.p0-p2 -->` for a note of three paragraphs): the id
  `edit -p` takes to change the note.

- `read --json` prints the snapshot (`inspect --json`'s, which now also
  carries `changes` and `comments` as `changes --json` and
  `comments --json` list them). `-p`, `--head`, `--tail`, `--changed` and
  `--by` keep the paragraphs they select and those paragraphs' changes and
  comments; the view-only flags exit 2 with it, and so does `--json` with
  two documents. `read --tables` prints the table grids. The Python and
  npm CLIs do the same. The library adds `inspect::snapshot`,
  `Snapshot::retain`, `inspect::tables_text` and
  `markdown::read_snapshot`; `ReadView` and `ReadDocx` gain `selected`.

- One writer for `-o`. `accept`, `reject`, `append` and `scrub` write
  what the output's extension names: `.docx` (and any other extension),
  `.md` (the agent view of the result), `.pdf`, `.png` (one file per page)
  or `.json` (the snapshot), and take `--revisions`, `--revision-palette`,
  `--dpi`, `--move-comments`, `--changed-pages` and `--pages` for a PDF or
  PNG. `read` (and `jubarte FILE`) takes `-o FILE.md` for the view or
  `-o FILE.json` for the snapshot, and `--force`. `convert` writes `.json`.
  The Python CLI writes them too (it has no `append` or `scrub`); the npm
  CLI writes all but `.png` (it has no PNG pages, `--pages`, `append` or
  `scrub`).

- `compare` takes the view's options (`--track-changes`, `--comments`,
  `--dates`, `--no-page-markers`, `-p`, `--head`, `--tail`, `--changed`,
  `--by`), `--reference-doc`, `--critic` and `--resource-path` for
  Markdown inputs, and the Rendering group. `-o` writes `.docx` (and any
  other extension), `.md` (the redline's agent view; CriticMarkup when
  both documents are Markdown), `.pdf`, `.png` or `.json` (the snapshot).
  The view's options go with the printed view or a `.md` view, the
  Rendering group with a `.pdf` or `.png`, the Markdown-input options
  with a Markdown document; anything else exits 2. Whether a `.md` is
  CriticMarkup is read from the documents' bytes when their names do not
  say, in the binary and in both hosts.

- **`xmllinq::Dom::attributes_iter`** walks an element's attributes as
  borrowed `(&XName, &str)` pairs, in declaration order, without the `Vec`
  of cloned pairs `Dom::attributes` builds. The crate's own read-only
  callers use it, and the emptiness checks use `Dom::has_attributes`.

- **Word math reads and writes as LaTeX.** `$…$` and `$$…$$` in a
  Markdown file write Word equations, and Office Math in a document reads
  back as the LaTeX that writes it, so an equation survives `convert -t
  md` and back. The symbol table grows 267 more LaTeX names and 10 more
  n-ary operators, primes, `\not`, sized brackets, `array`, `\bmod` and
  `\pmod` read and write, an equation in a table cell keeps its
  backslashes, currency beside math stays prose, and math nesting past
  the limit reads as text instead of overflowing the stack. (#466, #510,
  #513, #537)

- **PDF math: fractions, radicals and delimiters are laid out.**
  `convert` paints bar fractions, radicals and stretchy delimiters using
  the font's OpenType MATH constants and glyph variants (STIX Two Math is
  vendored for the tests), so a fraction no longer collapses into its
  characters on a line, a justified line keeps its box, and math letters
  no font covers stay searchable. (#498)

- **Charts print as Mermaid and write back.** A chart part prints as a
  Mermaid code block in the view, and converting the view back writes the
  chart again, so a diagram survives the round trip as text a reviewer
  can read. (#535)

- **Images and drawings print as readable forms.** Raster images and
  svgBlip drawings print with their payloads riding as reference
  definitions, a text box's words print as paragraphs with its payload as
  a frame, and a drawn text box's content prints as comments in the agent
  view. A Google Docs export's image syntax round-trips exactly, and a
  Markdown-to-Markdown `diff` lists a document's image destinations.
  (#535, #557, #579)

- **Fields print as `{CODE}result{/}` and write back.** A body field
  prints its code and cached result between braces — `{PAGE}1{/}` — and a
  field in the Markdown writes the field, result included; a header or
  footer field keeps its code and its result, and a bare one writes
  without a result. (#466, #502)

- **Code and monospace faces round-trip.** A run in a monospace face
  reads as code (faces matched by whole words, so text merely containing
  "mono" does not), and code spans and fences read back as code, so
  `Courier New` text stays a code span through the round trip. (#466)

- **Inline HTML reads as formatting.** `<b>`, `<i>`, `<s>`, `<code>`
  and `<mark>` read as bold, italic, strike, code and highlight; any
  other inline tag warns and stays text. An HTML block keeps its text,
  `<a href>` reads as a link, and link titles become tooltips. (#466)

- **Links to headings and bookmarks are Markdown links.** A link to a
  heading or a bookmark prints as a Markdown link whose target is the
  heading's `<a id>` anchor, a link to a heading that is not there warns,
  a link's own underline no longer prints as `<u>`, and code spans and
  link destinations open no highlight, field or change. (#466)

- **`jubarte check FILE --expect`** checks a document against a
  source-bound expectations file: typed paragraph-text predicates (exact
  equality, containment, absence) evaluated on the file's own final
  projection, `--json` for one result envelope. The expectations'
  `source.sha256` must name FILE's own bytes — an expectation file bound
  to the pre-edit source refuses as stale on the edited output — and the
  output-bound copy an `edit apply --expect` receipt carries re-checks a
  published output. (#845)

- **`edit template` drafts a guarded plan.** `jubarte edit template
  FILE --select p12,p20-p25 -o edits.json` writes an edit template for
  the selected paragraphs; applying it publishes through the
  commit-boundary recheck, so a template whose source changed refuses
  instead of editing the wrong text. (#843)

- **Typed edit actions and v2 plans.** `jubarte edit
  replace|rewrite|insert|delete|apply FILE` take typed actions over an
  immutable PlanV2 apply — mixed transactions, overlapping literal
  anchors counted, comment anchors and threads composed, actionable
  anchor refusals with bounded context, one `PreparedEdit` path for dry
  run and apply. Valid legacy calls keep v1 semantics; `./replace` names
  a legacy input whose name equals an action. Publication is atomic, an
  output aliasing the source is refused, and the v2 routes report through
  one machine result envelope with exit codes by meaning. (#821, #823,
  #825, #829, #831, #833, #835, #837)

- **`read --for-edit` prints the view an edit needs** — exact source
  hashes and each paragraph's operation readiness, as JSON — and the
  Python and npm CLIs take it too (WASM `forEdit`). (#839)

- **Focused runtime discovery: `capabilityQuery`.** WASM and npm gain
  `capabilityQuery`/`capabilityQueryText`, which answer what one runtime
  can do, and v1 capability profiles publish the dispatcher's bounds and
  closed schemas. (#847)

- **A paragraph's own spacing and size, and a table's geometry, ride
  on the view's lines.** A paragraph's own spacing (Word's `auto` and
  values past 1584pt included, the clause said last winning) and its text
  size (shown runs only, one size) print on its id line, and a table's
  geometry prints on its line, with a cell past the grid keeping its even
  width and geometry the reader refuses left unprinted; the styles legend
  keeps a style's right indent and the default style's omitted indent
  reads as zero. (#1472, #1473, #1474, #1489)

- **Markdown sets Word's Track Changes switch.** A `track_changes:`
  key in the front matter turns Track Changes on in the written document,
  and `convert --tracking on|off` overrides it per run (the Python and
  npm CLIs follow). (#466)

- **The view counts pages by layout when Word cached no breaks.** A
  document whose cached page breaks are missing or stale gets its
  `pages:` count from jubarte's own layout, so the header no longer
  undercounts. (#466)

- **Look-alike quotes read as the straight one.** A curled or
  look-alike quotation mark in `--anchor`, `find` or `--content` reads as
  the straight quote it resembles, so an anchor copied from text with
  `″` or `＂` still finds its words.

- **An edit plan can keep the comments an empty anchor would drop.**
  `removed_anchor_comments: "keep"` preserves a wholly removed anchor as
  an empty untracked anchor, and the comment-settlement receipts report
  every thread a plan removed, with the reason and the kept copy.

### Changed

- **What a settle removes is said before and after (#1159, plan 10).**
  `changes` ends a change's line with a tab and `holds comment c4's mark`
  (`holds comment marks c4, c5`) when it holds a comment's reference mark,
  a field after any `inside r1`, and
  `--json` lists them in `holds_comments` (Python `Change.holds_comments`):
  accepting that deletion, or rejecting that insertion, removes the
  threads. The agent view says it too: a note printed outside the change
  that holds its mark names it last, `{>>c4 @AC in r13: …<<}`, and
  converting the view back puts the mark inside r13 again (before, it came
  back outside, so accepting r13 no longer removed the thread). Every
  `removed_comments` record says `why`, `anchor removed by accepting r13`,
  naming the change the pass settled (the side of a move the plan named,
  the innermost of nested changes), and the receipt line and `jubarte
  accept`/`reject`'s warnings say the same (`ResolvedChanges::removed_why`).
  A reply to a comment whose paragraph the same plan
  deletes is no longer refused as `OVERLAPPING_EDITS`: it goes on the
  deleted text beside its parent, where Word shows the thread; a v2 plan
  with `"track": "off"` removes the anchor and is refused with
  `COMMENT_ANCHOR_REMOVED`, as any untracked deletion of an anchor is, and
  `--editing-mode` refuses the reply (exit 2): its clean copy keeps no
  thread for the reply to join.

- **`read`, `changes` and `comments` name the input's validity findings
  once (#1256).** An input `validate` would flag earns one stderr line,
  `input has 2 validity warnings; jubarte validate FILE lists them` (`1
  validity error`, or `1 validity error and 2 warnings`; `info` findings do
  not count), so an agent does not take them for its own; `-q` drops it
  and skips the check. A JSON read prints no such line (#1309): `read
  --json` carries the counts in its snapshot's `summary.input_validity`,
  `{"errors":0,"warnings":2,"next":"jubarte validate FILE"}` (a clean input
  `{"errors":0,"warnings":0}`), and `changes --json` and `comments --json`
  end with one `{"input_validity":{…}}` line, always, `-q` or not, so
  `--json 2>&1 | jq` stays JSON. `read --for-edit` prints JSON too, so its
  result carries the same `input_validity` (#1340). `read -o FILE.json`
  writes what `read --json` prints and keeps the stderr line, and `read
  --for-edit -o FILE.json` likewise. Library:
  `validate::{InputValidity, input_validity, input_validity_of,
  input_validity_line, input_notice_of}`, `Summary::input_validity`;
  Python `read_json(validity_file=)` and `input_validity_line`, WASM
  `readJson`'s `validityFile` and `inputValidityLine`. The npm CLI's `read`
  and `changes` and the Python CLI print the same. `--rules edits` without
  `--author` now says `--rules edits needs --author, and --original or
  --baseline`.

- **Edits inside another author's insertion land (#1136).** Under
  `existing_revisions: keep`, a replace, deletion or insertion in text
  another author inserted nests in their insertion: Accept All keeps the
  edit, Reject All removes their insertion with it. An edit inside your own
  insertion rewrites it in place. `delete_paragraph` of a paragraph holding
  revisions nests the deletion in other authors' insertions, leaves
  deletions as they are and removes your own pending insertion outright; a
  paragraph holding a move is still refused. The report's `summary` names
  the pieces of each split insertion under its source id
  (`split_revisions`, `EditReport::split_revisions`). An `edit apply` plan
  or a typed `edit delete --whole-block` of a paragraph holding another
  author's deletion no longer fails `REVISION_PRESERVATION_FAILED`: the
  clean copy drops the paragraph with its revisions, the redline keeps
  them.

- **A refusal names the revision in the way and every route around it
  (#1108).** Text another author deleted or moved, and a paragraph or run
  whose mark or formatting change is tracked, are refused with the
  revision's id, author and text, then accept, reject and a comment, each
  with what it costs, quoted in the syntax the caller wrote (operation
  flags, including flags recorded into a plan; a v1 plan; an `edit apply`
  plan; or a typed command, whose routes are its own `--accept-change ID`
  and `--reject-change ID` (#1310)). The structured `next_actions` (#1147)
  come with every such refusal, a v2 insertion-side one included, and with
  no other error: an anchor that merely quotes a refusal's words gets none.
  The `revision_refusals` data records the syntax its message quotes,
  `typed_action` for a typed command. Each JSON route pastes into
  its plan as written: ids are JSON-escaped, an `edit apply` route names
  both `accept` and `reject` lists, and its comment carries the refused
  operation's own `target`. An anchor that reaches into another author's
  deletion gets this refusal instead of `ANCHOR_NOT_FOUND`, copied with or
  without the deletion's marks, with a tab or a lookalike character. No
  refusal says to resolve the change first. A deletion restore takes is
  offered `restore` first ("put it back visibly"), in operation flags and
  v1 plans.

- **Edit plans: a `paragraph` key on a comment-thread operation is refused.**
  `reply_comment`, `resolve_comment`, `edit_comment` and `delete_comment`
  select a comment, not a paragraph, and used to ignore a stray `paragraph`
  silently; they now refuse it as an unknown field (`INVALID_PLAN`), as
  `list`, `watermark`, `settings`, `fill_control` and `page_setup` already
  did. Discovery publishes every v1 operation as closed
  (`additionalProperties: false`), with the bounds the dispatcher enforces.

- **Tables keep the borders Word passes; a Markdown table takes Word's thin
  grid.** The agent view prints a table's own borders (`borders single 0.5pt
  000000`), its rows' own (`row-borders … in t0.row1`, `w:tblPrEx`) and its
  cells' (`cell-borders … in all`, a row, or a cell), theme colours included
  (`theme accent1 tint 99 shade BF`), and the writer writes them back as they
  were. A Table Grid the source defines with its own uniform border keeps it. A table without a style is
  `TableNormal` (written with no `tblStyle`); a table that names nothing, as a
  plain Markdown table, takes Table Grid (single 0.5pt borders all round, as
  Word's Insert Table makes it) instead of the borderless Normal Table.
  `debug --check render` reports both kinds of border.

- **No PDF comment note by default.** `--revisions conventional` and
  `custom` no longer write a `/Text` annotation per comment: viewers
  painted it as a note icon over the commented words. The balloon still
  shows every comment; `PdfOptions::comment_notes` brings the note back.

- **The view nests list items by indentation.** An item past its list's
  first level is indented to the content column of the item it nests
  under, and its id line with it, so the Markdown reads as a nested list
  at any depth. `level N` stays on the id line only when indentation
  cannot say it (a list that starts deep, or a skipped level, which adds
  no indentation). Reading back, an HTML block indented inside a list item
  no longer leaves a stray empty paragraph.

- Python: `Document.accept`/`reject` raise `ValueError` for an id that is no
  change id or names two changes (it was `JubarteError`), and the CLI exits
  2 for it, as the binary does; a resolution that fails otherwise says
  `accept failed: …`.

- **`capabilities` prints a text summary**: one line per operation, then
  limits, rule sets, formats and id spellings; `--json` prints the manifest.
  The manifest gains `formats` (the extensions each task writes) and `ids`
  (the id spellings of revisions, comments, paragraphs and tables, by kind;
  a table's `tN` is `read -p`'s, a cell is `tN.rowR.cellC`). Python and npm
  print the same text.

- **`diff` writes text only.** It prints the paragraph patch, a text
  view (`--format github|word|normal|context|side-by-side`) or, for two
  Markdown documents, `--format critic`'s CriticMarkup, on stdout or in
  `-o FILE` (`.md`, `.markdown`, `.txt`, `.patch`, `.diff`; any other
  extension writes the same text), with `wrote FILE (md, N bytes)` on
  stderr and nothing on stdout. Without `-o` it writes no file: the
  `OLD_v_NEW.docx` it used to write beside a Word side is gone. `-o
  FILE.docx`, `.pdf` or `.png` exits 2 naming `compare OLD NEW -o FILE`;
  `--mode`, `--detail-threshold`, `--reference-doc`, `--critic`,
  `--resource-path`, `--revisions`, `--revision-palette`, `--dpi`,
  `--pages`, `--move-comments` and `--changed-pages` are compare's and
  convert's, and diff no longer takes them. `--format critic` with a Word
  side exits 1, naming compare; with `--author`, `--datetime` or `--columns`
  (the patch's) it exits 2. The patch's owner is `Jubarte` unless
  `--author` says otherwise (git's `user.name` is no longer read). An
  existing `-o` file exits 1 before anything is read. The Python and npm
  CLIs do the same, and their patch headers now name the paths as typed,
  as the binary's always did. `compare` (and `jubarte A B`), `accept`,
  `reject`, `append` and `scrub` note the retired `--changed-only` as
  convert does.

- **`edit` and `add` write one file, `FILE_modified.docx`**, beside FILE:
  the redline, or in `--editing-mode` the result. `-o FILE.docx` names
  it (another extension exits 2, naming `read`/`convert`); `--pdf` and
  `--png` write `FILE_modified.pdf` and `FILE_modified-page-NN.png`
  beside it. `--out-dir [DIR]` writes the bundle as before (clean,
  redline, `patch.diff`, `report.jsonl`), DIR defaulting to
  `<stem>.edit`; it excludes `-o`. The view names the file it read
  (`source: FILE_modified.docx`) and the `wrote` line and notes go to
  stderr, the bundle's too. A FILE already named `X_modified.docx` keeps
  its name, so editing it again needs `--force`. An existing output exits
  2 without `--force`; the plan runs first, so a refused plan still
  prints its report. `parse_cli_json`/`parseCli` hand the hosts the
  resolved `output` or `out_dir` (both null while recording into a plan),
  and the Python and npm CLIs write there.

- `edit` and `add` take `--plan` without a value: `<FILE's dir>/<stem>_plan.json`,
  for recording, appending and applying (`--consume` too); the summary's
  apply line spells it bare (`jubarte edit a.docx --plan`). A Word
  document's name as `--plan`'s or `--out-dir`'s value exits 2, saying
  FILE was read as the value.

- **`compare A B` prints the redline's agent view**, as `jubarte A B`
  does; it writes `A_v_B.docx` only when `-o` names it. Its `wrote` line
  moves to stderr and names the format (`wrote r.docx (docx, N bytes)`),
  in the Python and npm CLIs too.
  The Python and npm CLIs print the view too, and write the `.md` view,
  `.pdf` and `.json` (Python `.png` too); both refuse the Markdown-input
  options for now. A host without `read`
  refuses a compare that prints the view.

- `jubarte --help` lists the tasks; the shorthands' options are in
  `jubarte compare --help` and `jubarte read --help`.
  `--powertools-faithful` is hidden (`--mode powertools` stays).
  `jubarte FILE` refuses compare's own options (`--author`, `--mode`,
  `--critic`, the Rendering group, …) with exit 2: they took no effect.

- `accept`, `reject`, `append`, `scrub`, `convert` and `read -o` say
  `wrote PATH (KIND, N bytes[, P pages])` on stderr, or
  `wrote DIR/STEM-page-NN.png (png, K pages, D dpi)`; `-q` silences it
  (and `diff`'s line too). `convert` and `append` said it on stdout;
  `accept`, `reject` and `scrub` said nothing before.

- `convert --changed-only` (and the library's `PdfOptions::changed_only`)
  also keeps the pages a comment is anchored on and the first page of each
  section on which a revised header or footer paints. The comment listing's
  pages number on after the document's last page.

- `-q`/`--quiet` is one global option, after the task
  (`jubarte compare a.docx b.docx -q`) or before it (`jubarte -q read
  a.docx`); every other option goes after the task. Saying it twice
  (`-q -q`, `-qq`) is still quiet, and `jubarte -q help` prints the help.

- `validate` prints findings in one order every run (part, location, code,
  message); two findings at one location used to swap between runs.

- `diff-render` is `visual-diff`.

- `convert` and `diff` share one Rendering group (`--revisions`,
  `--revision-palette`, `--dpi`, `--move-comments`, `--changed-pages`,
  `--pages`), refused with any output that is not a PDF or PNG. `-o`'s
  extension (or `--to`) picks the one format convert writes: `-o X.png`
  writes PNG pages only, and convert no longer writes a PDF and PNGs in
  one run. `--pages` narrows a PDF too, and each PNG is named by its
  document page (`X-page-03.png`), also under `--changed-pages`.

- **Recording a plan checks it without applying it**, and a recording
  pins an unpinned plan and never narrows its filters (a recorded
  selection cannot join ids into an accept or reject that selected by
  author, kind or all).

### Deprecated

- `diff-render`, `text` and `inspect` (now `read`; `inspect FILE` prints
  the agent view, `inspect --json` the snapshot, `inspect --tables` the
  grid), `audit` (now `validate --rules a11y,style,structure`, which it
  still checks when no `--rules` is given), `--changed-only` (now
  `--changed-pages`), convert's `--pdf` and `--png` (now `--to pdf`/`--to
  png`; together they exit 2) keep working and print
  `note: X is now Y; X is removed in 0.14` on stderr. The shared parser's
  JSON carries those notes as `notes`, and `args.quiet` on every task.
  `jubarte --help` no longer lists them, nor `debug` (still documented in
  AGENTS.md); `self-update` is listed last.

### Removed

- `util::sha1::sha1_fingerprint`, `util::sha1::sha1_fingerprint128` and
  `util::sha1_fingerprint`, deprecated in 0.10.2 and promised to go after
  one release (#1456): call `fnv1a_64` and `fnv1a_128`. They never
  computed SHA-1.

- `comparer::finalize::promote_mid_pure_del_spacing_from_pprchange`,
  `strip_default_left_jc` and `strip_redundant_equal_spacing_pprchange`.
  Each ran all of `cleanup_spacing_and_default_jc` (M226, M228 and M231
  together), so its name promised one pass and it ran three: call
  `cleanup_spacing_and_default_jc`.

### Fixed

- **Cell borders that two groups name keep both.** The Markdown writer
  gave a cell only the first `cell-borders` group that named it. Every
  group that holds the cell now gives its sides, and a later group's
  side replaces an earlier one's. Found by the old-branch test harvest
  (#1558, #1617).

- **Border shadow and frame survive the Markdown round trip.** A border's
  `w:shadow` and `w:frame` print after its colour (`single 1pt 2F5496
  shadow frame`) and are written back. Before, the view dropped them
  (#1558).

- **Resolved Markdown verifies against the md→docx reader** (#1240).
  Resolving CriticMarkup splices the kept side of every mark back into
  the source and lets Markdown re-read it, and the splice is not
  compositional under CommonMark's delimiter rules: a kept `_b_`
  inside `*a…c*` read as literal underscores, `*a{++*b*++}c*` read as
  a bare `b`, `***a*b*c***` read as italic plus a stray `**`, so the
  md→md route and the docx route (`convert --to md --track-changes
  accept|reject`) disagreed for a seam inside a formatted span.
  Resolve now verifies every inline block holding a mark against the
  md→docx reader's own run model. A candidate that already reads like
  the model stays byte-identical; one that does not has the smallest
  inline span holding the mark re-emitted from the model as a
  common-format interval tree (`**a*b*c**`), the user's delimiter
  flavour first and the writer's second, every try verified before the
  next, and the block's whole inline content is the last resort with
  its block syntax kept. Where the reader's model is not a truth —
  a comment or a highlight inside a change, a mark holding a line
  break, a change named as nested inside another
  (`{~~old~>new~~}{r4+5 Bob re r3 Alice}`), an anchor the model cannot
  tie to its change, or a malformed delimiter — the splice is kept, as
  before. A partial overlap between two seam joins no longer splices
  both, which used to panic on a byte index past the text.

- **Text goes into an empty paragraph (#1263).** `edit FILE -p p9
  --content "text"`, an `insert` at `position` `start` or `end`, and a
  `rewrite` (in a first-version `--plan` and in `edit apply`) were refused
  `cannot insert text into an empty paragraph; use insert_paragraph`. Now
  they write one new run in your insertion, formatted as the paragraph
  mark (its `w:pPr/w:rPr` without the mark's own revisions or `w:oMath`,
  which only a mark takes), and leave the paragraph's properties as they
  were, under `keep` and through the comparer alike, in the body, a table
  cell, a header or a footer. An insert at `start`, a rewrite and
  `--content` go before a field or a bookmark the paragraph holds, an
  insert at `end` after it; several inserts keep their edges, each edge in
  plan order (#1303). Checked in Word: the redline opens; Accept All
  leaves one plain paragraph with the text, Reject All the empty
  paragraph.

- **A whole-paragraph edit follows a paragraph the settle merged alone
  (#1269).** Under a plan that settles revisions, `delete_paragraph`,
  `rewrite` and the other whole-paragraph operations were refused
  `PARAGRAPH_RESOLVED` for every paragraph that merged, in words that could
  repeat one id (`p0 merged into p0`). Now they apply when the merged
  paragraph has no other source. When it has, the message names each
  paragraph in its view (`p3 (as read) merges into p2 (after the plan's
  revisions settle) together with p4`), leaves out a settled id spelled
  like a source id it names, and gives a route that reaches the merged
  paragraph: a `contains` selector quoting its text (in a first-version
  `--plan`), or the `jubarte accept` or `jubarte reject` that settles the
  source as the plan does. A neighbour whose own text the settle changes
  still counts as a source of the merge, so an operation on one paragraph
  never removes another's words.

- **An insertion stays beside the bookmark, comment and permission markers
  it meets (#1156).** Before, an `insert` before a bookmarked word put the
  bookmark's start inside the new `w:ins`, and the edit was refused
  `CANDIDATE_INVALID` (`BOOKMARK_CROSSES_REVISION`). Inserting a word
  without a space, or an insert under `keep`, put the new words inside the
  bookmark, so Accept All grew the bookmark over them. Compare also dropped
  a permission range beside the insertion. Now the new words go before an
  opening marker and after a closing one, a comment's reference included,
  in the clean copy and in the redline alike, beside a tab or a break as
  beside text. Each range covers its source words after Accept All and
  Reject All. A replacement at a range's edge stays inside the range. A
  marker inside a word the edit touches keeps its place, a comment's
  beside a replacement's new words too: only one inside the replaced
  text goes into the deletion. Such an edit's redline is written directly, as
  under `keep`. A plan with `update_fields` still goes through Compare, so
  words typed at a heading a TOC lists refresh the TOC. When Compare carries
  the marker into an insertion, or Reject All of its redline would drop a
  bookmark, comment range or permission range of the source (a heading
  replaced whole, whose TOC entry would read "Error! Bookmark not
  defined."), the plan is refused with `CANDIDATE_INVALID` and the route
  (drop `update_fields`, then `jubarte convert FILE --update-fields`).

- **`read --json` and `--tables` take the view's options (#1158, #1255).**
  `--comments` and `--dates` were refused with `--json`, `--tables`,
  `--for-edit` and `-o FILE.json`, and `--no-page-markers` was dropped
  with a warning that broke `2>&1 | jq`; they shape only
  the view, so these take them and change nothing (the output is byte for
  byte the same). `--track-changes accept|reject` with `--json`,
  `-o FILE.json` or `--tables` settles the text of every paragraph and cell
  (`reject` reads the text and formatting Word's Reject All leaves: a
  `w:rPrChange` or `w:pPrChange` gives back what it replaced), keeps every
  id, and the snapshot names the choice in a top-level `track_changes`.
  `--tables -p` reads the selection as the view does (#1161): `t1` is the
  view's `t1`, a `pN` or a range inside a table reaches that table, and it
  prints the grids of the tables reached, each with the tables nested in
  it, then an empty line and `read -p` of the same selection, unchanged. A
  pick that reaches no table exits 2 (`p0 is not in a table; --tables -p
  takes tN, or a pN inside a table`); what the view refuses fails as
  `read -p` does. `jubarte FILE -p q7` (the one-document shorthand) exits 2
  for a selection that does not parse, as `jubarte read FILE -p q7` does.

- **A bound content control in a text box keeps its text in Word**
  (#1427). A cover page's Abstract, bound to a custom XML store, came
  back empty: the drawing kept the control's binding but not the store
  it names, and Word fills a bound control from its store. The writer
  now builds every store a plain text, date or list control's binding
  names and the main part does not reach, and drops any other binding
  so Word keeps the control's text. `jubarte validate` reports a
  binding with no store as `DATA_BINDING_WITHOUT_STORE` (a warning).

- **A text box's paragraph style survives Markdown** (#1423). The
  `styles:` legend listed only the styles of the body's own paragraphs,
  so a style used only inside a text box (a cover page's `No Spacing`)
  was left undefined in the docx written back, and the box's paragraph
  fell back to Normal. The legend now describes it too.

- **A header's or footer's paragraph indent survives Markdown** (#1422).
  A story paragraph's own `w:ind` was dropped, so the footer text next to
  a framed page number ran under the number. It now prints in the
  paragraph's entry (`p1: {text: …, indent: {first-line: 0.25in}}`) and
  converting back writes it.

- **A header's or footer's frame survives Markdown** (#526). Word's
  page-number frame (`w:framePr`, the number at the margin's edge) was
  dropped, so the number came back as an ordinary line of the header.
  A framed paragraph now prints paragraph by paragraph with its frame
  (`p0: {text: "{PAGE}1{/}", frame: {wrap: none, hAnchor: margin,
  vAnchor: text, xAlign: right, y: 1}}`), and converting back writes it.
  A frame naming the same attribute twice does not read, and the story
  entry with it: an element cannot carry both values, so the view names
  neither.

- **A style's zero spacing or single line survives the Markdown legend**
  (#1421). The `styles:` legend left out a `before`/`after` of 0 and a
  single line even when the default style has some, so a heading with no
  space after came back with the body's 10pt after it. The legend now
  says `after 0pt` and `line 1` whenever they set the style apart from
  its base.

- **The document's default spacing stays its default through Markdown**
  (#1420). The writer put the default paragraph style's spacing on
  `Normal` itself. A table style's own spacing (every built-in one sets
  none after, single) wins over the document default in its cells, but
  not over `Normal`, so each cell took the body's spacing and the tables
  grew: an 11-page document came back with 13 pages, in Word as in
  jubarte. That spacing is now written as `w:pPrDefault`, where Word
  keeps it.

- **A table style whose borders vary keeps them through Markdown** (#1419).
  The view named such a style `Ledger, borders vary`, which the writer
  could not rebuild, so the style came back without borders. A style of
  the document's own now lists each side it sets
  (`Ledger, borders bottom double 0.75pt`, with `#RRGGBB` and `space Npt`
  when they apply), and converting back writes those sides. Word's own
  table styles still say `borders vary`, except a Table Grid whose
  borders are not Word's thin grid: it is the document's own and lists
  its sides (`TableGrid, borders top double 3pt bottom double 3pt`).

- **A story's deleted or inserted last paragraph is a patch hunk of its own again.** The view writes that paragraph and the mark before it as separate changes, and since the settled dialect (#1135) the patch had joined them to the paragraph above (`@@ [p1] @@ Kept.[-\n\n-][-Last.-]`). `diff` and `patch_redline` now give `@@ -[p2] @@ [-Last.-]` (or `@@ [p2] @@ {+Last.+}`), also when Word split the paragraph into several marks or a line break sits in it. When another author changed the mark, accepting that change alone joins the paragraphs, so the mark stays a change named after its author.

- **Paths the CLI records for files it has not written yet are canonical** (#1244). The `wrote` summary, the receipts and the error envelopes name such a file by its deepest existing ancestor's real path plus the missing tail. Windows CI printed the runner's 8.3 short name (`RUNNER~1`) instead, and a missing file in a missing directory now stays absolute.

- **Duplicate ZIP and OPC part identities are refused on every route**
  (#600). OPC part names compare ASCII-case-insensitively (ECMA-376
  Part 2 §6.2.2.3 M1.12), so two entries naming one part — a byte-equal
  duplicate or a case alias of the main part, `[Content_Types].xml` or
  `_rels/.rels` — are refused with `DUPLICATE_PART` before the name
  index can collapse them; Word refuses the same packages. A single
  case-variant name is the part, exactly as Word opens it: every lookup
  resolves under the one identity, `set_part` replaces while keeping the
  stored spelling, and `to_zip` writes one entry per identity (reserved
  parts under their canonical spelling).

- Enforce XML tag policy at package admission and checked XML APIs with active limits: defaults of 1,024 attributes, 1,024-byte attribute names, 64 MiB values and 128 MiB start tags, plus duplicate expanded-name refusal. Preserve permissive comparison tree construction and linear bulk attribute insertion. Markdown conversion enforces explicit `MarkdownOptions::input_limits`, including on best-effort auxiliary parts; comparison-to-Markdown carries active limits. Ring-1 reports Word-fatal `DUPLICATE_EXPANDED_ATTRIBUTE` with part and element (#650).

- **Package writes plan against ZIP32 and never write ZIP64** (#604).
  Every writer — `PartFs::to_zip`, the Strict→Transitional repack, the
  Markdown default template — runs `zip32_plan` on the names and
  uncompressed lengths it is about to write, before `ZipWriter::new`.
  A package that would need ZIP64 — a part, a header offset or a
  central-directory size reaching `0xFFFFFFFF`, or 65,535 entries (the
  ZIP64 count marker) — is refused up front with `OUTPUT_LIMIT` instead
  of failing after the bytes are written. The Strict→Transitional
  normalizer now returns `Result<Vec<u8>, opc::Zip32Limit>` (a breaking
  change for Rust callers of `strict_to_transitional_docx` and
  `strict_to_transitional_docx_within`) so the
  refusal reaches the caller rather than handing back untranslated
  bytes; its zero-churn pass-through is unchanged. Written entries stay
  Deflated with version-needed 20 and no ZIP64 fields, asserted per
  entry in both headers.

- **`convert --to md --track-changes accept|reject` keeps the spans at a
  seam** (#1220). Resolving CriticMarkup in Markdown could leave two
  same-kind inline spans touching, which CommonMark then read as one
  span with the inner delimiters as text (`` `a``b` `` is a code span
  `a``b`, not `a` and `b`), could lose text behind a chain of marks,
  merge a kept body of a different kind into its neighbours (`*abc*`
  for `*a*{++`b`++}*c*`), or panic on non-ASCII spans. Resolve now
  joins same-kind spans that touch across anything resolving to
  nothing — code joined on its CommonMark contents and re-fenced, a
  different-kind body never merged, chains joined transitively, and a
  `_` run landing intraword or touching a run of the same character
  rewritten `*`/`_` to match — so the md→md route agrees with the
  docx route in both directions.

- **Dual-header ZIP packages are refused** (#594, GHSA-m665-v7pm-7grj). zip is bumped to 9.0.5, and every package open compares each entry's raw name in its local file header with its central record. A disagreement, or a central record with no local header, is `ZIP_HEADER_MISMATCH` on every route. `jubarte debug` reports it as a finding instead of refusing. A local time that differs from the central record is still accepted, as Word accepts it.

- **`accept`/`reject` keep untracked text under an orphan move.** A
  `w:moveTo` whose `w:name` range has no `w:moveFrom` counterpart (or a bare
  move element with no range at all) was treated as moved content: Reject All
  deleted every run inside the orphaned range markers, wrapped or not.
  Orphan move elements now resolve like `w:ins`/`w:del`, and on Reject the
  markers of an orphan `moveTo` range are dropped before the ranges are
  reversed, so `jubarte reject` keeps `Keep | B` on the #1235 probe, as
  Word's Reject All does. An orphan `moveFrom` range still deletes its
  unwrapped content on Accept, as Word's Accept All does (#1243: the first
  version of this fix dropped those markers too and kept the content).
  Well-formed moves (a `moveFrom` source and `moveTo` destination sharing a
  `w:name`) are unchanged.

- **Structural ZIP inconsistencies are refused** (#644). The same
  local-vs-central pass now compares the compression method, flag bits 0
  and 3 (bit 11 only when a raw name is non-ASCII — an ASCII name
  decodes identically with the bit set or clear),
  and — with bit 3 clear — the CRC-32 and ZIP64-resolved sizes (still
  `ZIP_HEADER_MISMATCH`); validates every entry's extent — local header,
  payload and data descriptor — for overlaps and placement before the
  central directory (`INVALID_PACKAGE`); checks each data descriptor,
  signed or unsigned, in the width the local header's ZIP64 form
  prescribes — a 0x0001 extra field alone prescribes 8-byte sizes — or
  the other width when only it matches the central record; and
  reads directory payloads through the bounded CRC stream so a corrupt
  one refuses and its bytes count against the budgets. `jubarte debug`
  reports each class as a `zip-header-mismatch`, `zip-extent`,
  `zip-descriptor` or `zip-directory-payload` finding instead of
  refusing, and an entry whose open or payload read still fails becomes
  a `zip-entry-read` finding of its own, so the structural findings
  always reach the report.

- **ZIP entries recorded as non-regular types are refused** (#639). The
  shared package open reads the raw central record's version-made-by and
  external attributes: on a Unix or OS X host the entry's recorded type must
  be a regular file, or a directory on a `/`-terminated name, and on any
  host the DOS directory attribute demands the same — a symlink, device,
  FIFO, socket or a directory type on a file name is `UNSUPPORTED_PACKAGE`
  on every route, and `jubarte debug` reports it as a `zip-special-entry`
  finding. `jubarte self-update` also lstats the extracted binary before
  installing it, closing the symlink a tar release archive can plant where
  the zip path already refuses one.

- **`accept`/`reject` settle tracked changes inside comment bodies**
  (`word/comments.xml`), as Word's Accept/Reject All do. An `--author` or
  `--kind` selection resolves them too; `--id` still cannot name one (they
  carry no listed id), so a selection that leaves any prints a note with
  the count. A comment whose revisions empty its paragraphs keeps one
  `w:p`, the shape Word writes.

- **Fractional plain values on table and frame geometry are rounded in
  redline output — a validity repair.** A gdocs-style fractional dxa
  (`w:tblpX="100.5"`, `w:framePr w:x="33.5"`) is schema-invalid on the
  integer-typed attributes the Strict measure whitelist reads
  (OpenXmlValidator: not a valid Int32), so `convert`'s output sweep rounds
  them (101, 34) as it already did for indents, spacing and cell margins.
  `jubarte debug` still reports the literal value: a diagnostic describes,
  it does not repair.

- **Writing an output works on Linux again (#1351).** cap-std opens
  directories `O_PATH` there, and `fsync` on an `O_PATH` descriptor fails
  with EBADF. Every write reported `Bad file descriptor (os error 9)` and
  "durability uncertain" after the file was already in place. The
  directory sync now opens `.` through the handle it holds, which walks no
  path, so root confinement still holds.

- **The view keeps a Word highlight's colour.** A highlight that is not
  yellow prints as `<mark class="hl-green">…</mark>` (the 16 names Word
  gives `w:highlight`), and the writer reads the class back, so green,
  cyan and the rest round-trip instead of collapsing into yellow. A
  stretch the view escapes (`<mark>file\_</mark>`) comes back as one
  highlighted run, and `debug --check render` no longer counts a
  highlight on a run that paints no glyph (a deleted page break) or
  misses a Strict package's direct highlights.

- **Literal CriticMarkup delimiters in a code block stay text.** A
  `SourceCode` paragraph whose text holds `{++x++}` printed it into its
  fence as is, and the next convert read it as a tracked change. The views
  now break such delimiters for the fence, and only there, where Markdown
  reads no backslash escape: a delimiter shape with a run of backslashes at
  its break point (`{` + the run + `++`, `++` + the run + `}`, `~` + the
  run + `>`) is written with the run made odd and read back halved
  (`{\++` for literal `{++`, `{\\\++` for literal `{\++`), every other
  backslash staying as written. The text comes back exactly, tracked
  changes inside a code block still round-trip (their delimiters are never
  escaped by a backslash before them), and hand-written code such as
  `C:\\path`, a regex `\\d+` or LaTeX `\\` reaches Word byte-identical
  instead of losing half its backslashes.

- **`--commonmark` escapes CriticMarkup and Markdown syntax.** A document
  without tracked changes printed its text as is, so Word text such as
  `{++a++}`, `\{++a++}`, `- not a list`, `1. not a list`, `# not a
  heading`, `> not a quote`, `*not em*`, `` `not code` ``, `[not](link)`
  or `<b>not html</b>` came back as real changes, lists, a heading, a
  quote, emphasis, code, a link or HTML. The plain conversion now escapes
  both, as the agent view always did, and round-trips as itself.

- **A header or footer keeps its own text colour, borders and alignment.**
  The view prints a story whose paragraphs carry them paragraph by paragraph
  (`id: header1`, then `p0: {text: …, center, color: "595959", border: bottom
  single 0.5pt 8496B0 space 4pt}`), and the writer writes each back. Demo.docx's
  grey, ruled header had come back as plain black text, and a footer whose
  second paragraph was left-aligned came back centred.

- **Markdown views keep a section's column line and line grid.** `page:` and
  `sections:` print `column line` (`w:cols w:sep`) and `line grid 15.6pt`
  (`w:docGrid`; also `line and char grid`, `char grid`, `char space N`), and
  the writer writes them back. Demo.docx's two columns had lost their
  divider, and every line had moved off Word's 15.6pt grid.

- **A highlight prints as `<mark>`, so `{==…==}` is only a comment's
  scope.** With `==` for both, a highlight at the edge of a comment's scope
  printed `{====lit==…==}` and came back as the literal text `==lit==`, and
  text ending in `=` beside a highlight (`a===b==`) lost the highlight.
  Hand-written `==x==` still reads as a highlight. Edit anchors drop
  `<mark>` and `<s>` as they drop `<u>`.

- **Underline, highlight and strikethrough keep their spaces.** The view
  kept a formatted stretch's edge spaces outside `<u>`, `==` and `~~`, so a
  round trip broke the underline and the highlight at every space (and a
  form's underlined blanks came back plain). The spaces now stay inside the
  marks as `&#32;`; a strike `~~` cannot open or close falls back to
  `<s>`, and a plain space between underlined words stays plain.
  `debug --check render` counts a highlighted or underlined stretch once,
  however many runs Word split it into.

- **Markdown written to Word reads back as it was written** in five more
  shapes the round trip lost: a link inside one tracked change stays one
  change (a HYPERLINK field in its mark, its title and target escaped as
  field arguments, its field in one mark when the change has two); a
  bulleted heading keeps one bullet (`## - X`, not `## - - X`); Word's
  built-in Grid and List Table styles (`GridTable4-Accent1`) are written as
  Word defines them, coloured from the document's theme, so banding and
  borders survive; and a field across paragraphs, such as a table of
  contents, keeps its code: `{TOC \o "1-3"}` begins its first entry and
  `{/}` closes after its last.

- **Markdown written to Word reads back as it was written** in four more:
  a comment range that ends before its notes' paragraph (`in c2` left off
  the id line) stays out of that paragraph; a comment whose note sits inside
  a tracked change (`{++{>>c1 …<<}more++}`) stays in the change; emphasis
  whose `**` or `_` could not open or close where it stands (`0.081*`
  bolded) reads `<b>\*</b>`, so its neighbours keep their own marks; and a
  run of tracked-change markers beside one emphasis or link joins it
  together or not at all, so a raised link of several runs stays one link.

- **The npm CLI does what the binary does where it said it could not.**
  `convert` turns Word into Word (`--track-changes accept|reject`,
  `--update-fields`, `--report`) and into its paged Markdown (`--to md`,
  `-o x.md`/`.txt`, `--no-page-markers`); `edit --pdf` writes the PDF beside
  the document (and `redline.pdf`/`clean.pdf` with a `render` report line in
  a bundle), taking `--revisions` and `--dpi`. An unreadable input prints
  the binary's `CODE: FILE is …` line (`INVALID_PACKAGE`, `LEGACY_DOC`); a
  plan that does not read prints only `error: INVALID_PLAN: …` (exit 3, no
  report); a refusal's summary keeps `"operation": null`; and
  `capabilities` answers no for what the CLI refuses (append, scrub,
  validate, repair, audit, comments, visual diffs, page ranges, PNG) and
  lists formats only for its own tasks.

- **`-q` says what it hides**: status lines (`wrote`, the `SUCCESS`
  block and an edit's summary), not what a reading task prints.

- **The Python and npm CLIs honour `convert --tracking`.** Both accepted the
  flag and dropped it; Python `from_markdown(tracking=…)` and wasm
  `markdownToDocx` `tracking` now set Word's Track Changes switch, and one
  JSON options reader (`jubarte::markdown::docx_options_from_json`) serves
  both bindings. `_native.markdown_to_docx` takes `options_json`.

- `edit` under `existing_revisions: "keep"`, and a plan rewriting what a
  comment covers (`c0.text`), dropped a `format_run`, `insert_footnote` or
  `page_setup` from the redline while the clean copy had it and the report
  said `ok`. A `format_run` is a tracked format change (`w:rPrChange`)
  now; under `keep` the operations it cannot track are refused, naming
  them, and a `c0.text` plan holding one goes through the comparer.

- Replacing exactly the text a comment covers kept the deletion inside the
  comment only in a plan of text edits: with paragraph formatting beside it,
  or `whole: true`, Reject All left the comment on nothing. `update_fields`
  beside such an edit is refused, as under `keep`, and both refusals name
  `jubarte convert FILE --update-fields`, not the removed `fields update`.

- A retired spelling's note was printed only when the run parsed: `compare
  … --changed-only` with a usage error, or `audit FILE --rules edits`
  without `--original`, printed the error alone. The note now leads the
  error on every host (npm's `audit`, which has no `validate`, included).

- `audit FILE --rules …` was told to use `validate --rules
  a11y,style,structure`, which drops the rules it asked for. With
  `--rules` given, the note says `validate` alone.

- A `.docx` that is not a ZIP read seven ways across the tasks (`read`
  said "Could not find EOCD", `diff` said "compare failed: cannot write
  the document"). Every Word input now reads `INVALID_PACKAGE: FILE is
  not a ZIP file: no end-of-central-directory record`, as `LEGACY_DOC`
  already did.

- `\newpage` (and `\pagebreak`, `\clearpage`, `\cleardoublepage`) on a
  line of its own printed as text in the Word output; it is now a page
  break, which the view writes in its own words (`page-break` on the id
  line) and plain `convert --to md` as a `<!-- page-break -->` line, which
  reads back as the break. Plain Markdown dropped page breaks before.

- A task the host lacks (npm's `visual-diff`, `validate`, `scrub`, …) said
  "not a task", "not supported by this adapter" or a clap conflict by
  argument count; it is now one usage error naming the task.

- `jubarte revisions x.docx` said only "reading revisions: No such file":
  the removed task's name reads as the first file to compare. A note now
  names `changes` first, on every host.

- A plan's `"comment_id": "c5"` was refused. (`#c5`, which an unquoted
  shell reads as a comment, is now refused too; write `c5`.)

- `changes` listed a text box's change twice, once per
  `mc:AlternateContent` branch.

- `convert --changed-only` named PNG pages by their index among the kept
  pages (document page 3, kept alone, was `-page-01.png`).

- `convert -o x.json` wrote a PDF, `diff -o x.json` wrote Word, `diff`
  wrote `OLD_v_NEW.docx` beside a Word input without `-o`, and `convert
  --pdf -o x.png` wrote a PDF named `x.png`.

- `edit`: replacing exactly the text a comment covers keeps both the
  deletion and the insertion inside the comment's range, so Accept All
  leaves the comment on the new text and Reject All on the old. The
  redline used to put the insertion before the range and the deletion
  after it.

- Markdown to Word, notes: when a footnote and an endnote (or two notes)
  share a label such as `[^1]`, the first reference names the first
  definition and each later reference the next one, each a note of its
  own, with a warning. A label defined once and referenced twice stays one
  note.

- Markdown to Word: a change by `@??` (or `??: ""` in `authors:`) is
  written back with an empty `w:author`, which Word opens.

- Heading bookmarks: the reader and the writer share one github-slugger,
  and only the exact name the writer gives a heading reads as its own
  bookmark; any other (`title-01`, a lone `title-1`) stays an `<a id>`
  anchor.

- The agent view hides a run whose character style has `w:vanish`
  (directly or through `basedOn`), as it hides directly hidden text.

- A table row whose cells and gaps overflow the grid and that holds an
  empty cell gets per-row paragraph counts on the table line
  (`<!-- tN … cells tN.row0 p1-p3 tN.row1 p4 … -->`), so the empty cell
  comes back.

- An empty or null `document_owner:` is ignored with a warning ("names no
  one"); the reader takes the first non-empty `dc:creator`, keeping its
  spaces but not the XML's indentation.

- An author dated with a range `first..last` keeps it across the round
  trip (the last day lands on a formatting mark, an inserted row or a
  header mark); a bare day reads as its midnight, and a reversed range
  warns ("its days run backwards") and writes the marks undated.

- A comment inside one side of a substitution
  (`{~~old~>{==new==}~~}{>>c0 …<<}`) stays one substitution.

- **PDF columns divide the body only, and `w:sep` draws the rule.** A
  header and footer span the page in a columned section, the column
  separator Word draws (`w:cols w:sep`) is painted, and a face taller
  than the line grid snaps to its own grid lines or the multiple's,
  whichever is more. (#1644)

- **A simple PAGE or NUMPAGES field in a header or footer is
  computed.** A `w:fldSimple` field showed its cached result ("Page 1 of
  1" on every page of a two-page document); only the complex form was
  computed, and the Markdown writer emits the simple form for `{PAGE}`.
  (#466)

- **Strict OOXML packages read and compare as themselves.** A
  package's dialect is read from the root tag wherever it lies, only
  namespace declarations are translated and only in Strict packages,
  attribute values that wholly are Strict URIs translate, and Strict
  universal measures are normalized on both inputs — columns and float
  geometry included — before the diff, so a Strict document compares
  equal to its Transitional twin. (#536, #576)

- **Tables pair by shared words when no row is identical.** The
  comparer pairs single rows across partitions when they share a word,
  separates row words at cell and paragraph boundaries, and keeps wrapped
  rows, re-spanned cells and row content order when merging tables, so a
  table whose every row changed still aligns as Word aligns it. (#497)

- **A table property change records the grid; an unchanged table
  records nothing.** A `w:tblPr` change now records its `w:tblGrid` too,
  as Word does, and an unchanged table no longer carries an empty
  `w:tblPrChange` or `w:tblGridChange` into the redline. (#434)

- **`validate` flags five more shapes.** Nested text boxes are
  flagged Word-fatal (the CLI warns when a document holds one, and the
  Markdown route edits their stories), nested deletion text is detected
  with measured nesting severity and written through nested insertions,
  duplicate revision ids across markup kinds are rejected (ids scoped to
  SDK record types, fresh ids for copied property histories), unequal
  move range counts warn, and `STYLE_NAME_COMMA` flags a comma in a
  style's `w:name` the writer cannot rebuild. (#491, #1132, #1134,
  #1137, #1153, #1214)

- **Permission ranges and move ranges survive a compare.** Ranges
  belonging to the same move join, permission ranges keep apart across
  structure (empty body pairs placed inside the next paragraph), and
  deleted permission ranges settle as Word settles them — endpoints at
  their source offsets, restoration indexed by source occurrence —
  through edits and settlement alike. (#1214, #1215)

- **A comments part with no revision is left as it is.** The
  comparer's comment tidy no longer rewrites a `word/comments.xml` that
  holds no revision — the untouched part skips the tidy too — so comments
  no plan touched come back byte-identical. (#1467)

- **Redacted text stays out of reports and confirmations.** A
  `redact` operation's report and confirmation are rebuilt from safe
  fields, with the original document's view omitted, so the words a plan
  removed cannot come back through the report channel. (#1196)

- **PNG pages draw no box at a comment's anchor.** A page image
  painted a box where a comment anchors, marking text the page does not;
  the anchor's box is gone. (#482)

- **rustls 0.23.46** fixes RUSTSEC-2026-0346 and RUSTSEC-2026-0347
  (rustls-webpki 0.103.16 and webpki-roots 1.0.10 with it). (#475)

- **A `notes: rev:` clause warns that it is not written**, so a view
  that carries one says what was dropped instead of staying silent.

- **`&#32;` reads as a space to edit anchors, and a paragraph's own
  leading spaces print as `&#32;`.** A find, an anchor or `--content`
  copied from the view reads the reference as the space it stands for,
  and a `&#32;` is a space to the anchor's emphasis rules too.

- **A tracked page break names its change on the id line.** (#503)

- **The separators' formatting changes round-trip through `notes:
  {fmt: …}`.** A footnote or endnote separator's formatting changes read
  back from the `notes:` clause and are written again, with the
  separator's metadata preserved and revision ids reserved.

- **A header keeps the empty paragraphs after its text.** (#506)

- **Section references print in Word's order.** (#489)

- **Story character style kinds survive the round trip.** (#869)

- **A code fence's closing line stays clear of a span opener**
  (#991), an indented code block is verbatim (no fence or CriticMarkup),
  and a note anchored in a code block keeps its text.

- **Table-style names that hold a comma are quoted and never written
  into `w:name`; outline levels 7–9 and an empty paragraph's stated
  outline level survive the round trip; a named clause renames a built-in
  table style.** (#593)

- **Style fidelity: Word's older table styles, `LineNumber` and
  renamed note styles are written as Word defines them; a stated default
  style, a section's stated start type, a switched-off `titlePg`,
  keep-lines and indents, and a theme font in the defaults come back as
  stated.** (#485)

- **Revision timestamps report in the recorded local offset** (UTC
  linked for comments); the view writes the UTC instant beside Word's
  wall clock for offset dates, and UTC is preferred when reading tracked
  changes. (#1118)

- **A retried insertion point leaves no stale revision refusal.**
  (#1308)

- **`--force` refuses a bundle directory it cannot read.** (#1431)

- **Publication reports cleanup warnings and failed restores, rejects
  bidi file names, and export confinement is hardened.**

- **Review-gem refinements:** control characters are escaped in
  printed ZIP entry names, entry type is judged on the Unicode
  Path-decoded name, and failed entry reads still charge the inflate
  budget; docDefaults leads the style legend's order; a styled paragraph
  keeps the outline level it states; a style change lands in the style's
  own `pPr` and `rPr`; settle hints shell-quote a colon id; and compare
  keeps the caller's limits when inputs are equal after accepting.

- **Anchor parsing: block marks lead raw lines, a list item's
  continuation indent is not anchor text, whitespace is reported near
  matches, and an escaped `&` stays literal in a find.**

- **Formatting history inside a kept change stays frozen** — also a
  cell mark in a kept formatting change's history.

- **Agent-facing paths print with `/` on every OS**, and Windows
  verbatim prefixes are stripped (a verbatim disk or UNC prefix keeps the
  root).

- **Admission tolerates unquoted attributes** (`validate_xml` stays
  strict). (#1055)

- **Absurd universal measures are refused, and page twips are rounded
  before matching a named size.**

- **Hosts: editing-mode conflicts come from the engine;
  `--update-fields` needs `-o`; PNG side files follow the writer.**

- **A PDF keeps a revised footnote on its page**, with chrome tables,
  header pictures and listing numbers pinned.

- **A side file over a PNG output names the PNG output; a table of
  backslashes round-trips (#538), and an even backslash run before a
  table-cell pipe splits the cell.**

- **Compare: a reused header keeps its own relationships, a renamed
  header copy never hides another header, and boundary bookmarks stay
  outside inline revisions.** (#827)

- **Marks without `w:id` print `noid`, and a `w:id` holding a tag
  separator stays one change.**

- **A comment anchored in a header or footer reads back unanchored,
  and header and footer changes pair as the body's do.**

- **Empty table rows are kept in the agent view to preserve row
  numbering, and a reference definition's destination is read
  literally.**

- **`read -p`, `--head`, `--tail` and `--changed` see a nested list
  item, and `edit` rejects `--accept`/`--reject` combined with
  `--reply`.**

### Security

- **opentelemetry-api 1.45.1** in `jubarte-python/uv.lock` fixes
  GHSA-xcq7-p3pf-p88h. It arrives through the `mcp` extra; only that
  package moves.

## [0.12.0] - 2026-10-11

> **Summary.** jubarte FILE and jubarte read print an agent view (a YAML header, then the document as Markdown with paragraph ids, CriticMarkup changes and comments), and jubarte A B prints its redline's view instead of writing a file (-o writes it; compare A B is unchanged). jubarte edit and the new jubarte add take their operations from flags. Text diffs come as github, word, normal, context and side-by-side views. convert --move-comments lists comments after the last page with [JR1] markers, --changed-only keeps only the pages a change touches, and convert --update-fields writes Roman and letter NUMPAGES, PAGEREF and SEQ results as Word does. Redlines no longer hang Word 16.115 on a deleted empty PAGE field or a header table only one version has, and drop headers and footers no section uses. PDF text no font has glyphs for stays searchable, and math letters painted from their ASCII italic read back as math letters. rdocx-opc 0.2 fixes RUSTSEC-2026-0194. On two 600-item samples scored against Word's own output: redlines mean 71.89 (Docxodus 64.95), counting as 0 the 23 redlines Word 16.115 crashes opening (KNOWN_ISSUES #8); PDFs mean 79.19 (LibreOffice 54.61), no failed document.
>
> **Docs.** docs/rust.md, docs/python.md and docs/javascript.md carry the regenerated CLI and API references; docs/TEXT_DIFF.md is new; examples/cli/help-cli.md and examples/cli/visualize/ show the agent view and every command's help; the library READMEs and their fragments document read, edit, add and the new PDF options; KNOWN_ISSUES.md records #9 (WASM fonts, text layer fixed) and #10 (float re-layout); docs/WORD_DIFFERENCES.md lists the open Word differences; the root plan files moved under docs/.

### Added

- `jubarte FILE` prints the agent view (`read FILE`; a lone name without
  an extension needs `read`); `jubarte A B` prints
  the agent view of their redline, with the `read` options (`-o FILE` writes
  it instead; `compare A B` still writes `<A>_v_<B>.docx`). Same on the
  Python and npm CLIs.

- `jubarte edit FILE -p WHERE --anchor TEXT --content TEXT` (also
  `--delete`, `--resolve`, `--style`) and the new `jubarte add` (a paragraph,
  a comment, or a reply to `c5`) take their operations from flags: each `-p`
  starts one, so one command carries several. `--author`, `--datetime`,
  `--existing-revisions` (default `auto`: keep a file's tracked changes) and
  `--editing-mode` (clean copy and report only) apply to all of them;
  `--out-dir` defaults to `<stem>.edit` beside the file. After applying, both
  print the changed blocks as the agent view (the patch stays in
  `patch.diff`). Same flags on the Python and npm CLIs; library:
  `jubarte::edit::flags::plan_from_flags`, `jubarte::markdown::changed_view`,
  Python `_native.flag_plan`/`changed_view`, WASM `flagPlan`/`changedView`.

- Document text diff views: `github` (`unified`, `text`), `word`, `normal`,
  `context` and `side-by-side`, across Rust, Python and WASM. Line views
  preserve existing CriticMarkup; word diff accepts all changes in both
  documents before making fresh marks. `--accept-changes`, `-U`/`--context`
  and `--full-lines` control the view. Long lines use a 70-character window
  around the first change. Text output creates no implicit Word file.

- `convert --move-comments` (also on `diff`; library:
  `PdfOptions::comments = CommentPlacement::End`): the comments leave the
  balloon pane and are listed after the last page, each with its marker,
  author and page. The page keeps its own width, the commented text keeps
  its tint, and a superscript `[JR1]` in the author's ink marks where each
  comment is anchored (a reply rides on its thread's marker). The default
  stays Word's balloons beside the text.
- `convert --changed-only` (also on `diff`; library:
  `PdfOptions::changed_only`): the whole document is laid out, then only
  the pages a change bar marks are kept, so page numbers and "Page N of M"
  stay the document's. A document without body changes keeps its first
  page. With `--move-comments`, only the kept pages' comments are listed.
- `PdfOptions` has two more public fields (`comments`, `changed_only`): a
  struct literal that names every field needs `..PdfOptions::default()`.
- Python and JavaScript take both options. In Python they are
  `PdfOptions(move_comments=…, changed_only=…)` for `Document.to_pdf`,
  `to_png` and `render`, keyword arguments of `docx_to_pdf`, and
  `python -m jubarte_redlines convert --move-comments / --changed-only`.
  In JavaScript they are `docxToPdf`'s two trailing arguments and
  `npx jubarte-redlines convert --move-comments / --changed-only`.

- `convert --update-fields` writes `NUMPAGES`, `PAGEREF` and `SEQ` results in
  `\* roman` and `\* alphabetic` as Word does: the switch's first letter
  picks the case, letters past z repeat (aa, bb, ...), the last `\*`
  switch wins, and past 780 in letters or 32767 in Roman numerals the
  result is Word's "Error! Number cannot be represented in specified
  format." A `SEQ` identifier with either format keeps counting. Other
  formats still keep the cached result.

### Security

- Upgrade rdocx-opc to 0.2 with quick-xml 0.41, fixing RUSTSEC-2026-0194
  and RUSTSEC-2026-0195 across the native, Python, WASM and in-process
  graphs, including the public `Relationships::from_xml` parser. Keep
  the 256-attribute OPC admission budget and remove the advisory exceptions
  and obsolete WASM-only dependency patch. Relationship/content-type parser
  behavior is preserved; consumers sharing dependency types must also use
  rdocx-opc 0.2 (the re-exported XML errors now use quick-xml 0.41).
- Add direct relationship-parser regression tests and fuzzing. Fuzz checks
  now block CI; native runs instrument mimalloc's C code and enable its
  AddressSanitizer allocation tracking alongside Rust instrumentation.
- Add contribution and private vulnerability-reporting policies, daily
  dependency/history-secret checks, mandatory release security checks and
  an OpenSSF criterion evidence ledger.

### Changed

- **Breaking: the `revisions` command is gone.** `changes` is the one
  listing: every tracked change in every story, with the ids `accept`,
  `reject` and `edit` take. `revisions` printed Docxodus's grouping, with
  no ids, no headers or footers, and a count that disagreed with
  `changes`. The word is a file name again (`jubarte revisions new.docx`
  compares two files); the library's `get_revisions` and the wasm
  `getRevisions` export stay.
- `jubarte A B` no longer writes a file unless `-o` is given: it prints the
  redline's agent view. `jubarte compare A B` is unchanged.
- `edit` prints the redline's changed blocks as the agent view instead of
  the patch (`patch.diff` is still written); `--plan` and `--out-dir` are
  optional.

- `jubarte read` (alias `text`) prints the agent view: a YAML header
  (authors with handles and timestamps, owner, page setup, styles,
  headers, footers, sections), an `<!-- pN -->` id line before every
  paragraph (same numbering as `edit`), tracked changes as CriticMarkup
  followed by a note with their `w:id` and author handle, comments with
  their ids and thread parents, page markers from the layout pass, and
  `-p` / `--head` / `--tail` / `--changed` (only the blocks with marks;
  `--by AUTHOR` for one author's) / `--comments none` / `--dates` /
  `--no-page-markers`. `edit` selectors take the view's short ids (`p3`,
  `header1.p1`, `t0.r1.c2`). The previous `[body:p:N]` projection remains
  available as `inspect::markdown` in the library bindings.
- CLI commands, help, aliases, defaults and usage validation now share a
  clap-derived schema across native, Python and WASM. `compare` (alias
  `redline`) is discoverable alongside the existing two-file shorthand.
- Python extensions avoid symbol stripping that can corrupt macOS
  Mach-O metadata and prevent startup.

- The desktop app is no longer tracked here: it is its own repository
  (arthrod/jubarte-app), cloned untracked at `jubarte-app/` or named by
  `JUBARTE_APP_DIR`. `scripts/release.sh` no longer bumps or commits the
  app's files. It runs the app's `scripts/release-engine.sh` instead,
  with `--preflight` at step 0 and in full as step 13: jubarte.pro, the
  app's facts, and its version files on a `release/vX.Y.Z` pull request.
  Step 14 is folded into it. `release_downstream.sh`, `check_site_live.py`
  and `check_release_facts.py` moved to the app with their tests. The
  app's CI builds the app, and the app has its own Dependabot config.

### Fixed

- Compare: a deleted or inserted field with no shown result
  (`<w:fldSimple w:instr="PAGE"/>`) stayed live in the redline, so
  rejecting or accepting kept the wrong page numbers, and Word 16.115 hung
  on such a footer. It is now tracked the way Word's own redline does.
- Compare: a header or footer that only one document has now marks its
  table rows deleted or inserted along with their text; Word 16.115 hung
  on a footer table whose cells were deleted but whose rows were not.
- Compare: a redline no longer keeps the original's headers and footers
  that no section uses once the revised document's section breaks bring
  their own. Word's own redlines carry none. Their half-matched page
  number fields made `jubarte validate` report them as Word-fatal.
- Removing a header, footer or other related part (redline, Accept, Reject,
  markdown export) also removes the images and other parts that only it
  used, instead of leaving them in the package with nothing pointing at
  them.
- `jubarte validate` no longer calls two shapes Word opens Word-fatal: a
  table cell that ends in a content control holding its last paragraph
  (its repair added a blank line to the cell), and Office's legacy
  `[trash]/` folder, which needs no content type.

- A comment anchored in a justified line lost its balloon: the stretched
  line repaints its words as new runs, which dropped the run's comments.

- PDF: list labels past z repeat the letter as Word does (z, aa, bb, ...,
  ZZ, AAA), where jubarte counted on like spreadsheet columns (aa, ab, ac,
  AZ, BA). Past 780 letter labels start again at "a", and past 32767
  Roman ones at "I", as in Word; a huge list start no longer builds a
  label millions of characters long.
- PDF: a list label wider than its hanging indent sends the text to the
  next tab stop, as Word does, instead of painting the text over the
  label.
- PDF: text no available font has glyphs for (Japanese in the WebAssembly
  build) is still left unpainted, but search and copy now read it, where
  they read nothing or NULs. Math letters painted from their ASCII italic
  read back as Word's math letters (𝑎, not a). Both use `/ActualText`.
- `scripts/release.sh` step 11 runs `gh release download` with no terminal
  attached and keeps gh's error. Under `script` (the 0.11.3 run's log
  wrapper), gh's progress display queried the terminal and failed, and the
  discarded error read as "no wheels" while the release held all seven. A
  release that lists every wheel but will not download them now stops the
  run instead of falling back to a single local wheel.

## [0.11.3] - 2026-10-06

> **Summary.** jubarte convert reads a Word 97-2003 .doc (text, headings, lists, bold and italic, tables) and marks each page in Markdown output (<!-- page N of M -->); --timeout bounds a conversion. PDFs set OMML math in Cambria Math as Word does. The font report counts a face guessed from a name's class as substituted, and a repeated identical relationship Id opens repaired. MSRV is Rust 1.94. On two 600-item samples scored against Word's own output: redlines mean 72.06 (Docxodus 65.15), counting as 0 the 23 redlines Word 16.115 crashes opening (KNOWN_ISSUES #8); PDFs mean 79.14 (LibreOffice 54.61), no failed document.
>
> **Docs.** README.md, docs/rust.md and the four registry READMEs state MSRV 1.94. docs/rust.md, docs/python.md and docs/javascript.md carry generated CLI and API references. docs/adoption/ and examples/adoption/ are new: one page per agent skill jubarte replaces, each row with runnable evidence; the .doc rows read released. KNOWN_ISSUES.md #8 records the Word 16.115 crash.

### Added

- `jubarte convert old.doc` reads a Word 97-2003 `.doc`: it reads the OLE
  compound file and the Word 97 piece table into text, paragraphs, Title
  and Heading 1-9, one-level tables (header rows kept), field results,
  bold and italic runs, and bulleted and numbered lists with their
  nesting. The default output is `.docx`; `-t md`, PDF and PNG go through
  the converted package. Encrypted, Word 6/95 and non-Word compound files
  stay `LEGACY_DOC`, and malformed structure is refused, never guessed: a
  reversed, gapped or truncated piece table, a table outside its stream,
  a repeated FAT sector, an invalid bold/italic toggle. The other commands
  still refuse a `.doc`, now with the convert command in the hint.
- `jubarte convert FILE -t md` writes `<!-- page N of M -->` before the
  first block of each page. The page comes from laying the document out
  and matching each block's opening text on the painted pages.
  `--no-page-markers` opts out. The markers are HTML comments, so the
  Markdown converts back unchanged. Block keys come from pulldown-cmark
  (images, link reference definitions, autolinks, escaped pipes, literal
  angle brackets), and a marker never splits a loose list or a fenced
  block.
- `jubarte convert --timeout SECONDS` exits 124 (`timeout(1)`'s status)
  once the limit passes.
- docs/rust.md, docs/python.md and docs/javascript.md carry generated CLI
  and API references (`scripts/gen_docs.sh`). A docs CI job fails on
  drift, and `release.sh` regenerates them.
- docs/adoption/ and examples/adoption/: one page per agent skill (Anthropic
  `docx`, OpenAI `doc`, MCP) that jubarte replaces. Each row has
  side-by-side evidence with a `run.sh`, a test in `tests/adoption.rs`, a
  plan for every gap, and the measured disk each skill's container drops.
  The `adoption` workflow runs it all.

### Fixed

- Convert, math: OMML text is set in Cambria Math with letters from the
  mathematical italic block (`m:sty p`/`b` upright, `m:nor` ordinary text).
  A display equation is centred unless its `m:jc` says otherwise. `m:d`
  draws its delimiters and separators, and they follow the script
  placement. Math is found under any namespace prefix. A face that lacks
  a letter keeps the whole run in ASCII italic. Math text drops the
  paragraph's hAnsi face. Deleted text counts toward a display
  paragraph. On the 29 corpus documents with a display equation, against
  Word's PDFs: jaccard +0.030 [+0.0005, +0.070], text_boundary +0.056
  [+0.008, +0.116] against 0.11.2; the 116-document control without
  OMML is byte-identical.
- Font report: a family placed on an installed face by its name's class
  ("Fake Serif Pro" on Times, Helvetica on Arial) was reported as an
  explicit match, so `--fail-on-substitution` passed. It is now `generic`
  and counts as substituted; metric twins (Carlito, Caladea, Liberation,
  Arimo, Tinos, Cousine) still count as their family, and a trailing
  "PS MT" is not a substitution. Layout is unchanged.
- Package: a relationship Id repeated with an identical relationship is
  kept once on open and reported as a repaired `DUPLICATE_RID`. Two
  different relationships under one Id are refused. Word refuses both.
- Command line: converting to `.docx` or Markdown refuses `--report`,
  `--font-report` and `--fail-on-substitution` instead of ignoring them.
  A page-marker warning names the step that failed, and `--timeout` must
  be finite.

### Changed

- MSRV is Rust 1.94 (was 1.88), for the library, the CLI and the Python
  binding. The MSRV-aware resolver had been holding `aes` at 0.9.2 and
  `rdocx-opc` at 0.1.0 (their next releases need 1.89 and 1.93; the WASM
  build already patched rdocx-opc 0.1.2). 1.94 also brings
  `slice::array_windows` and `str::floor_char_boundary`, which replace
  hand-rolled pair indexing and char-boundary loops. The CI MSRV job runs
  the all-feature suite on 1.94.
- The code is clippy-clean on 1.94 as well as on current stable: six
  boolean expressions 1.94's `nonminimal_bool` flagged are rewritten
  (`is_none_or`, a named pair test, a named toggle closure), and the lint
  suppressions left in examples, tests and the Python binding are
  `#[expect(..., reason = ...)]`, which fails once a suppression is no
  longer needed. `tests/common` keeps `allow(dead_code)`: each test binary
  uses part of it.
- Every CLI command but `convert` refuses a `.doc` (and any other
  admission refusal) leading with its code, `LEGACY_DOC: old.doc is …`, as
  the library and Python already report it; the convert hint follows.
- `validate --json` is described as JSON Lines in its help: one object per
  finding, nothing when there is none.

### Docs

- The adoption pages say what is true today. Python's `convert` is a
  subset of the CLI's (no `--timeout`, `--fail-on-substitution` or `-t`),
  and exit 2 also means `validate` findings. The npm CLI needs Node 18.3+,
  and crates.io builds need Rust 1.94+ from 0.11.3. The `body:rev:`
  limitation is that id-less revisions share the id, not that `--id`
  cannot select it. The MCP feature-gate note no longer reads as pending,
  and Gemini CLI issue #20298 is closed as not planned. Two claims now
  match what is tested: folder 16 alone holds Word's own answer, and
  `tests/adoption.rs` runs the rows that have a jubarte command.
- Example evidence: 14's table matches its committed page sizes (soffice
  painted the size-less pandoc file at Letter, not A4). 00 measures the
  14 MB download it quotes (`download_jubarte.tsv`). 07 names its validate
  output `.jsonl`, and finds the validator inside the checkout, not at a
  path on one machine. 04 cites the commit on this branch, and 16 says its
  Word script logs were not kept.
- The adoption workflow also runs on changes to `tests/common/`,
  `Cargo.toml` and `Cargo.lock`.

### Known issues

- Microsoft Word 16.115 (installed on the bench 2026-10-05) crashes opening
  23 of this release's 600 benchmark redlines, so they score 0 in
  `release_info/`. 22 are byte-identical to the 0.11.2 redlines the previous
  Word opened on 2026-10-03; the cause is not found yet (KNOWN_ISSUES.md #8).

## [0.11.2] - 2026-10-03

> **Summary.** A changed paragraph is marked word by word or replaced whole by Word's own rule (the kept characters against 15 % of the longer side), measured in 1,178 Word comparisons. jubarte convert paints comments as Word's balloons (resolved ones faded, ranges bracketed) and follows Word's PDF more closely in headers, table rows inside content controls, footnotes, justified lines and embedded fonts. The CLI runs on Windows debug builds. Every release now ships its benchmark evidence in release_info/. On two 600-item samples scored against Word's own output: redlines mean 76.71 (Docxodus 69.24), PDFs mean 78.77 (LibreOffice 54.06), no failed item.
>
> **Docs.** README.md says how a PDF shows comments (Word's balloons in the markup pane) and that uvx/npx run the CLI from 0.11.0; the four registry READMEs are cut from it. docs/adoption/install-matrix.md lists the wheels and binaries every release since 0.11.0 carries, and adoption rows 0.11.0 shipped read released. docs/WORD_DIFFERENCES.md records comment balloons and sticky notes (10a). New release_info/README.md specifies the six evidence files; VERSIONING.md and AGENTS.md describe the evidence step. docs/python.md and docs/javascript.md are regenerated by the gates.

### Added

- `jubarte convert` paints Word's comment balloons in the markup pane. Word's
  Save as PDF draws each comment as "Commented [initials n]: " bold plus the
  comment's text, in a rounded box filled with the author's tint and stroked
  in the author's ink, a dotted connector from the end of the commented
  range, and the range itself tinted. jubarte left the pane empty and wrote
  a non-printed sticky note, which the raster scorer painted as an icon over
  the text. The painter (`paint_comment_balloons`) is built from the geometry
  of all 466 balloons of 152 corpus documents: chrome in page units (box
  16.67pt right of the pane, 3.4pt short of its right edge, inset 3.4,
  stroke 0.358, connector 0.179 dotted, stack gap 0.717, elbow 4.6 under the
  box top), text at the Balloon Text style's size (9pt unless the document
  sets it), first line 2.67 under the box top, pitch 1.236 × size. Word mode
  writes no comment annotation, as Word does; the other revision styles keep
  the sticky note, now bound where its range ends and anchored at the run
  end when a paragraph's close binds it. Two bugs fell out: a note on a run
  the line builder merged into its predecessor was dropped, and notes were
  bound at the range start instead of the end. Author colours cannot be
  copied (the same author gets up to six palette colours across the
  oracle's documents); the order-of-appearance palette stays. On the 537
  corpus documents with comments: mean 71.98 → 75.35, ≥90 from 25 to 58.
- A resolved comment (`w15:done` in commentsExtended.xml) paints faded, as
  Word paints it: text in Word's BFBFBF grey, the box stroked in the
  author's tint and filled 16 % of it toward white, the commented words under
  the pale fill. A `w:commentRangeStart` at body level opens the range for
  the paragraphs that follow. Balloon numbers count through the document
  whatever the author, and a reply takes its thread's number with "R" and its
  rank, the parent counting as 1.
- An empty comment range gets its balloon: a range end with its own start
  before it in the paragraph is live (an empty range followed by its
  reference is a reference alone), where the live-end rule read it dead. The
  151-document balloon survey is exact at 151/151, and a paragraph holding
  only the reference mark gets an empty strut run to carry the note.
- A commented range gets Word's brackets at its ends: a 0.179pt stroke down
  each end of the tinted text with 0.18pt serifs pointing in, the dotted
  connector leaving the closing one.

### Changed

- Compare, Word mode: a changed paragraph is marked word by word or replaced
  whole by Word's own rule. The kept span is the characters of the words the
  alignment keeps, with the blank inside each kept run and the one on either
  side of it, plus the paragraph mark on both sides; at 0.15 or more of the
  longer side's characters the paragraph is word-level, under it the words
  are replaced, and the rule runs again on each gap between the anchors it
  keeps (1178 single-paragraph Word probes of 6 to 4800 words: wave7 and
  edge1 1.000, paragraphs of 40 words or fewer 0.992, short1 0.988, asym1
  0.983, denom1 0.924). The engine used to void each common run shorter than
  2 % of the window, which replaced paragraphs whose kept runs are short:
  on 807 Word-labelled paragraphs its verdict agreed with Word's 41 % of the
  time, now 89 %. Only whole paragraphs are judged; a fragment the run
  resolvers cut out of a multi-paragraph region keeps their arrangement. A
  field's code counts as text, so a field whose code is kept stays paired
  at either end of a replaced window. `weighted_lcs_pairs` (Hirschberg,
  linear space) returns the alignment and `kept_span` sums it. On a
  2000-pair control scored against Word's redlines: +0.09 mean, interval
  above zero.

### Fixed

- Compare:
  - A wholly deleted hyperlink, or a deletion holding a complete HYPERLINK
    field, swaps behind the insertion that replaces it, as Word writes a
    replacement (insertion first). The writer's swap only saw a bare `w:del`
    followed by `w:ins` (corpus a098bd5ad9, the one real loss of the
    2000-pair control, −15.3). A wrapper holding part of a field stays put.
  - The resolver's replaced branch emits the deletion first. Ahead of
    content that is itself a tracked insertion on the revised side (a table
    pasted with Track Changes on, after a cell replaced across a hyperlink
    field) the insertion-first order made the region re-streamer delete and
    re-insert that content whole: 101 replaced paragraphs Word keeps in the
    corpus pair docx_lots_of_comments_addition. The written order is still
    Word's.
- Package:
  - Relationships and content types written with explicit closes are read: a
    `<Relationship …></Relationship>` (and a `Default`/`Override` in
    `[Content_Types].xml`) was skipped, and with it the part it named.
    Corpus 37c6c62345 lost its footer and scored 49. `PartFs::open` re-reads
    every `.rels` and the content types with each explicit close folded in.
    1 of 2925 corpus documents; Word reads both forms alike.
- Tables, grids and borders:
  - A row a content control wraps is painted, as Word paints it: a `w:tr`
    inside `w:sdt` (a repeating section's rows) was skipped on purpose to
    hold a page count the packing has since caught up with;
    `wrapped_children(dom, table, "tr")` gathers rows through content
    controls and custom XML alike. 38 of 2925 corpus documents carry row- or
    cell-level controls; the five in the 300-document sample averaged 64
    against 83.
  - A short row's cell wider than its grid columns spans the columns its
    width covers: `stretch_stale_spans` extends a lone wide `tcW` cell over
    the following columns (25f1d311bd's tcW 10080 over a four-column grid,
    no gridSpan, was given the first 84pt column, wrapping its paragraph
    into 15 lines and a page more than Word). 91 corpus documents carry such
    a row.
  - A row's `gridBefore`/`gridAfter` skip is as wide as the grid columns it
    leaves empty, not a cell's margins: ed36b607e8's 5-twip first column
    keeps Word's 0.25pt, where a margin-fitted placeholder pushed the row's
    text 10.5pt in and wrapped its lines a word short. A placeholder is no
    cell: it sets no minimum in the column fitters, whatever it spans, and no
    preference for its column.
  - A double cell or table border takes room for three strokes, two lines
    and the gap between them: `parse_border_edge` sizes a double `sz` at
    3 × sz/8 (Word probes: a single sz=6 top border puts the first row's
    text 0.72 under its no-border place, a double 2.16 to 2.25, a single
    sz=24 2.88), and row pitch and the split rule follow. The painter still
    fills that room as one band where Word strokes two lines. 4910ce2060's
    double-rimmed rows sat 1.5pt high.
- Lists and tabs:
  - A deeper list item ahead of its parent level uses up the parent's start:
    ed36b607e8's list (lvl 0 start 3) opens with 3.1 3.2 3.3 and Word
    numbers the first lvl-0 paragraph after them 4; the engine gave it 3 and
    every later label ran one behind.
  - A numbering label tabs to the first typed stop past it inside its
    hanging gutter: a custom left stop the paragraph sets, or, failing one,
    the hanging indent; a `num` stop is Word's own, re-derived from the
    level, and does not count (`TabStop.numbering`). f23fc5de2e's text
    starts 14.2pt in as Word's does, not 35.45; 36 corpus documents, 217
    paragraphs.
  - A right-aligned list label (`lvlJc=right`, ISO `end`) ends on the
    paragraph's first-line indent, left less hanging, running into the
    margin when the label is wider, and the suffix tab takes the text on
    from there. Six positions measured in Word's PDFs; the four tests that
    pinned the old tuck now assert Word's.
  - A right stop that cannot hold the text after the last tab wraps the
    paragraph: the TOC right-tab path steps aside when the suffix is wider
    than the room up to the stop, and the ordinary wrapper breaks the lines
    (7c02cf95f3's definitions, 55.8 → 83.6); long TOC entries whose head
    overruns the stop on one line keep it.
- Lines, spacing and justification:
  - Under `doNotUseHTMLParagraphAutoSpacing` Word ignores
    `beforeAutospacing`/`afterAutospacing` and lays the explicit values out
    (f23fc5de2e's rows pitch 23.8pt, not 41.8); the flag's other effect,
    summing after and before instead of taking the larger, stays. Five
    corpus documents.
  - The justified squeeze keeps a justified line's last word while the
    overflow is within a quarter of the line's total space width and within
    0.345 × (last word + one space). Four rounds of Word probes, 741
    single-paragraph documents, 23 boundaries at 0.2pt steps; the old
    (word + 2 spaces) / 3 cap violated all 23.
  - `w:wpJustification` (Word's "do full justification like WordPerfect 6.x"
    option, the compat-14 set Texas's statutes export): a justified line
    keeps its last word while its natural width is within 1.04 of the
    measure, a monospaced line compressed uniformly, a proportional one
    losing a quarter of each space first (42 Word probes). 041ec70002 runs
    Word's 35 pages (was 37) at 84.1 (was 51.8); seven corpus documents
    carry the flag.
- Fonts:
  - An `rPrDefault` that names no face is the OOXML default, Times New
    Roman, whatever theme part the package carries; a missing one keeps
    Word's built-in Aptos. A `<w:gridCol/>` without a width is a grid Word
    never wrote: the columns are content-fitted, and the autofit maximum
    leaves out a line's trailing blanks, which hang past the cell
    (0edc50c464, cbb3bab843, 6d510ca476).
  - An installed family wins over the document's embedded copy:
    `load_embedded_fonts` leaves out a font-table family the catalogue
    paints from an installed face (`catalogue_paints_family`). A machine
    without the family paints the embedded copy, and families the catalogue
    does not paint keep their embedded faces. 8c11ad13af 50.7 →
    92.1, its every Times line having stood 0.65pt short; 75 corpus
    documents embed fonts.
  - A Symbol or Wingdings character paints where those faces are absent.
    Word writes its list bullets as U+F0B7 in Symbol and U+F0A7 in
    Wingdings; a text face standing in for them (the WASM build, a host
    without Word's fonts) has a glyph for neither, and the bullet painted
    nothing. The stand-in paints the Unicode character behind the code:
    Adobe's Symbol encoding in the Symbol slot (• for F0B7, α for F061),
    and for Wingdings' bullets, boxes, arrows and ticks the nearest shape
    the face has (`Face::symbol_stand_in`). The PDF's text reads the same
    character back. A face that has the code keeps its own glyph, so a
    machine with Word's fonts converts as before. Found by the new binding
    parity check: 7 of 12 sampled redlines lost bullets in the WASM PDF.
  - The font report lists only the fonts the document asks for. The
    comment balloons' label face (Times New Roman bold) and the default face
    were resolved for every document, comments or not, so a document asking
    for Carlito alone reported Calibri too and `--fail-on-substitution`
    failed on a machine without Calibri. The balloon faces are resolved only
    when a balloon is painted.
- Command line:
  - The CLI runs on an 8 MiB stack on every platform, what Linux and macOS
    give a main thread. Windows gives 1 MiB, which the debug build's command
    dispatch overflowed before it read an argument: every CLI test of the
    Windows CI exited 0xC00000FD. The release binary did not overflow at
    1 MiB on the 1,200 jobs of the two release samples.
- Headers, footers and notes:
  - A page or column break in a header or footer paragraph, where no page
    can break, ends a line of its run's font (11 Word probes: a 9.5pt break
    alone stands an 11.0 line, a 20pt one 23.0), the body moving down by as
    much while the header is taller than the top margin and staying at the
    margin when it fits; `collect_visible_marked` takes a `BreakText` mode.
    4910ce2060's header title sat 11pt high and its page 2 sat 3.3 and
    11.2pt high (54.1 → 81.1). A break inside a header's table cell is
    still dropped: Word has not been probed there.
  - A flat inline connector (a stroked `wps:wsp` without a text box) in a
    header takes a line of its run and paints, its stroke centred half a
    point under that line's baseline and the shape starting its line by the
    paragraph's alignment; 12 corpus documents carry one. e0fe3a82eb:
    jaccard 0.25 → 0.73, text boundary 0.36 → 1.0, break drift 19 → 0.
  - The footnote area is the separator note's own paragraph (its run's
    font, size, line rule and before and after; the mark's size does not
    count), the rule is the separator run's OS/2 strikeout line
    (`Face::strike_pos_pt` / `strike_size_pt`), and a paragraph's last line
    over the notes must fit together with its space after (13 Letter probes
    plus six variants; `FootnoteCatalog::separator`, `after_over_notes`).
    bc85714262 +27.9, 636ef078e7 +19.2 on the sample.
- The markup pane:
  - The markup page takes Word's measured scale and top per page geometry:
    the scale is a whole 1/300 (the fit floor(300 × (W − 8.64) / (W − mr +
    265.68)) / 300 reproduces all 14 measured scales) and the top sits on
    the 1/300in grid where the exact fit (before the floor) centres it,
    read from `MARKUP_PANE_TABLE` (410 corpus PDFs with comments, 14 page
    geometries) and fitted for any other. Two refuted first cuts (a twips'
    A4 width, a gap applied below the page) were caught by the 300-document
    sample before release.
- Release tooling:
  - `scripts/release.sh` regenerates docs/javascript.md after rebuilding the
    npm package (v0.11.0 shipped a reference generated from the 0.10.1
    typings, missing appendDocuments, auditDocument, diffDocuments,
    markdownToDocx, validateDocument and more) and checks PyPI on the
    project listing, which a cached 404 cannot hide.
  - `scripts/release.sh` refuses a release without its benchmark evidence
    (step 3, `scripts/check_release_info.py`): the six `release_info/` files
    of the version, the two 600-item samples with every file's sha256
    (hashed against the bench's own files when a bench checkout is at
    hand), the two results scored on exactly those samples, and the
    website and app change lists. `release_info/README.md` is the spec.
  - Step 1 also moves `jubarte-wasm/Cargo.toml` and
    `jubarte-rust-inproc/Cargo.toml` (both sat on 0.10.0) and dies on a
    half-bumped tree, naming the file; the dry-run step wipes a previous
    release's sdist; PyPI gets a partial wheel set only with `--no-wait`.
  - The script tests run on Python 3.8 (a parenthesised `with`,
    `itertools.pairwise` and `str.removeprefix` kept the convert-sweep job
    red on the self-hosted runner), and `jubarte-wasm` no longer lists
    `js-sys`, which it never used.
  - Where main takes changes only through a pull request, the release
    commits go up as `release/vX.Y.Z`, and the pull request is merged at
    once with a merge commit (`scripts/push_main.sh`), so the tag still
    names the commit the gates ran on. A direct push is tried first.
  - Git stores `release_info/` byte for byte (`.gitattributes`, `-text`):
    the samples are CRLF csv and the results name them by sha256, so a
    checkout that rewrote line ends would fail step 3.
- Documentation and registries: the README says how a PDF shows comments
  and that `uvx`/`npx` run the CLI from 0.11.0; the install matrix lists
  the wheels and binaries every release since 0.11.0 carries (it still
  described 0.10.1); adoption rows 0.11.0 shipped read released. Keywords
  on crates.io, PyPI and both npm packages, and topics on GitHub.

## [0.11.0] - 2026-10-03

> **Summary.** Edit plans become a document editor (rewrite, insert_table, list, format_run, insert_footnote, insert_image, page_setup, fill_control, watermark, redact, settings, fields update, and occurrence targeting); new jubarte audit, validate (with --repair and --original), scrub, append, comments, diff-render and convert --pages; Markdown alongside Word; a jubarte-mcp MCP server and uvx/npx runners. Inputs are admitted under limits before anything inflates, and a legacy .doc or encrypted file is refused with LEGACY_DOC. The default conventional marks follow Litera Compare. Python wheels for Windows and musl Linux. docx-to-PDF follows Word 16 in more table, text box, spacing, list, font, header, footer, note and field cases. Breaking: WmlDocument::document_byte_array is private (use bytes()), AdmissionErrorKind gains LegacyDocument, and sha1_fingerprint* are renamed fnv1a_* (deprecated aliases stay one release).
>
> **Docs.** README rewritten around the task-to-command table, with per-library READMEs (README.crates.md, jubarte-python/README.md, jubarte-wasm/npm and cli READMEs) cut from fragments by scripts/library_readmes.py and locked by tests/readme_coverage.rs; the crate's rustdoc front page maps every task to its module, with no dead intra-doc links (release.sh now builds the docs with -D warnings); new docs/rust.md, docs/python.md, docs/javascript.md, docs/MARKDOWN.md and docs/adoption/ (MCP, install matrix, agent skills); docs/SELF_UPDATE.md covers signed releases; docs/WORD_DIFFERENCES.md, WORD_LAYOUT_RULES.md and WORD_COMPARE_RULES.md extended; GEMINI.md added; RESULTS.md removed in favour of the neurotic bench.

### Added

- `scripts/release.sh` step 13 runs `scripts/check_release_facts.py`: the
  release is not done until jubarte-app's `data/facts.jsonl`, which
  jubarte.pro and the Mac app print the engine's version, date, files and
  release list from, names the new version, lists a wheel for every required
  platform tag and is committed in the jubarte-app checkout. Step 12's
  `release_downstream.sh` now commits `data/` with the app's release files
  (the facts were left behind as "not a release file").
- `jubarte self-update` can require a zipsign (ed25519) signature on the
  release archive, checked against public keys built into the binary, so a
  replaced archive is refused even when its `SHA256SUMS.txt` was replaced
  too. Signing is staged: no key ships yet, and until the maintainers
  commit one the updater checks `SHA256SUMS.txt` only, as before. Once a
  key and a first signed release are set, that release and every later
  one must be signed; a `--version` downgrade to an earlier, unsigned
  release is checked by its SHA-256 alone. `release.yml` signs each
  archive before writing `SHA256SUMS.txt` when the `ZIPSIGN_PRIVATE_KEY`
  secret is set, and refuses to publish unsigned archives once a public
  key is committed under `keys/`. Setup and key rotation:
  [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md#signed-releases).
- The font report says whether each requested font was substituted: a
  `substituted` field in `convert --font-report` and `--report` (true for
  the `word_substitution`, `generic` and `unknown` steps, and for
  `open_fallback` unless the bundled face is the requested family itself),
  `FontReportEntry::substituted()` in Rust, and
  `FontResolution.substituted` with `RenderReport.substitutions` in
  Python. `jubarte convert --fail-on-substitution` lists each substitution
  on stderr and exits 4 after writing every output, for CI.
- `comparer::CompareMode { Word, PowerTools }` and
  `WmlComparerSettings::new(mode)`, with `with_author`, `with_date`,
  `with_detail_threshold` and `with_input_limits` builders, so callers can
  name a supported configuration instead of writing a struct literal. The
  fields stay public; nothing existing breaks.
- `admission::InputLimitOverrides`: key-by-key overrides of an
  `InputLimits` budget parsed from JSON, unknown keys refused.
- Python `compare_documents(..., input_limits=...)`,
  `get_revisions_json(..., input_limits=...)` and
  `CompareOptions(input_limits=...)`; WASM `compareDocuments(...,
  inputLimitsJson)` and `getRevisions(..., inputLimitsJson)`. Hosts can now
  lower the 512 MiB / 2 GiB compare budget, which a 32-bit WASM heap cannot
  hold.
- `WmlDocument::bytes()`, the bytes the document was opened from.

- `fuzz/`: cargo-fuzz targets for `admission::admit`,
  `strict_translation::strict_to_transitional_docx_within` and
  `compare_documents_with_settings`, seeded from the repository's `.docx`
  fixtures (`fuzz/seed.sh`), and a non-blocking CI job (`fuzz-smoke`) that
  runs each for 60 seconds. See `fuzz/README.md`.

- `jubarte-mcp`, an MCP server over stdio in the Python package
  (`pip install 'jubarte-redlines[mcp]'`): `docx_text`, `docx_inspect`,
  `docx_edit`, `docx_render`, `docx_compare`, `docx_changes`, `docx_accept`,
  `docx_reject`, `docx_capabilities`, and `docx_validate`, `docx_comments`,
  `docx_audit` (which report a missing engine feature until it lands). Every
  path must resolve under `--root`, outputs are files, and nothing is
  replaced without `overwrite`. Setup for Claude Code, Codex and Gemini CLI
  in [docs/adoption/mcp.md](docs/adoption/mcp.md).
- Gemini CLI extension: `gemini-extension.json` and `GEMINI.md` at the
  repository root; `scripts/bump-version.mjs` keeps the manifest version in
  step with `Cargo.toml`.
- Python: `Document.inspect_json()` returns the engine's inspect snapshot as
  JSON text.
- `jubarte diff-render A B` (`convert::diff_render`) lays out and rasterizes
  both documents at one resolution (`--dpi`, default 100) and compares the
  pages pixel for pixel: each page's `changed_ratio`, the box around the
  changed pixels, and `only_in` for a page one side lacks. `--out-dir` writes
  `a-page-NN.png`, `b-page-NN.png` and `diff-page-NN.png` (b's page with the
  change painted magenta and boxed; `--no-overlay` skips it) for the pages
  that differ, and `diff.json`; `--json` prints it. Exits 0 when no page
  differs and 5 when one does, so a CI step can gate on it. Python:
  `jubarte_redlines.diff_render(a, b, dpi=)` returns a `RenderDiff` of
  `PageDiff`s, and `python -m jubarte_redlines diff-render` matches the
  binary.
- `jubarte convert FILE --png --pages 1-3,7` rasterizes only those pages
  (counted from 1) after one layout pass of the whole document; the page
  report still covers every page. `RenderRequest.pages` (zero-based) in
  Rust, `Document.to_png(pages=)` and `Document.render(pages=)` in Python. A
  page past the end is `ConvertError::PageOutOfRange`. `RenderRequest` is no
  longer `Copy`.
- `capabilities` reports `operations.diff_render` and
  `operations.page_ranges`.
- Edit plans take `"existing_revisions": "keep"`: another party's tracked
  changes stay tracked, byte for byte, and the plan's edits become new
  revisions beside them by the plan's author and date (ids after the
  highest in the package), emitted directly instead of through compare.
  The clean copy has the plan's edits applied and theirs still tracked;
  the report's revision counts and the patch (`patch.diff`, Python
  `EditResult.diff`) cover the plan's own changes. Edits inside their
  revisions, and deleting, merging or reformatting a paragraph whose mark
  or properties they changed, are refused. New tables are inserted rows,
  list numbering and paragraph formatting are `w:pPrChange`, and comment
  replies, resolutions and deletions carry into the redline. New
  `markdown::patch_own_changes`, capability
  `operations.edit_keeps_revisions`, and Python `existing_revisions="keep"`.
- `watermark` edit operation (`{"kind":"watermark","text":"DRAFT"}`, with
  optional `color`, `diagonal` and `font`; Python `EditPlan.watermark`)
  writes Word's own VML text watermark, in its `Watermarks` content
  control, into every default header. A first section without a default
  header gets a new header part with its relationship, content-type
  override and `w:headerReference`. The comparison base carries the same
  watermark, so the redline holds it untracked. One per document: a second
  one, or a header that already holds one, is refused with
  `UNSUPPORTED_STRUCTURE`.
- `jubarte validate FILE` (`validate::validate`, Python `Document.validate`,
  WASM `validateDocument`): Word-validity findings beyond the schema, as
  data. Each finding carries a stable `code` (`TEXT_INSIDE_DELETION`,
  `MC_UNBOUND_PREFIX`, `DANGLING_RELATIONSHIP`, ...), the part and element
  path, whether Word refuses or repairs the file for it (`word_fatal`) and
  whether `--repair` fixes it. The Ring-1 invariants the test suite gated
  every produced package on (`tests/common/validity.rs`) now live in the
  library, with the five `jubarte debug` triage checks
  (`debug::findings`) behind them. Exit 0 clean, 2 findings, 1 unreadable;
  `--json` prints one object per finding.
- `jubarte validate FILE --repair OUT.docx` (`validate::repair`,
  `Document.repair`, `repairDocument`) fixes the findings with a
  deterministic fix (unbound `mc:Choice Requires` prefixes, `w:t` inside a
  deletion, `w:delText` outside one or under a move source, bookmarks in
  single-value content controls, dangling relationship attributes,
  duplicate drawing and revision ids, paragraph ids outside Word's range,
  table cells without a last paragraph, orphan comment anchors) and lists
  what remains. A package with nothing repairable comes back as its own
  bytes (#333).
- `jubarte validate EDITED --original ORIGINAL --author NAME`
  (`validate::audit_tracked`, `Document.audit_tracked`, `auditTracked`):
  every text change against the original must be a revision by that
  author; a paragraph that still differs after rejecting the author's
  changes is an `UNTRACKED_EDIT` finding at its id, another author's change
  a `FOREIGN_AUTHOR` one. It replaces `validate.py --original --author`.
- `capabilities` reports `operations.validate` and `operations.repair`.
- `jubarte scrub` (`scrub::scrub`, Python `Document.scrub`, WASM
  `scrubDocument`) removes who touched a document before it goes out: one
  author alias for every tracked change, comment and `people.xml` person
  (without presence information), no rsids, no creator, last editor,
  revision number, dates, manager, company or custom properties, and no
  comments. Each is optional (`ScrubOptions`); without a choice all four go
  under the alias "Author". Text, tracked changes and revision dates stay.
  Scrub refuses (`ScrubError::Invalid`) to write a package with a
  `validate` finding the input did not have. Capability `operations.scrub`.
- `redact` edit operation (`{"kind":"redact","paragraph":...,"find":"..."}`,
  optional `occurrence`; Python `EditPlan.redact`) replaces the text with
  one full block per character in the clean copy and the redline,
  untracked. The plan is
  refused with `REDACTION_LEAK` when the text still occurs in either output
  (`scrub::leaks`: text, attribute values, a paragraph's joined run text,
  raw bytes of binary parts); the refusal names the parts and never repeats
  the text, nor does the report. Works under `existing_revisions: "keep"`.
- `settings` edit operation (`{"kind":"settings","track_revisions":true,
  "update_fields":true,"protection":{"edit":"trackedChanges"}}`; Python
  `EditPlan.settings`) writes `w:trackRevisions`, `w:updateFields` and
  `w:documentProtection` into `word/settings.xml`, created with its
  relationship and content type when missing, in `CT_Settings` order
  (`settings::SETTINGS_ORDER`, held to `tests/data/wml_main_schema.json` by
  `tests/schema_consistency.rs`). The clean copy and the redline both carry
  them. `false` and `"edit": "none"` remove; a `password` is refused with
  `UNSUPPORTED` (Word's legacy hash is not written); one per plan.
- Python wheels for Windows x86_64 and musl Linux (Alpine) x86_64/arm64; the
  glibc floor of the Linux wheels drops from 2.34 to 2.28 (RHEL 8, Debian
  10, Ubuntu 20.04). `scripts/check_release_artifacts.py` refuses to publish
  a wheel set that misses an advertised platform, and `scripts/release.sh`
  runs it before `uv publish`.
- `docs/rust.md`, `docs/python.md` and `docs/javascript.md` carry the
  CLI references of the `jubarte`, Python and npm commands, taken from
  their real `--help`, and the Python and jubarte-wasm API references,
  taken from the shipped module and declaration files.
  `scripts/gen_docs.sh` regenerates them, release step 5 runs it and
  commits the pages, and the Docs CI job fails when they drift from the
  code.
- `jubarte audit FILE [--json] [--rules a11y,style,structure|CODE,...]
  [--strict]` (`audit::audit`) reports accessibility, style and structure
  findings, each with its rule set, severity and the paragraph id
  (`body:p:N`, `footer1:p:N`) an edit plan targets, or a part name:
  `HEADING_SKIP`, `IMAGE_NO_DESCR`, `TABLE_NO_HEADER_ROW`, `MISSING_LANG`,
  `LITERAL_BULLET`, `EMPTY_SPACER_PARAGRAPH`,
  `DIRECT_FORMATTING_OVERRIDES_STYLE`, `STALE_FIELD_CACHE` (an empty TOC,
  or a `NUMPAGES` cache that differs from the laid-out page count) and
  `FONT_SUBSTITUTED` (a font this machine draws with another face). It exits
  0 when nothing fails, 2 on an `error` finding, and on a `warning` too with
  `--strict`. The layout pass the last two rules need runs only when they
  are selected and have something to check; `--json` reports `layout: true`
  when it ran. Python `Document.audit(rules=None)` returns `AuditFinding`s,
  WASM `auditDocument(docx, rules)` the JSON report, and `jubarte
  capabilities` lists `operations.audit` and `audit_rules`. The slim WASM
  build has no layout pass (`audit::audit_report_with` with none), so it
  leaves `FONT_SUBSTITUTED` out and does not compare `NUMPAGES` caches with
  a page count; that keeps the renderer out of it (6.12 MB against 6.06 MB
  before; linking the layout pass made it 16.0 MB).
- `scripts/release.sh` step 12 runs `scripts/release_downstream.sh`: jubarte.pro
  moves to the release (download page, demo engine) and is deployed, the
  app's release files are committed on `release/vx.y.z` in the jubarte-app
  repository with a pull request, and the Mac App Store and benchmark
  commands are printed; `release_downstream.sh x.y.z --app` uploads the App
  Store build (Submit for Review stays a person's click).
- Markdown alongside Word ([docs/MARKDOWN.md](docs/MARKDOWN.md)):
  - `jubarte convert draft.md` (`markdown::markdown_to_docx`) writes
    CommonMark with GitHub's tables, strikethrough, task lists and footnotes
    as `.docx`, PDF or PNG, pandoc style (`-f`/`-t`, `--reference-doc`,
    `--resource-path`). CriticMarkup becomes Word tracked changes and
    comments; `--track-changes all|accept|reject` (pandoc's flag) keeps,
    accepts or rejects them, and `--no-critic` reads the delimiters as text.
    A change can cross paragraph breaks; a block that is one change whole is
    added or removed with its paragraph mark. On a `.docx`,
    `--track-changes accept|reject` renders the accepted or rejected
    document.
  - `jubarte diff OLD NEW` prints the changes between any two documents,
    Word or Markdown, as a patch in the style of git diff
    (`markdown::patch_documents`). Only changed paragraphs are shown, each
    whole, under `@@ [line:N] @@` for Markdown or `@@ [body:p:N] @@` (the id
    `jubarte text` and edit plans use) for Word; `@@ -[body:p:N] @@` marks
    a removed paragraph, located in the old version. Changes are `[-old-]{+new+}`, widened to whole words
    and numbers; highlights and comments stay CriticMarkup. The owner is
    named once, on the `+++` line (`--author`, else git's `user.name`, else
    Redline; `--date`, else now); a change by someone else is followed by
    `{>>Name (date)<<}`. Lines wrap at 72 columns (`--columns`, 0 for none).
    `--format critic` prints two Markdown documents as CriticMarkup, pandiff
    style (`markdown::diff_markdown`). With `-o` it writes a Word redline or
    PDF of any two documents (`markdown::redline`) and still prints the
    patch. The positional compare takes Markdown too. Without `-d`, a Word
    redline carries the current date, so two runs differ.
  - Word against Markdown: the Markdown's edits are applied to the Word
    document (`markdown::apply_markdown`) and only they show in the
    redline; empty paragraphs, fields, links, formatting and section breaks
    stay. On a fixture with three edits, the redline has 4 revisions where
    writing the Markdown to Word and comparing gave 434.
  - `markdown::resolve_critic` accepts or rejects CriticMarkup in Markdown.
  - `jubarte edit` writes the redline's patch as `patch.diff` beside the
    other outputs and prints it; `-q` prints nothing. `jubarte capabilities`
    lists `patch`.
  - `jubarte convert draft.md --page letter|a4` (`DocxOptions::page`,
    `markdown::PageSize`) writes Markdown on US Letter (the default) or A4,
    both with one-inch margins. A `--reference-doc`'s page setup wins, and
    asking for A4 with one adds a warning.
  - `jubarte text FILE --track-changes all|accept|reject` prints the
    document as Markdown with its tracked changes as CriticMarkup, or with
    every change accepted or rejected, as `convert -t md` does; the output
    then has no `[body:p:N]` ids.
  - Python: `jubarte_redlines.from_markdown(text, reference=, page=,
    author=, date=, critic=, track_changes=)` returns a `Document`
    (`_native.markdown_to_docx` returns the bytes; engine warnings are
    raised as `UserWarning`), and `python -m jubarte_redlines convert
    draft.md` writes `draft.docx`, or a PDF or PNG pages.
  - WASM: `markdownToDocx(text, optionsJson, reference)` in the full and
    slim builds.
  - Python: `jubarte_redlines.diff(old, new)` and `Document.diff(other)`
    take documents, bytes, Markdown text or paths and return a `Diff`
    (`str()`, `.hunks`, Markdown display in notebooks); `EditResult.diff` is
    an edit's patch. WASM: `diffDocuments(old, new, author, date)` and
    `EditOutput.patch`.
- Edit plans: `rewrite` gives a paragraph its new text and applies only the
  words that differ, keeping runs, formatting, tabs and fields; and
  `insert_paragraph` takes `like`, the paragraph whose properties the new
  one copies instead of the anchor's. Both are in `jubarte capabilities`,
  the Python plan builder and the agent skill.
- Edit plans: `insert_table` puts a table before or after a body
  paragraph (`rows`, optional `header_row`, `widths_dxa` and `style`).
  The table has a DXA `w:tblW` equal to the sum of the column widths, a
  `w:gridCol` and a DXA `w:tcW` for every column, and `w:tblHeader` on the
  first row when asked. Its cells take the anchor's paragraph style.
  `TableGrid` is added to `styles.xml` when absent. The redline shows the
  rows inserted. Ragged rows and mismatched widths are `INVALID_EDIT`, and
  nested tables are `UNSUPPORTED_STRUCTURE`. In `jubarte capabilities`, the
  Python builder (`EditPlan.insert_table`) and the agent skill.
- Edit plans: `list` makes body paragraphs a bulleted, decimal or
  lower-letter list (`paragraphs`, `kind_of_list`, `level` 0 to 8,
  `restart`). A new list adds one `w:abstractNum` (nine levels) and one
  `w:num` to `numbering.xml`, which is created when absent. Every
  `w:abstractNum` stays before every `w:num`, with ids after the
  document's. Each paragraph gets direct `w:numPr`, and `ListParagraph`
  when it has no style. The redline shows a `w:pPrChange` per paragraph.
  `restart: false` continues the nearest preceding list. Python:
  `EditPlan.list_paragraphs`.
- `inspect` reads tables as grids: the snapshot (`jubarte inspect --json`,
  `inspect::inspect_json`, WASM `inspectDocument`) gains `tables`. Each
  body table, nested ones included, has `index`, `rows` of cells
  (`paragraph_ids`, `text`), `header_rows` and `widths_dxa`. The ids feed
  straight into an edit plan. Also available as `inspect::tables` and
  `jubarte inspect FILE --tables`. Python: `Snapshot.tables` (`Table`,
  `TableCell`).
- Edit plans: `delete_paragraph` takes an optional `comment`, anchored on
  the deleted text in the redline (the clean copy has no paragraph to hold
  it) and shown on the removed paragraph's hunk in the patch. Python:
  `EditPlan.delete_paragraph(..., comment=)`. When the comparer deletes an
  identical neighbouring paragraph instead, the plan is refused
  (`UNSUPPORTED_STRUCTURE`) rather than leave the comment on text that
  stays.
- Edit plans: `format_run` changes the run formatting of one occurrence of
  existing text (`find`, optional 1-based `occurrence`); the redline records
  the old formatting as `w:rPrChange`. Run `format` (here and on `replace`,
  `insert`) adds `font`, `size_pt`, `color` (six hex digits or `auto`),
  `strike` and `caps` to `bold`/`italic`/`underline`/`highlight`. Python:
  `EditPlan.format_run(...)`.
- Edit plans: `insert_footnote` adds a footnote whose reference mark follows
  one occurrence of `after` in a body paragraph (optional 1-based
  `occurrence`). The note is appended after the highest footnote id; the
  footnotes part, with Word's separator notes, is created when the source
  has none. `FootnoteText` and `FootnoteReference` are used when the styles
  part defines them, direct superscript otherwise. Python:
  `EditPlan.insert_footnote(...)`.
- Edit plans: `insert_image` inserts a paragraph holding one inline picture
  before or after a body paragraph (`position`). The plan carries the file
  as `image_base64`; PNG, JPEG, GIF, BMP and TIFF are embedded, anything
  else is refused with `UNSUPPORTED_IMAGE`, and a `content_type` that does
  not match the bytes is `INVALID_EDIT`. `width_emu` sets the width and
  keeps the aspect ratio; by default the picture is its pixel size at 96 dpi,
  at most 6.5 inches wide. `alt` becomes the picture's description. Python:
  `EditPlan.insert_image(..., image=bytes)`.
- Edit plans: `page_setup` sets the page size (`letter`, `a4` or
  `{"width_dxa", "height_dxa"}`), `orientation` and `margins_dxa` (any of
  top, right, bottom, left, header, footer) of the last section or, with
  `"section": "all"`, of every section. Unchanged values stay; a document
  without a final section gets Word's default Letter page first. The
  redline records the old geometry as `w:sectPrChange` on every changed
  section: the comparer records the final one, and the edit adds the record
  to mid-document sections. Margins that leave no text width or height are
  refused. Python: `EditPlan.page_setup(...)`.
- `uvx jubarte-redlines redline a.docx b.docx -o redline.docx` runs the CLI
  without an install: the Python wheel installs a `jubarte-redlines`
  console script, `redline` is an alias of `compare`, and usage names the
  command the user typed. On main after 0.10.1; ships with the next
  release.
- `npx jubarte-redlines redline a.docx b.docx -o redline.docx` runs the
  same CLI: a new npm package, `jubarte-redlines` (jubarte-wasm/cli), puts
  a bin over the jubarte-wasm API with the commands, flags, messages and
  exit codes of `uvx jubarte-redlines`. On main after 0.10.1; ships with
  the next release.
- Comment threads (`comments::list_comments`): `jubarte comments FILE
  [--json] [--author NAME] [--latest]`, Python `Document.comments()` and
  WASM `listComments` list every comment with its thread (`parent`,
  `done`), the text it is anchored to and 80 characters on either side.
  Edit plans reply to (`reply_comment`), resolve or reopen
  (`resolve_comment`), reword (`edit_comment`) and delete
  (`delete_comment`, with its replies and anchors) existing comments, and
  `comment` takes `through` to cover several paragraphs. Whenever an edit
  writes comments, every comment paragraph gets a `w14:paraId` and
  `commentsExtended.xml`, `commentsIds.xml` and `commentsExtensible.xml`
  hold one row per comment. New refusals: `UNKNOWN_COMMENT`, and
  `COMMENT_NOT_IN_BODY` (previously `UNSUPPORTED_STRUCTURE`) for a comment
  anchored in a header, footer or note. The capabilities manifest lists the
  four operations and `operations.comment_threads`.
- `jubarte append a.docx b.docx [c.docx ...] -o out.docx`
  (`append::append_documents`, Python `Document.append`, WASM
  `appendDocuments`) puts each document after the previous one on a new
  page (`--section-break continuous|none` joins them on the same page;
  `--keep-sections` keeps each appended document's page setup, headers and
  footers as a section of its own). Images, links, headers, styles, lists,
  footnotes and endnotes come along under ids that do not collide; a style
  whose type and name the first document already has takes its definition
  there. Comments are removed and reported as `COMMENTS_DROPPED` by
  default; `--carry-comments` (`AppendOptions.comments = Carry`, Python
  `comments="carry"`, WASM `{"comments":"carry"}`) carries the comments the
  appended body anchors under fresh ids, whole (formatting, line breaks,
  links, the styles they use), with their reply threads, resolution and
  dates; comments anchored in notes, headers and footers are still dropped
  and reported. `capabilities` lists `append`.
- `inspect` lists the body's content controls under `controls` (id
  `body:sdt:N`, tag, alias, kind, text, paragraphs, lock, choices, checkbox
  state, placeholder flag); Rust `inspect::controls()` and Python
  `Snapshot.controls` / `ContentControl` read them.
- Edit plans fill content controls: `fill_control` selects a control by id,
  tag or alias and writes `text`, a list `choice`, a checkbox state
  (`checked`) or a `date` in the control's format, keeping its properties.
  Locked controls are refused with the new `LOCKED_CONTROL` code; choices
  outside the list, value forms the control cannot take and impossible
  dates are `INVALID_EDIT`. Python `EditPlan.fill_control`; capabilities
  report `operations.content_controls` and `fill_control`. The redline
  shows a fill as tracked text without the control (KNOWN_ISSUES.md #7).
- `occurrence` (1-based) on the `replace`, `insert`, `delete` and `comment`
  edit operations picks one hit of a repeated anchor; the Python builders
  take `occurrence=`. Without it, the `AMBIGUOUS_ANCHOR` refusal now says
  how many times the anchor occurs and the range to choose from.
  `occurrence: 0`, or `occurrence` with no anchor to pick from (`insert`
  at a `position`, `comment` without `find`), is an `INVALID_EDIT`. Rust
  code that builds these `OperationKind` variants with struct literals
  must add `occurrence: None`.
- Field results from jubarte's layout: `jubarte fields update FILE -o OUT
  [--json]` (`fields::update_fields`, Python `Document.update_fields()`,
  WASM `updateFields` in the full build) rebuilds each body `TOC` from the
  `Heading1`..`Heading9` paragraphs, with `_Toc` bookmarks, hyperlinks and
  dot-leader page numbers, and writes `PAGEREF`, `REF`, `NUMPAGES` and
  `SEQ` results in the body, headers, footers and notes. Field codes stay;
  a field whose switches it does not implement keeps its cached result. Edit
  plans gain `insert_toc` and `"update_fields": true` (Python
  `EditPlan.insert_toc`, `EditPlan(update_fields=True)`); `capabilities`
  reports `operations.fields`. Page numbers are jubarte's, not Word's
  ([docs/WORD_DIFFERENCES.md](docs/WORD_DIFFERENCES.md) section 11).

### Changed

- `--revisions conventional` (the default, `RevisionPalette::CONVENTIONAL`)
  now paints Litera Compare's marks: an insertion or deletion is marked once
  (blue underline, red strike) and a move twice, in green (double strike
  where it left, double underline where it landed). Insertions were
  double-underlined and moved-from text single-struck, so a landed move read
  as an insertion except for its colour. `word` and `custom` are unchanged.
- A Word 97-2003 `.doc`, or an encrypted document of any Word version (both
  are OLE compound files), is refused with the new `LEGACY_DOC` code and the
  hint to save it as `.docx` without a password, on every entry point and in
  every binding; RTF is `UNSUPPORTED_PACKAGE` ("an RTF file, not a .docx
  package"). Both used to surface as a ZIP error (`INVALID_PACKAGE`). The
  check is `admission::sniff`, which `admit` runs before any budget and the
  CLI runs as it reads a file. `admission::code_first` puts a refusal's code
  in front of whatever wrapped it (`I/O error: INPUT_LIMIT: …` reads
  `INPUT_LIMIT: …`, `document B: LEGACY_DOC: …` reads `LEGACY_DOC:
  document B: …`); the CLI's compare, revisions and convert failures,
  Python's `JubarteError` and WASM's thrown strings print refusals that
  way. `AdmissionErrorKind` gains `LegacyDocument` (a match on it needs the
  arm) and `ALL`.
- Crate docs: the "Lossless" tagline is now "Word-faithful", and a Fidelity
  section lists the main things the comparer normalizes (non-standard
  `w:sdtPr` children, `mc:AlternateContent`, and in the default mode the
  original's tracked changes, internal anchor hyperlinks, content controls in
  any paragraph that carries a revision, redundant default spacing, and the
  breaking versus non-breaking space distinction). It is not exhaustive.
- Doc comments that described a SHA-1 string check behind the LCS common-run
  match now say what the code does: the 128-bit FNV-1a fingerprint of the
  hash string alone decides equality there. The macro-generated docs in
  `namespaces` show the actual URI and local name, `w:cols` and
  `SECT_GEOMETRY` comments are corrected, and the repeated finalize passes in
  the comparer say why they run twice. `DEFAULT_DATE` and `MC::ns()` replace
  duplicated literals. No output change.

- The redline comparer admits both inputs before it inflates anything
  (`compare_documents*`, `edit` plans that compare, `WmlDocument::from_bytes`).
  A package past the budget is an `Err` whose message carries the stable
  `INPUT_LIMIT` code (or `UNSUPPORTED_PACKAGE`, `INVALID_PACKAGE`, ...) and
  names the side ("original document: ..."); the typed
  `admission::AdmissionError` is the `io::Error` source of the returned
  `OpcError::Io(InvalidData)`. The budget is the new
  `admission::InputLimits::compare()` (512 MiB per file and part, 2 GiB
  inflated, 10 000 entries, depth 256) and
  `WmlComparerSettings::input_limits` overrides it. Admission is one extra
  inflate pass over each input. The `inspect` and `edit` budget
  (`InputLimits::default()`) is unchanged.
- `strict_translation::strict_to_transitional_docx` no longer sizes an
  allocation from the ZIP central directory's declared size or copies the
  input first, and stops at the budget instead of inflating without limit;
  `strict_to_transitional_docx_within` takes the budget explicitly. Over the
  budget it returns the input unchanged, as it already did for an unreadable
  archive.
- CI: every third-party GitHub Action in `ci.yml` and `release.yml` is
  pinned to a commit SHA, with the release tag it resolves to as a trailing
  comment so Dependabot can bump both; Dependabot groups Cargo minor and
  patch bumps into one weekly pull request. Clippy with `-D warnings` now
  also runs on jubarte-rust-inproc, jubarte-python and jubarte-wasm (for
  `wasm32-unknown-unknown`), standalone packages the root workspace does
  not cover; jubarte-app/src-tauri keeps its own CI. The MSRV job runs the
  all-feature test suite on Rust 1.88 instead of `cargo check`, as
  README.md has said it does.
- Matching a style name against Word's built-in styles no longer allocates
  a lowercase copy of the name on each lookup; the answer is unchanged.
- `util::sha1::sha1_fingerprint` and `sha1_fingerprint128` are renamed
  `fnv1a_64` and `fnv1a_128`: they compute 64- and 128-bit FNV-1a of the
  SHA-1 hex string, not SHA-1, and their docs now say so. Output is
  unchanged. The 128-bit key alone decides equality in the LCS common-run
  match and the interior-run skip, which the docs state instead of claiming
  a string check follows (other paths, such as prefix and suffix trimming,
  still compare the hash strings).
  `ComparisonUnit::sha1_key` and `sha1_key128` return the same FNV-1a
  values and keep their names for now. `jubarte::util::fnv1a_128` is
  re-exported beside `fnv1a_64`.
- `ComparisonLog` and `CompareContext` derive `Debug`.
- Admission now checks end-tag names while it scans each XML part: a
  mismatched, stray or unclosed element is `InvalidXml` instead of passing
  the scan. All 819 `.docx`/`.docm`/`.dotx`
  files in the repository still pass.
- The library and the CLI `forbid(unsafe_code)` (examples keep the package
  `deny`), and `admission`, `strict_translation` and `opc` deny
  `clippy::indexing_slicing` and `clippy::arithmetic_side_effects` outside
  tests. The ZIP end-of-central-directory scan, the ZIP64 record read and
  the budget counters in those modules now use checked or saturating
  arithmetic and `get`; behaviour is unchanged.
- `accept_revisions`, `reject_revisions`, `get_revisions`, `list_changes` (and
  the accept/reject-changes functions) and the `convert` functions admit their
  input under `InputLimits::compare()` before they inflate it, as the compare
  path does; a package past the budget is an `Err` carrying `INPUT_LIMIT`
  instead of an allocation abort. Admission also reads the main part once
  more for nesting depth when its name is not `.xml` or `.rels`.
- Copied media parts are named with a lowercase extension
  (`word/media/P{sha256}.png`), since part names are case-insensitive, so the
  same image under `.png` and `.PNG` is one part.
- `word_tokens`: emoji skin tone modifiers stay in their sequence, and a mark
  that is also alphabetic (a Devanagari vowel sign) with nothing before it no
  longer starts a word. Flags, keycaps and tag sequences may still split.
- The crate and PyPI descriptions and the Python and npm READMEs say "Word-faithful" instead of "Lossless".
- **Breaking:** `WmlDocument::document_byte_array` is private; read it
  with `WmlDocument::bytes()`.
- `get_revisions` admits its input under `settings.input_limits` instead
  of always using `InputLimits::compare()` (the default settings give the
  same budget).
- Each published package carries its own README, generated at release
  from the repository README and a per-package fragment by
  `scripts/library_readmes.py`: crates.io (`README.crates.md`), PyPI,
  and both npm packages. Links are pinned to the release tag, because
  the registries cannot resolve relative links. The release checks that
  every artifact carries the README for its version (#339).
- `tiny-skia` 0.12: `png` 0.17 and `bitflags` 1 leave the engine's
  dependency tree (the Mac app still builds them through Tauri's `ico`).
  Rendered PDFs are byte-identical (#340).

### Deprecated

- `util::sha1::sha1_fingerprint`, `util::sha1::sha1_fingerprint128` and
  `util::sha1_fingerprint` stay as deprecated aliases of the `fnv1a_*`
  names for one release and are then removed.

### Fixed

- The crate's front page (docs.rs) described only comparing, accepting and
  rejecting. It now maps every task to where it starts: compare, changes,
  edit plans, inspect, PDF/PNG, Markdown, validate and repair, audit, scrub,
  append, comments, fields, admission and capabilities, with a compiled
  edit-plan example. `docs/rust.md`'s API tour gains the same modules.

- 41 intra-doc links rendered as bare brackets. A module documented both on
  its `pub mod` line and inside its own file resolved its links at the crate
  root, so `admission`, `append`, `audit`, `comments`, `edit`, `markdown`,
  `scrub`, `settings`, `validate` and `unid` lost theirs; the rest linked to
  private items. `scripts/release.sh` step 4 now builds the public docs with
  `RUSTDOCFLAGS="-D warnings"`, as docs.rs renders them.

- `scripts/release.sh` step 4 deleted `jubarte-python/uv.lock` after pytest,
  though the file is tracked: the release left a dirty tree and a resumed run
  stopped in preflight. The lockfile stays, and the release commit stages it.

- `capabilities().limits.stories` listed `body` only; `inspect` and `edit`
  address headers, footers, footnotes and endnotes too, and the manifest
  now says so (text boxes stay reported in `summary` but not editable).

- A footnote or endnote layout the renumbering step cannot resolve is an
  `Err` from `compare_documents*`, not a panic that aborts the Python
  interpreter or the WASM instance.
  `comparer::try_compare_bodies_faithful_with_notes` returns the
  `RectifyError`; `compare_bodies_faithful_with_notes` keeps its signature and
  panics as before. (`process_footnote_endnote` still has `expect` calls on
  the same path.)

- `limits.stories` in the capability manifest is now built from the list
  `inspect` and `edit` resolve selectors against (`inspect::STORY_KINDS`),
  checked by a round trip from the manifest through `inspect` to
  `apply_plan`, so the two cannot drift again. A story is named in a
  selector by its id, the part's file stem that `inspect` prints
  (`header1:p:0`, `{"story": "footnotes", "index": 0}`). The module doc
  no longer claims the operations are derived from the compiled features:
  the library compiles every operation in, and only a wrapper build (WASM
  without `pdf`) turns one off.
- `markup_simplifier::transform_element_to_single_character_runs` no longer
  panics (debug builds) or silently keeps only the first run (release builds)
  when its element is a `w:r`: an empty run, a run with only `w:rPr` or an
  empty `w:t`, and a run of two or more characters each explode into zero or
  several runs, not one root. A `w:r` is outside the function's contract and
  is returned unchanged with the DOM untouched. The signature is unchanged.
  Also, a one-character `w:t` holding a tab, CR or LF now carries
  `xml:space="preserve"` like a space does.
- A line ended by a `w:br` keeps its break when the paragraph reflows
  past a header or body float with square or tight wrapping. The lines
  below the float no longer run together, and a justified line that
  ends in a break stays unstretched under `doNotExpandShiftReturn`.
- Below compatibility mode 15, a page-anchored floating table that
  starts below the body top gets one body height of rows from its own
  top on its first page, past the bottom margin, as Word does; the
  paragraph after it flows in the room above it. Mode 15 still breaks at
  the page's floor.
- A paragraph that opens with a page break takes no line before the
  break, so a page filled to its last line breaks once instead of
  leaving a blank page. Priority d9b54326f3: 32 pages → Word's 30.
- An object taller than what is left of a page under nothing but the
  space before that opened the page stays on that page, past the bottom
  margin, as in Word, instead of leaving it blank. Priority 9f2c60b301's
  648pt cover box under Heading 1's 18pt before: 9 pages → Word's 8.
- CriticMarkup: a closing delimiter or `~>` behind a backslash is text, as
  an escaped opener already was, so `{++a \++} b++}` inserts `a \++} b`.
  Reading Markdown full of openers without closers no longer takes time
  quadratic in its length (120,000 unclosed openers took minutes; now
  milliseconds).
- Input admission (`admission::admit`) no longer panics on two integer
  overflows. A ZIP64 locator whose record offset is near `usize::MAX`
  overflowed the bounds check before the end record was read; the offset
  is now checked and the record read through `get`, so any offset that
  does not fit the buffer is `INVALID_PACKAGE`. Budgets of `u64::MAX` for
  `max_part_bytes` and `max_uncompressed_bytes` overflowed the per-part
  read cap (`cap + 1`); it saturates. Valid archives admit as before.
- Comparing the same pair twice in one process gives identical bytes. An
  image copied into the redline was named `word/media/P{n}.ext` from a
  process-wide counter, so its part name and the relationship Target
  pointing at it changed with every earlier compare (a server, a WASM
  instance or parallel tests). It is now named from the SHA-256 of its
  bytes (`word/media/P{sha256}.ext`), and identical images share one
  part.
- A redline keeps every embedded OLE picture (`w:object`, such as a
  Word.Picture.8 diagram) with its picture and OLE data. Before, the
  comparer re-emitted each one as an empty `<w:object …/>`: unchanged ones
  lost their picture, and inserted or deleted ones lost their `w:ins` or
  `w:del` too (5a6c's inserted diagram, which GroupDocs kept).
- A combining mark or zero width joiner stays in the word it follows, so
  decomposed text (`cafe` + U+0301) is one word like `café`, and an emoji
  sequence joined by U+200D is one token. A decomposed-accent word that
  changes is now one replaced word in `diff_markdown`, `jubarte diff` and
  the `rewrite` edit operation, not a word whose accent is left outside
  the change. A mark with nothing to attach to is a token of its own. Adds
  the `unicode-properties` dependency (already in the tree through
  `rustybuzz`).
- A footnote or endnote reference whose definition, or whose whole notes
  part, is missing no longer panics in `process_footnote_endnote`; the
  comparer returns a typed `Err` (`RectifyError::MissingNoteDef` or
  `MissingSourcePart`), and every lookup is made before a definition is
  rewritten. `process_footnote_endnote` now returns
  `Result<(), RectifyError>`, and `RectifyError` gains `MissingSourcePart`
  and `UnsupportedReferenceStatus`, which is a breaking change for callers
  that match it exhaustively.
- Numbering ids read from a document (`w:abstractNumId`, `w:numId`,
  `w:numPicBulletId`) no longer overflow when one is `i32::MAX`; the "next
  free id" arithmetic saturates (a panic in debug builds, a wrapped id in
  release builds before).
- docx-to-PDF follows Word 16 in the cases below. Each was measured
  against a probe document Word laid out (`word_pdf.py`), and a test
  holds Word's numbers. Page breaks and keeping:
  - A table row that does not fit moves to the next page whole only when
    one of its cells opens on a keep-with-next paragraph. keepLines, or
    keepNext on a cell's last paragraph, still lets it break between
    paragraphs. Before, 2ad8d15e88's reference list ended a page long.
  - A row cut at the page end keeps a line only while that line, the
    row's top rule and the table's bottom rule fit above the body
    bottom. The cut closes with the table's bottom border and ends under
    its last line. 2b479f55f8 now opens every page on Word's row.
  - A cell paragraph whose lines fit but whose space after does not is
    still cut before its last line at a row split, and an atLeast row
    height holds again for the part carried to the next page.
  - The part of a row carried to the next page opens under its own top
    rule, so the next row starts a rule lower.
  - A footnote cited from a table cell claims its row's page. Before,
    the note vanished (ece10bd712).
  - A paragraph that ends a border group fits its last line, the
    border's space and its width above the body floor, or opens the next
    page with them. Before, a split group drew its rule at the first
    page's foot.
  - A keep-with-next paragraph of inline pictures moves with its
    follower, measured by the pictures' height, not one text line.
  - A blank paragraph that carries a section break takes no line, but
    its space after still offsets the next page's opening space before,
    even after a table (12d245d664 page 48).
  - A page-anchored floating table whose offset lies above the text
    already on its page starts the next page at that offset.
  - Objects fixed to the page stay put when a full-width float at the
    body top pushes the text down.
  - `pgNumType w:start="0"` numbers the first page 0. We clamped it to 1
    and printed every number one high (3936a8fe56).
- Tables:
  - Tables with nothing between them are one table: the second keeps its
    row widths and starts at the first one's left edge, and the joined
    table aligns by its widest part. Tables of different layout types
    stay apart.
  - From compatibility mode 15, a left-aligned table puts its left rule's
    outer edge on the margin and its grid half that rule inside. A
    centred table centres its grid. Text sits its cell margin, or half
    its own rule when wider, past the grid line.
  - A compat-15 pct table keeps its rim at the pct width and its columns
    share the room inside half of each outer rule. A cell with no margins
    sets its text half its own side rule in on both sides and wraps
    there.
  - Without a `w:default` table style, an unstyled table keeps its border
    on the margin and an unnamed cell margin pads 0.5pt, not 108 twips.
  - `w:start` and `w:end` cell margins are the left and right margins;
    start beats a left set beside it.
  - A dxa-width autofit table gives every column its longest word, even
    past the right margin. A pct or auto table stays at the measure and
    breaks the words. An autofit grid wider than its room re-autofits
    between longest words and preferred widths.
  - In a fixed table, a pct cell in any row sets its column, and
    overflowing pct columns starve the rest down to their margins plus
    1pt. A row's largest top and bottom cell margins apply to every
    cell.
  - From compatibility mode 15, a space wider than its cell hangs past
    it; legacy layout stacks it. A cell narrower than a letter gives each
    punctuation mark its own line.
  - A justified cell line keeps its last word by narrowing its spaces,
    as a compat-15 body line does. A justified numbered cell line
    spreads from its indent.
  - A `w:hideMark` cell's empty last paragraph takes no height
    (e1cfa0591a: 14 pages to Word's 12). An empty cell paragraph whose
    mark is hidden takes no line.
  - Two or more text-anchored floating tables at one cell anchor sit
    side by side. A cell's exact line sets its baseline as a body line
    does; its text sat 4.7pt high.
  - A floating table in a cell followed only by empty paragraphs runs
    those lines beside it. A row reads only its own `w:trPr`, never a
    nested table's.
  - Cell text wraps beside a square float at the cell's left.
  - With `w:adjustLineHeightInTable`, cell lines snap to the section's
    document grid as body lines do.
  - A table style's own `w:ind` left and `w:jc` reach every cell
    paragraph whose style chain leaves them unset. Cell shading paints
    under the rules, which it used to hide.
  - Under `w:linkStyles`, a cell takes its table style's size unless the
    file's Normal sets its own.
  - Word mode (`--revisions word`) keeps a row-revised table's old fixed
    layout when its `w:tblPrChange` recorded one
    ([docs/WORD_DIFFERENCES.md](docs/WORD_DIFFERENCES.md) #7).
  - A centred table's 22-inch width cap ignores its `w:tblInd`.
  - Contextual spacing and the after/before fold stop at a nested table.
  - A table style's `w:b` or `w:i` with `w:val="0"` turns bold or italic
    off in its cells instead of on (#338).
- Text boxes, frames and floats:
  - A paragraph-relative text box that would run past the bottom margin
    is lifted to end there, as Word lifts it. The lines that meet it
    start the next page (49fe5bd42a: 3 pages to Word's 5).
  - A framePr with `yAlign` and no `y` floats aligned in its margin or
    page, out of the flow (3ec631ca50: 4 pages to Word's 3). A floating
    table wholly beside the text column narrows no line.
  - A page-wide square float in a compat-15 header drops the header line
    below it, and the body starts under that line.
  - A text box's painted outline insets its text by half its width. A
    fitted (`a:spAutoFit`) box takes the height its lines paint, and
    sizes a blank line by its paragraph mark.
  - An inline DrawingML text box in a header or footer takes its line
    and paints. A bordered paragraph's fill reaches its rules, also
    across the spacing inside a border group.
  - A VML shape wraps text only by its `w10:wrap`, and its wrap distance
    defaults to 9pt left and right. An inline unfilled `v:rect` (an old
    horizontal line) strokes and sizes its line.
  - A SmartArt diagram takes the line of a picture its size. A
    paragraph holding only a diagram lays no empty text line above it
    (25e6d4b508: 51 pages to Word's 50).
  - The run holding an inline shape sizes its line (3936a8fe56: 15 pages
    to Word's 16).
  - An underlined or inserted inline picture keeps its run's descent
    under the picture, DrawingML or VML (`w:pict`, or a `w:object` such as
    an embedded Word.Picture.8). A picture-only paragraph lays out its
    `w:br` lines. The control redline 5a6c9a5c went from 33 pages to
    Word's 35.
  - A header picture after text wraps when the text's last line leaves
    no room for it.
  - A table inside a text box lays out as a table between the box's
    paragraphs. Before, its cells ran together as one line of text, and
    5a6c9a5c's banner lost its grey band (#338).
  - An inline VML picture is sized in whole pixels at 143 dpi, as Word
    sizes it: points to HIMETRIC, HIMETRIC to pixels, and pixels to
    twips, each rounded half up. A 37pt image is 36.75pt. A DrawingML
    picture keeps its extent (#338).
  - A WMF picture's text records paint as PDF text in their selected
    font. Its rectangles fill with the brush and outline with the pen,
    and a null brush leaves them hollow. Before, 5a6c9a5c's diagram
    showed only its "+" and "→" (#338).
  - A row of inline pictures starts at the paragraph's first-line indent,
    unless the paragraph has a list label (#338).
- Lines, spacing and breaks:
  - `w:contextualSpacing` drops only the flagged paragraph's share of
    the gap. A flagged after of 6 over a plain before of 20 leaves 14,
    not 0.
  - With `w:doNotUseHTMLParagraphAutoSpacing`, paragraph spacing adds up
    instead of taking the larger value.
  - Auto spacing inherits attribute by attribute; a later plain
    `w:before` is the value an `w:beforeAutospacing="0"` brings back.
  - A line-break run sizes only a line it stands alone on. A paragraph
    of only tabs is a line of its paragraph mark. A paragraph mark's
    character style sizes its line.
  - A no-break space (U+00A0, U+2007, U+202F) neither breaks a line nor
    hangs past it.
  - `w:noBreakHyphen` paints a hyphen and never splits. A compat-15
    justified line squeezes an overflowing word only within Word's
    limits: a quarter of a space, and a third of the word plus two
    spaces.
  - `compressPunctuation` narrows spaces only where Word does: before
    mode 15, only while the default font names no face, and only Times
    New Roman and Arial spaces.
- Lists and tabs:
  - From compatibility mode 15, a right tab past the right margin lands
    on the margin; legacy layout keeps the stop. In a header, the text
    after such a tab stays on its line in every mode; we wrapped it.
  - Tab leaders fill whole cells, one mark wide, of a grid counted from
    the page's left edge, up to the last cell that ends by the stop.
  - A typed label such as `<tab>(a)<tab>` right-aligns on its own stop in
    the hanging gutter and its text wraps on the next stop (c73c128db4).
  - A justified line whose text after its last tab holds spaces keeps
    that tab's stop and leader.
  - A list label takes its paragraph mark's character style and its
    revision: an inserted mark underlines the label through its tab, a
    deleted one strikes it. A tab's gap carries its run's underline and
    strike.
  - A negative numbering-level indent beats the paragraph style's.
- Fonts and scripts:
  - Absent fonts take Word 16.114's own substitutes: Myriad Pro is Segoe
    UI, Proxima Nova Tahoma, Futura PT Century Gothic, Adobe Garamond Pro
    Garamond. An unknown font-table entry is Cambria when roman and
    Calibri otherwise. A Bold-only family paints its regular runs in that
    Bold (1fef0faeb8: 18 pages to Word's 20).
  - Japanese or Chinese text the East Asian face lacks paints in MS
    Gothic (when that face is Times New Roman) or MS Mincho, not
    Microsoft YaHei. Without Word's East Asian faces (Linux and WASM
    builds), a wide character a face lacks measures one em, not the
    0.75em `.notdef`.
  - Windows reads its own font folders, `%WINDIR%\Fonts` and the per-user
    `%LOCALAPPDATA%\Microsoft\Windows\Fonts`, and matches their
    abbreviated files (`timesbd.ttf`, `cour.ttf`) by the family name
    inside. Before, it searched only macOS paths and painted Calibri,
    Times New Roman and MS Gothic with open-source substitutes. The on-disk
    font index moves to rules 2, so earlier answers are searched again.
  - An Office 365 cloud East Asian face (name-table version ending
    `;O365`) takes 1.3 times its head box for the line, not its hhea
    body.
  - `w:balanceSingleByteDoubleByteWidth` widens spaces to the East Asian
    face's average width, only under Word's East Asian layout. A closing
    bracket before another bracket, or an opening one before an opening
    one, advances half an em. Lone full-width punctuation keeps its full
    width.
  - ASCII between ideographs keeps the ascii face. The gap after Latin
    text before East Asian text is half the Latin face's average width.
    Hangul takes the autoSpace quarter em, and `w:autoSpaceDE`/`DN` off
    drop it.
  - `w:spaceForUL` adds no space under underlined East Asian text.
  - A character style toggles its paragraph style's bold, italic, strike
    and caps: both set means plain.
  - An `hAnsi` face beside an `asciiTheme` slot paints only characters
    past U+007F.
  - Calibri's Thai paints in Leelawadee UI, its Devanagari in Mangal, its
    compatibility jamo in Malgun Gothic. List and page-number formats
    that write a script load its fallback face. Scripts only in a header,
    footer, note or comment load their fallbacks too.
  - Text written as numeric character references loads the faces its
    characters need.
  - A precomposed Latin letter (ě, č, ů) paints as its own glyph.
  - Symbol-font text extracts as Word writes it: list bullets read back
    as •.
- Headers, footers, notes and fields:
  - Endnote marks number in reference order, in the format the section's
    `w:endnotePr` gives (lowerRoman by default). The endnote separator
    stands on its own line over the notes.
  - A header field is named by its first word: an INCLUDEPICTURE whose
    path holds "page" no longer paints as a PAGE number.
  - A `w:sectPrChange` record's header references are not the section's
    own.
  - A paragraph style's `w:vanish` hides its runs; a hidden row folds
    every run's properties.
  - A leading empty revised header or footer paragraph gets its change
    bar, and an all-empty footer paints its paragraph borders.
  - A header or footer STYLEREF shows the first body text in the named
    paragraph or character style on its page (with `\l`, the last). With
    no match on the page it shows the last before the page, and before
    any match the first after. The field's line is laid out again around
    the new text, so a right-aligned or centred header stays on its edge.
    Before, every page printed the cached result; 5a6c9a5c's "s. 1" is
    now s. 4, s. 9 and s. 12, as in Word (#338).
  - Word mode (`--revisions word`) paints an inserted field that Word
    recomputes (PAGE, NUMPAGES, SECTION, STYLEREF, REF, DATE, SEQ)
    unmarked, as Word does. Cached fields such as DOCPROPERTY stay
    marked, and the default revision styles mark both
    ([docs/WORD_DIFFERENCES.md](docs/WORD_DIFFERENCES.md) #5, #338).

## [0.10.1] - 2026-09-30

> **Summary.** Tracked changes can be accepted or rejected one at a time, as Word's Accept / Reject This Change, from the CLI, edit plans (resolve_revisions), Python and WASM; Reject All matches Word on text and mark state; docx-to-PDF follows Word far more closely on page breaks, keep-with-next, table rows, floating tables, headers, footers and comment balloons; jubarte debug gains --check render, diff A B [C …] and -c text.
>
> **Docs.** New docs/WORD_COMMENT_BALLOONS.md (when Word draws no comment balloon) and docs/api/ snapshots of the Rust and WASM API surface for release drift review; docs/WORD_DIFFERENCES.md extended; VERSIONING.md documents the step-5 API-docs review; the jubarte-documents skill and the npm README cover per-change accept/reject.

### Changed

- The tests no longer need neurotic_docx_bench (or the local
  `_to_improve_accepted_changes` folder) checked out beside this repository.
  The 537 documents they read are copied into `tests/corpus/` under the
  relative paths the tests name. Their provenance, including the
  docx-corpus ODC-By attribution, is in
  `LICENSES/LicenseRef-Bench-Fixtures.txt`. About 180 fixtures had
  moved in the bench, so the tests naming them skipped without a word;
  they run again, and `tests/bench_fixture_paths_resolve.rs` fails on any
  named copy that is missing.

### Added

- `jubarte debug FILE --check render` lists what each story part should put
  on the page, for diagnosing a PDF that scores low against Word's. It reads
  every part: body, headers, footers, footnotes, endnotes and comments, with
  their text boxes. For each part it lists:
  - the sections;
  - tables, with their size, style, float position, direct and cell shading,
    the table style's shading, and the first row's text;
  - frames and anchored drawings;
  - fields such as PAGE;
  - paragraph styles and shading;
  - text colour, highlight, run shading and hidden text, direct or from a
    style;
  - requested fonts;
  - whether a replaced run's deletion comes before its insertion or after
    it.

  A `(layout)` entry adds jubarte's page count and the face each requested
  font resolved to (installed, embedded, altName or a substitute), plus the
  `docDefaults`, theme and `fontTable.xml` fonts. With two files it prints
  only the lines that differ.
- `jubarte debug diff A B [C …]` compares two or more packages element by
  element instead of line by line: styles pair by type and name, paragraphs
  by their text (a rewritten one meets the one it replaced), tables by
  their first text, cells by position, notes and comments by text, headers
  and footers by the section role that shows them. Each hunk names the
  element's path (`word/styles.xml › style paragraph "Body Text" › rPr`)
  and prints the property items or run segments not every file holds; with
  three or more files each line names the files holding it. rsids,
  paragraph ids, revision ids/authors/dates, relationship ids (a link or
  image shows what it points to instead), docProps save stamps and counts,
  attribute order, on/off values and empty property blocks are dropped
  unless `--raw`; a part that is not well-formed XML is reported, not
  compared. `--style`, `--para-text` and `-p` narrow the report, `--full`
  prints the shown elements' common lines too without counting them. The
  same in the library (`jubarte::debug::diff::diff`).
- Tracked changes one at a time, as Word's Accept / Reject This Change:
  `jubarte changes FILE` lists every change with an id (`body:rev:12`,
  `header1:rev:3`), its kind, target, author and text (`--json` for JSON
  lines); `jubarte accept` / `reject` take `--id`, `--author` and `--kind`
  (repeatable; a change must match every flag given) and keep every other
  change tracked. Ids stay valid while changes remain, both sides of a move
  resolve together, and a change's `inside` names the change whose
  resolution takes it along. The same in the library
  (`jubarte::changes::{list_changes, accept_changes, reject_changes}`),
  Python (`Document.changes()`, `Document.accept(ids=, authors=, kinds=)`)
  and WASM (`listChanges`, `acceptChanges`, `rejectChanges`).
- Edit plans take `resolve_revisions: {"accept": FILTER, "reject": FILTER}`
  to resolve a selection of the tracked changes before editing; what it
  leaves follows `existing_revisions`. A change selected by both sides is
  refused with `REVISION_CONFLICT`, an id the document lacks with
  `UNKNOWN_CHANGE`; the report lists the resolved ids.
- `jubarte debug FILE -c text` prints each story part's paragraphs with
  `{+inserted+}` / `[-deleted-]` runs, the paragraph mark's revision state
  and table rows; `-c xml` prints part XML one element per line without
  namespace declarations, rsids or paragraph ids. With two files both print
  only the lines that differ, part by part. `-c runs` is `text` with each
  paragraph's direct properties and each run's direct formatting, so two
  builds compare formatting without XML noise.

### Fixed

- Consecutive paragraphs in a text box are spaced by the larger of one's
  space after and the next one's space before, as in a table cell. The
  first paragraph's auto space before and the last one's auto space after
  are dropped. Before, the two were added together.
- A tab whose next default stop lies past the right edge, with no custom
  stop left on the line, moves to the next line and is measured from that
  line's start, as Word does in compatibility modes 14 and 15.
- A table row whose cells are centred or bottom-aligned breaks at the page
  end like any other row. Before, it stayed whole and moved to the next
  page. The docxide suite's education_consultant_posting now starts its
  Background row on page 1, as Word does.
- A Korean font the machine lacks (charset 81, or Hangul in its name) is
  painted in Batang when the font table calls it roman, and in Malgun
  Gothic otherwise, as Word 16 paints it. Before, it fell back to Microsoft
  YaHei, which has no Hangul, so the text was lost.
- Hangul in a run whose fonts are named only for Latin text is painted in
  the run's East Asian face, as ideographs and kana already were. With
  an empty theme `a:ea` and a ko-KR East Asian language, that face is
  the theme's Hang script font. Before, the Latin face drew nothing for
  the syllables: the docxide suite's Korean conference form lost "발표"
  from its title.
- Trailing tabs count toward a centred or right-aligned line; only
  trailing spaces hang past it. Before, the tabs were measured as glyphs
  and taken off, so cb4f8b4a43's centred "Date:" and its 14 tabs sat
  61pt in from the margin, not 3.6pt.
- A table row moves to the next page whole when one of its cells can put
  nothing on the page, as Word 16 does whatever the row's vertical
  alignment. Before, the other cells' first lines stayed behind.
- A justified line holding only tabs keeps their stops and dot leaders.
  Before, the justified painter drew the tabs as text, so the docxide
  suite's air_pollution_permit_form lost two of its four dotted lines.
- A missing script-family font (a `w:family="script"` font-table entry
  such as Vivaldi) paints in Calibri, as Word 16 does. Before, it fell to
  Cambria, whose wider 100pt bold "Messiah" wrapped and gave
  handels_messiah a ninth page.
- `w:contextualSpacing` drops the space between same-style paragraphs
  inside a table cell, as it already did in the body. Before, cb4f8b4a43's
  Title lines stepped 28pt apart, not Word's 20pt.
- Below compatibility mode 15, a table style's font size overrides the
  Normal style's in unstyled cells only when Normal is 11 or 12pt, as in
  Word 16. Before, it overrode any size, so the cf02 redline's 10.5pt
  Normal cells painted at 11pt and ran to 12 pages, not Word's 11.
- A table row splitting at the page end measures a cell's first cut
  against the room its margins leave. Before, the empty head was sized as
  a default 11pt paragraph, so 3138fff3a6's two-line 8pt titles placed no
  line and their rows moved to the next page whole, where Word splits
  them.
- From compatibility mode 15, a cell paragraph cut by a table row split
  keeps widow/orphan control: two lines on each side, or it moves to the
  next page whole, as Word 16 does. Below mode 15, Word still cuts it
  anywhere. Before, compat-15 two-line cells split one line each side:
  30f195a272's first table row started on page 1, where Word's starts on
  page 2.
- A table too wide to sit beside a square-wrapped picture starts below
  it, as in Word 16; a narrower table still sits beside it. Before, the
  table ran over the picture: 109f20a2b3's continued table painted
  across its header logo, 74pt higher than Word's.
- A "keep with next" paragraph that ends on a page's last line, its
  successor pushed over, keeps its tail with that successor as Word does.
  From compatibility mode 15 it splits and carries its last two lines
  over (one with widow control off). Older modes move it whole, together
  with the keep-with-next paragraphs chained before it. A chain of short
  keep-with-next paragraphs now stays with the body below it, and its
  spacing no longer counts the space between two paragraphs twice. A
  chain of unsplittable ones taller than a page moves to a fresh page
  once and then fills pages. A keepLines paragraph comes whole with the
  heading above it.
  Priority 4e7bb2a1be's event chain moves to page 2 as in Word (2 pages
  → Word's 3).
- Paragraph borders in headers and footers paint wherever Word draws
  them: on an empty paragraph above the part's text (a footer's opening
  rule, a running head's rule under its title table) and on a text
  paragraph below one. Before, jubarte reserved their space but drew no
  rule: c73c128db4's legislation pages lost both the head and foot rules.
- A `cantSplit` table row taller than a whole page breaks anyway, as in
  Word 16: it moves off the page it started on and breaks at the foot of
  the next one. Before, it stayed whole and ran off the page: 2b479f55f8's
  5e row painted 116pt below its page, and the file came to 31 pages, not
  Word's 33.
- A section that omits a header or footer type takes that type from the
  nearest earlier section that names it (ECMA-376 17.10.5), as Word does.
  Before, a section naming only some types lost the rest: 2b479f55f8's
  sections 2 and 3 dropped the "ICA Internship Award 2022/23" header from
  page 4 on, and its pages ran two short of Word's.
- A continuous section break that changes the page size or orientation
  starts a new page with the new size, as Word does. Before, the page
  kept its old size: b535008087's landscape section ran on portrait pages
  and came to 18 pages, not Word's 21.
- A nested table row taller than what is left of the page breaks inside
  itself, as Word breaks it, when keeping it whole would leave a quarter
  of the page empty. Rows marked `cantSplit` or with an exact height stay
  whole. The `w:br` line breaks of a cell paragraph split across pages
  stay where they were. Before, the story parts ran together. In the
  docx-to-pdf work set, three newsletters of one family gained 4 to 16
  pixel points each (net +30.2), with nothing worse. 931e978ee2 is back at
  Word's three pages.
- A float that leaves no room on its line in a multi-column section moves
  the text to the next column, not to the next page.
- A fixed-layout table whose cell widths run past 22 inches ends 22
  inches right of the margin, as Word ends it. The column that crosses
  that line keeps what is left, and each later one about 1.4pt, so its
  text stacks a character, or a space, per line. 2b479f55f8's landscape
  process table now makes that document 32 pages (Word 33, was 24).
- A square-wrapped float whose top sits below its anchor's first line no
  longer pushes that line to the next page.
- A header picture with no height adds no line under the header text.
- An empty header or footer paragraph paints its shading across its
  line, above or below the text or in a part with no text, as Word does.
- The review fixes for PR #247:
  - A run's `w:rPr` blocks apply `w:vanish` in order, so a later
    character style that turns it off shows the run again.
  - In a header or footer, a full-width picture that follows the
    paragraph's text wraps to the line under that text, and the text
    keeps its first line.
  - The empty paragraphs above a footer table count each `w:br` as a line.
  - An auto-width frame is as wide as its widest line, measured between
    `w:br` breaks, not the text on both sides of a break added up.
- `jubarte convert --revisions word` draws a comment balloon only where
  Word's Save as PDF does. Word draws none for a comment whose range ends at
  body or cell level, or before any content in its paragraph, and none for a
  reply to such a comment. With no balloon left, Word keeps the full page,
  with no grey markup pane. 29 of the 222 documents in the docx-to-pdf worst
  set were shrunk beside a pane Word never draws. The other revision styles
  still draw every comment.
- A comment whose range holds only a tab keeps its balloon.
- Comments load when `commentsExtended.xml` is related before
  `comments.xml`. The lookup took the extended part for the comments part,
  and every comment was lost (with_comments_clean/37c6c62345).
- A frame at a page y that runs from the margin or the column
  (`w:framePr vAnchor="page" hAnchor="margin"`) floats at its position, as
  Word paints it, instead of flowing as a plain paragraph. Without an x it
  sits on the margin; without a width or height it takes its text's size, up
  to the column's width. A frame too tall for the page moves up to the
  page's top. The worst document in the docx-to-pdf set (61e3967518 ×
  1424386e9b) went from 25.6 to 42.1 pixel, 8.4 to 20.2 Jaccard.
- A header paragraph's shading (`w:shd`) paints behind its lines, out to
  its border. Only the top and bottom rules were painted, so white text on
  a shaded heading vanished (clean/2261da4dae: 32.9 to 43.3 pixel, 7.2 to
  23.3 Jaccard).
- A left `w:ptab` in a header or footer moves nothing before any text and
  starts a new line after it, as Word prints it. "Metadata", then a right
  ptab and "Page N of M", then a left ptab and "Downloaded" printed on one
  line and ran off the page. Five documents in the docx-to-pdf set gained
  4 to 17 points (pixel plus Jaccard), e.g. clean/ef7870f9f4 from 45.9/10.7
  to 50.1/24.1.
- In compatibility mode 15, a paragraph holding only a page break is one
  line on the page it ends. When that line doesn't fit, Word moves it down
  a page and the break leaves that page blank (a9de4ed3f9's cover, then an
  empty page 2, then the register on page 3). Together with the ptab fix,
  a9de4ed3f9 went from 34.2/9.4 to 42.9/18.0, and clean/3936a8fe56 from
  39.7/10.9 to 45.8/19.6.
- A character style's `w:vanish` hides its runs; a direct `w:vanish
  w:val="0"` shows them again. 4910ce2060's content-control placeholders
  printed in cells Word leaves empty. `jubarte debug --check render` lists
  them as `vanish via "Style"`.
- A run whose `w:t` or `w:delText` is a single space prints that space.
  Redlines that delete a lone space between words ("under Section",
  "for the period") printed the words fused. In the docx-to-pdf work set,
  a189917's redline went from 45.5/20.9 to 59.3/51.3 and back to Word's
  page count, and fc5d9fe's from 48.1/17.1 to 62.9/37.7.
  `jubarte debug --check xml` now shows such a space as the element's text.
- A header or footer paragraph whose frame holds a right or centre
  `w:ptab` relative to the margin spans the margins, as Word prints it.
  54f4bb7's centred footer frame printed "Metadata … Page 2 of 28" as a
  narrow block in the page centre (42.7/11.1 to 54.6/21.3).
- The `w:br` ending a table-cell line no longer sizes a line that holds
  text. 246c7fcf50's white 24pt break after a 16pt title made the title
  line 24pt tall and pushed the header down (43.9/24.1 to 49.7/28.7).
- Empty paragraphs above a footer's first table no longer lift it. The
  footer grows up from the page bottom, so those lines sit above the table;
  246c7fcf50's "Year 6 …" footer table printed 27.6pt high (its redline
  went from 38.9/12.1 to 45.7/20.7, the clean document from 49.7/28.7 to
  53.8/35.5).
- An empty paragraph whose mark's character style vanishes takes no line.
  4910ce2060's ContentControlHidden marks under "System Name" each added a
  line and pushed the document onto a third page; it prints on Word's two
  (31.4/12.2 to 54.1/18.2).
- A header or footer picture as wide as its line puts the paragraph's text
  on the line under it, and that picture line keeps the text's descent.
  A tab after it shares the next line with the text that follows it, and
  deleted text counts as text there because the redline paints it.
  0800162a66's banner and "Akreditācijas ekspertu" took five pages
  against Word's four (34.4/10.2 to 64.7/48.2).
- A header picture too wide to sit beside a square-wrapped float in its
  paragraph starts under the float, below its distB and effect extent.
- Trailing spaces hang past centred and right-aligned header and footer
  lines, as they do in the body.
- Footer pictures placed from their paragraph sit on that paragraph's top,
  not on the footer's (432bf1c280's logos under "January 2025").
- A header or footer line holding a tab wraps at its words when its text
  runs past the margin. A tab in a centred or right-aligned line jumps from
  the line's start before the line is aligned, as in the body. 3ee2d0c's
  centred banner, tab and deleted "Zgłoszenie … 2024 rok”" printed as one
  line running 56pt past the margin (53.3/39.0 to 73.1/62.2).
- A floating table with no `vertAnchor` is placed from the top margin, as
  Word reads it, not from the text cursor. The text before it on its page
  moves under it when there is no room beside it. A table too tall for
  the page holds one body height of rows on its first page. It starts the
  next page when text is already on this one. e11fc429b2's two
  page-anchored tables went from 49.7/42.3 to 87.3/83.3 (Word probes g1-g6,
  fb, fe).
- A wrapping picture (square, tight or top-and-bottom) placed off its column
  or paragraph stays on the page, as in Word: it is pulled in from the left,
  top and foot edges instead of running off them. A wrapNone picture may
  still leave the page. When such a picture leaves its paragraph's first
  line no room beside or under it, the paragraph and the picture start the
  next page. 3bfcb371e2's 615x797 cover used to paint over the minutes
  before it; now it opens its own page at 0,0. 30b6e87178 went from
  33.4/20.9 to 62.1/54.7, 49fe5bd42a from 47.1/9.1 to 58.6/19.3 (Word
  probes h1-h6, g3, g5, n3-n5, t3, t4).
- A picture anchored after all of a paragraph's text that fills its page
  (no room beside or under it) takes that page alone, as in Word. Each
  page that could still hold the rest of the paragraph keeps only its
  first line. The last line opens the picture's page with its top at the
  body floor, and what follows starts the next page. 9b22b88370 went from
  26.4/23.1 to 64.5/49.4, 842ef93738 from 25.2/37.1 to 70.8/79.5 (Word
  probes k1-k14).
- A header or footer line without tabs paints its strikes and underlines:
  a deleted run's strike, an inserted run's underline, `w:u` and
  `w:strike`. Only its glyphs used to reach the page. 9b22b88370's
  deleted header title is struck as in Word; clean 00163e55a0 went from
  49.3 to 72.7 pixel, and the work set from 48.22 to 48.34.
- Change bars follow Word further. A revised paragraph that crosses a page
  or column is barred on each one it fills, not only where it starts; a
  paragraph whose mark alone is inserted, deleted or reformatted is barred;
  and so are a header's or footer's revised lines and empty paragraphs.
  450945a41c went from 11.6 to 36.6 jaccard; the work set from
  48.34/25.98 to 48.41/26.23 (93 better, 1 worse), with the tip holdout
  at 46.08 to 46.15.
- Rejecting a redline gives back what a changed style inherited in the
  original. Word's Reject All reads a style's old record against its
  built-in defaults, so a record holding only the style's own properties
  rejected to Times New Roman 10 pt, left-aligned and single-spaced
  (b6f757462e's List Paragraph lost Normal's justification and 1.08 lines).
  The records of styles based on another now carry what the original's
  docDefaults and chain gave them, less the built-in values. Paragraph-style
  mismatches against the original after our own reject, 150 pairs: 267 to
  30, and every pPr now matches. A bold or italic that an unrecorded parent
  (or the docDefaults) turns on stays in the record, so it is not rejected
  to off.
- Rejecting a redline in which Normal changed no longer strips the run
  properties of the styles based on Normal: the change record each of them
  gets keeps the style's own fonts, sizes and languages, font slot by font
  slot and language by language, over Normal's old ones, as Word records them (a67dcf9e05's Balloon Text went
  from Tahoma 8 pt to Times New Roman 12 pt). Paragraph-style property
  mismatches against the original after our own reject, over the same 150
  pairs: 298 to 236.
- `jubarte reject` no longer panics ("No parent for AddBeforeSelf") on
  nested block content controls whose every paragraph is inserted, as in
  Word's "Page Numbers" footer parts. A control whose every paragraph goes
  is dropped with them instead of being restored over the paragraph that
  follows. A control whose text alone is deleted, or that holds no run,
  stays (emptied) even when another paragraph mark of the part is deleted,
  as it already did otherwise.
- `jubarte accept` / `reject` (and `accept_revisions` / `reject_revisions`)
  now save what Word's Accept All / Reject All save. Measured against Word on
  51 of Word's own redlines: every paragraph's text and mark state match
  (5 files differed), 46 match run formatting too, and all 51 pass the
  OOXML validator.
  - A comment whose reference was deleted (or, on reject, inserted) goes with
    its anchors and its commentsExtended / commentsIds / commentsExtensible
    entries; with no comment left the comment parts are not written;
    people.xml keeps only the remaining authors.
  - A run of deleted paragraph marks continues through moved-from marks and
    across a table whose rows are all deleted.
  - A bookmark whose span was wholly deleted goes; an empty bookmark inside
    deleted text now survives, where the deletion was. No half bookmark is
    left behind.
  - Bookmarks and comments are renumbered from one counter in document
    order, as Word numbers them.
  - A paragraph style naming a character style is dropped, and no empty
    `w:rPr` / `w:pPr` is written.
- `jubarte reject` matches Word's Reject All on the text and mark state of
  all 100 of Word's redlines in the bench's `rejected_tracking` set (12
  differed). All outputs but one pass the OOXML validator; that one's input
  and Word's own Reject All fail it too.
  - A move range that ends inside a table no longer takes the table's
    `tblPr` and `tblGrid` with it (Word refused the file), and an emptied
    moved heading before a table goes instead of keeping a live `moveFrom`
    mark.
  - A footnote or endnote whose reference was resolved away goes.
  - Adjacent tables alike in every whole-table property (style, float,
    direction) become one table, as Word holds them; a later table's own
    borders, widths and margins ride on its rows as `tblPrEx`.
  - Rejecting a section change keeps the section's header and footer
    references. When a section break goes, the next section takes its
    headers and footers if it had none of its own. Header and footer parts
    no section shows are not written.
  - Rejecting a paragraph style's recorded change restores its old
    properties the way Word reads them: against Word's built-in defaults
    (Times New Roman, 10pt, single spacing, left-aligned), not against the
    style's ancestors. A property the old record lacks takes the built-in
    value, and a value the style would inherit anyway is dropped. The
    style's linked character style takes the restored run properties.
  - A numbered style's restored paragraph properties drop what its numbering
    level already says (the level's indents, tabs and spacing), as Word does.
    The level is the one Word numbers with: through a numbering style link,
    at the nearest defined ilvl, a full level override without paragraph
    properties saying none, and none at all under an explicit `numId=0`.
  - A linked character style takes its paragraph style's whole effective run
    properties, less what the docDefaults say, and a linked character style
    based on it follows. An old record's bold, italic or other run toggle
    reads against the nearest ancestor the same reject restores, as Word
    reads it. The built-in off hyphenation suppression, auto text alignment
    and auto colour are written where the style chain says otherwise.
- A redline re-caches every themed colour its styles carry against the theme
  it ships, as Word does: table-style shading fills and colours and border
  colours too, not only text colours. Tints and shades are truncated per
  channel the way Word writes them (accent 156082, tint 3F: B2DEF2).
- A redline whose original has no theme (or only one nothing references)
  ships Word's own default Office theme, byte for byte, instead of the
  revision's theme; Word never takes the revision's (664 of 664 bench
  redlines, and Word probes with a custom revision theme).
- A redline whose original has a theme but no styles part keeps the
  original's theme. It was replaced with Word's default theme along with the
  missing stylesheet.
- `jubarte debug A B -c text` / `-c runs` pairs header and footer parts by
  the section reference that shows them, so Word's renumbered parts compare
  with their counterparts.

## [0.10.0] - 2026-09-28

> **Summary.** Documents are now editable by agents: inspect, JSON edit plans that write a clean copy plus a tracked redline, and headers, footers and notes as editable stories, from the CLI, Python and WASM alike; Word-mode redlines keep changed text boxes, per-section headers and footers, and comment anchors where Word does.
>
> **Docs.** README links the page-by-page engine comparison site (top banner, Benchmarks, Find us) and documents self-update and --mode word|powertools; new docs/WORD_DIFFERENCES.md, docs/SELF_UPDATE.md, the jubarte-documents agent skill and the Acme letter example; the jubarte-wasm npm README covers the agent API; VERSIONING.md and bump-version.mjs make release.sh the source of truth.

### Added

- `jubarte A B --mode word|powertools` names the compare presets: `word`
  (the default, Word Compare's layout) or `powertools` (the classic
  PowerTools fallback, still also spelled `--powertools-faithful`).
  `docs/WORD_DIFFERENCES.md` lists where jubarte's redline still differs
  from Word's and which mode gives which.
- `jubarte-wasm` gains the agent surface the CLI and Python already have:
  `inspectDocument`, `documentMarkdown`, `sourceSha256`, `applyEditPlan`
  and `previewEditPlan` (refusals returned as data with their code and
  per-operation outcomes), `editReportJsonl` and `capabilities`, in the
  full and slim builds.
- Input admission for the agent surfaces: `inspect`, `text`, `edit` and
  the Python/WASM facades over them refuse a package before parsing it when
  it breaks a resource budget (64 MiB file, 10,000 entries, 64 MiB per
  inflated part, 256 MiB inflated in total, XML nesting 256), repeats a
  part name, uses an unsafe entry path, encryption or compression other
  than deflate, or is not a WordprocessingML package. Refusals carry the
  codes `INPUT_LIMIT`, `DUPLICATE_PART`, `UNSUPPORTED_PACKAGE`,
  `INVALID_PACKAGE` and `INVALID_XML`; `capabilities` reports the budgets.
  Compare keeps its historical tolerance.
- `jubarte inspect` reads a document as numbered paragraphs (JSON with a
  source hash, or Markdown) with styles, numbering, formatting spans and
  the structures an edit cannot address (fields, hyperlinks, content
  controls, revisions); `jubarte capabilities` reports what the build can
  do.
- `jubarte edit` applies a JSON plan of uniquely anchored operations
  (`replace`, `insert`, `delete`, `comment`, `insert_paragraph`,
  `delete_paragraph`, `format_paragraph`, `merge_paragraphs`)
  to a copy, previews it, and writes the clean copy, the tracked redline
  and a per-operation report. A plan that carries the source's SHA-256
  (`source_sha256`) is refused for any other source; without it the plan
  runs unguarded and the report flags that. An
  anchor that is not unique, or text inside a field (simple or complex,
  even one with an empty result), hyperlink, content control or revision,
  is refused. Comment text must be nonempty without control characters,
  and several `insert_paragraph` operations after one anchor keep plan
  order.
- `jubarte convert --png` renders pages to PNG (1–1200 dpi, at most 2^28
  pixels a page) and checks every output path before the first write.
- Python: `jubarte_redlines.read()` returns a `Document` with inspect, edit,
  convert, compare and accept/reject methods, and `python -m
  jubarte_redlines` exposes the same commands (compare is the explicit
  `compare` subcommand there); `EditPlanError.message`
  carries the engine's detail.
- Edit plans restyle and join paragraphs. `format_paragraph` sets a
  paragraph style (by id or name; an unknown one is refused with
  `UNKNOWN_STYLE` and the defined ids), alignment, line spacing as a
  multiple and space before/after in points, as one tracked property change
  that reject undoes. `merge_paragraphs` joins the next paragraph onto one,
  with an optional separator; the joined paragraph keeps the second one's
  properties, as Word's accept of a deleted mark does, and bookmarks and
  comment ranges between the two move to the join. `replace` and `insert`
  take a `format` (bold, italic, underline, a Word highlight colour) for the
  new text only. The Python `EditPlan` gains the matching builders.
- `replace` takes `"whole": true` (Python `whole=True`): the redline shows
  all of `find` deleted, then all of the replacement inserted, as typing
  over a selection with Track Changes on does, instead of Word Compare's
  word-level diff that keeps shared words. Deletions the comparer placed
  just outside the change are gathered in, formatting stays on each side,
  and a comment on the change stays on the inserted text. When the diff
  cannot be regrouped the operation keeps the word-level redline and its
  report line carries a `message` saying why.
- Headers, footers, footnotes and endnotes are editable stories. `inspect`
  lists them under `stories` and `jubarte text` prints them after the body,
  each paragraph under an id such as `header1:p:0` or `footnotes:p:2`. An
  edit plan addresses them by that id or by adding `"story": "footer1"` to
  an `index`, `starts_with` or `contains` selector (those search the body
  otherwise). The change is tracked in the story's own part, `"whole": true`
  included, and the report counts header and footer revisions. Comments
  stay body-only, since Word cannot anchor one in a header or footer. The
  Python `Snapshot` gains `stories`.
- Adoption guides, workflow examples and a document-operations agent skill.
  The Acme letter example now ships `make_letter.py`, which writes its
  source letter byte for byte, and its plan runs all twelve edits.
- `jubarte self-update [--check] [--yes] [--version X]` installs a GitHub
  release after checking its SHA-256 against the release's
  `SHA256SUMS.txt`. It contacts GitHub only when run; nothing checks for
  updates in the background. Build with `--no-default-features --features
  cli` for a binary without it (docs/SELF_UPDATE.md).

### Changed

- quick-xml 0.41 → 0.42 for input admission and XML checks. Admission now
  transcodes UTF-16 parts (SharePoint `customXml` items, which Word opens)
  before scanning them, so the new release's UTF-8 validation refuses none
  of the 500 fixture documents; bytes that are not UTF-8 still decode
  lossily, as before.
- Dependencies: clap 4.6.7, flate2 1.1.10 and serde_json 1.0.151 across the
  engine, Python, WASM, in-process and app workspaces; quick-xml 0.42 in
  the app and in the WASM build's patched `rdocx-opc`; the ooxmlsdk test
  oracle 0.12.
- `scripts/release.sh` requires a sixth note,
  `--how-readme-and-other-docs-were-updated`, written under the changelog
  summary and into the release commit, next to a list of the docs changed
  since the previous tag. `scripts/bump-version.mjs` warns that
  `release.sh` is the source of truth for releases.

### Fixed

- `jubarte-wasm`: the patched `rdocx-opc` decodes escaped attribute values
  when it reads relationships and content types, so a hyperlink target
  holding `&amp;` is written back once instead of as `&amp;amp;`.
- A text box whose text changed no longer disappears from a Word-mode
  redline. Three late passes that rebuild a revised paragraph from the text
  of its insertion and deletion also read the text inside the box, so the
  drawing and its VML fallback were dropped and both copies of the box's
  text landed in the anchoring paragraph ("…of the postOverall…"); accepting
  the redline did not give the revised document. They now leave any
  paragraph alone whose runs hold anything besides text. Present since at
  least 0.7.1 (fixtures_500 00b81efae883).
- Word mode marks a text box's changed words inside the one box, as Word
  does, in the DrawingML shape and its VML fallback alike, instead of
  deleting the old box and inserting the new one. A shape wrapped in
  `mc:AlternateContent` is now a word of its own in the Word-mode diff, as
  a bare drawing already was, so a changed box no longer takes the
  unchanged shape or text beside it into its replacement (a deleted and
  reinserted VML group repeated its shape id; fixtures_500 003329b501a7).
  Changing one word in every text box of the fixture documents now keeps
  all their boxes (22 of 22 documents).
- A document whose sections have their own headers or footers no longer
  redlines them against the wrong section's: they were paired by kind and
  type alone, so every section's default footer met the last section's,
  and an unchanged "Page 1 of 4" footer came out deleted and reinserted as
  "Page 4 of 4". Parts pair by section now, and an unchanged part is left
  alone.
- Word mode no longer invents paragraph property changes. The last
  paragraph of a story with a few words revised kept its spacing live only
  when the paragraph was replaced whole; otherwise its space before moved
  into a `w:pPrChange`, and an unchanged justified paragraph got an empty
  one. Word records neither (Word's own redlines of a body, a header and a
  text box), and the parity ladder's heading-4 pair drops the two
  `w:pPrChange`s Word does not write.

- Changes in a header or footer that carries a relationship (a logo, a
  hyperlink) are now in the redline. The comparer skipped every such part
  and kept the original's, so the redline silently lost the change and
  accepting it did not give the revised document. It now diffs the part
  whenever each relationship the revised part uses means the same thing in
  the original's (same type and target, same bytes for an image), which is
  the case for an edit plan and for most comparisons of two versions.
- A paragraph selector given as a bare id string (`"paragraph": "body:p:88"`,
  as the agent skill's own example writes it) failed to parse with "data did
  not match any variant"; only `{"id": ...}` worked. Both forms are
  accepted now.
- Two paragraphs joined into one now read as Word's Compare shows a join:
  the first paragraph's mark is deleted and only the separator is inserted.
  The paragraph LCS paired the joined paragraph with the first original,
  which stranded the second one's words behind that mark, so Word mode
  showed them as a move ("Delivery is DDP to the Buyer's site.", six words
  and more) or deleted and inserted them again. The cross-paragraph stream
  now also tries the region without that pairing and keeps whichever keeps
  more text, and it no longer fuses the last word of one paragraph and the
  first of the next ("TRIAL." + "Each") into one compound.
- `insert_paragraph` runs start from the anchor's body run, the one with
  the most text, instead of its first run, so a bold lead-in such as
  "(f) Notice of Inability to Comply." no longer makes the whole new
  paragraph bold.
- Comments carried into a redline land on their own occurrence of repeated
  text, not the first one: two comments on the same words in different
  places no longer collapse into one (Word keeps all six in the M35
  renumbered pair; jubarte kept four). A long commented range whose inside
  changed now maps by the text at its two ends instead of being dropped.
- Word mode keeps the built-in ids of styles whose id differs from their
  name (`CommentText` for "annotation text", `CommentReference`,
  `CommentSubject`, `MacroText`, `TOAHeading`, `TableofFigures`,
  `TableofAuthorities`). Renaming them left `comments.xml` pointing at
  undefined styles, which stripped the comment formatting.
- An unchanged last paragraph after a replaced block came out inserted
  and deleted again when the documents' first paragraphs matched: the
  positional paragraph zip no longer pairs a paragraph whose identical
  copy sits elsewhere in the other document (Word keeps it unchanged).

## [0.9.3] - 2026-09-27

> **Summary.** Redlines are now scored against Word's own redline of each pair, rendered by Word, and this release fixes what that exposed: redlines Word refused to open, blank field codes, missing fonts, and misplaced equations.

### Fixed

- A field whose code stayed while its result changed now stays one field, as in Word's redline. Only the result's words are marked. Before, the field's begin, separate and end were glued to the result's first and last words, so "Contaminated Sites Act 2003" against "Firearms Act 1973" (a STYLEREF title) matched only " Act ". That left a deleted and an inserted field whose ends crossed, and Word refused the redline (db433183 × 9377099d).

- A field whose code changed is now replaced whole, nested fields included, as Word's redline does. Every part of a field carries the codes of the fields around it, so an `IF` that lost `\*MERGEFORMAT` no longer keeps its begin and inner DOCPROPERTY fields while the rest is half deleted (98bf5f3d × a3701d36).

- A changed table of contents no longer opens inside its replacement. Word mode's head junction folds a short deleted title into the first inserted paragraph when both share a word. When that title was a TOC's first entry, the fold carried the deleted TOC's begin above the inserted TOC's end, and the two fields crossed (98bf5f3d × a3701d36, "Part 1—Preliminary" against "Part 1—Introduction"). A fold that would carry half a field across other fields no longer happens.

- The same input writes the same bytes. Another dependency turns on the `zip` crate's `time` feature, and with it every package entry carried the time of the write, so two runs a second apart gave different files. Entries are now dated 1980-01-01, as Office dates its own.

- Two point comments inside one run now keep their places. An anchor that ended one text piece of a multi-piece run went after the whole run, so "Note" landed after "東京" and behind "Second note". The run now splits between the pieces.

- Rejecting every change in center_alignment × center_aligned_bold now restores the original. A pass forced the inserted "This" and "text" into kept text while the next paragraph still deleted them, so the rejected document read "This text This document …". The cross-paragraph stream already keeps those words where Word does, so the pass is gone. The revised tail now fuses into the sole deleted paragraph and keeps that paragraph's deleted mark, so accepting no longer leaves an empty last paragraph.

- Replaced regions now follow Word's replace-gap grammar, which Docxodus 12 decodes.
  - **Empty paragraphs.** An empty paragraph found in both documents no longer anchors two unrelated regions. It pairs only as part of Word's pilcrow chain, or as the two stories' final marks. The chain breaks when an original paragraph with words faces an empty one.
  - **Interior replaces.** Inside the body, a replaced region keeps every old and new paragraph whole: new paragraphs first, each under an inserted mark, then the old ones under deleted marks. Before, "TWO" fused into "e" and "A" into "a" (list_with_table_break × broken_complex_list, docxide 13.2 vs Docxodus 93.3).
  - **The story's tail.** When the revised document's last paragraph holds text, that text fuses into the first deleted paragraph of the tail, even across a deleted table. That paragraph keeps its own properties and deleted mark, so accepting every change no longer leaves a stray empty paragraph (support_tickets_table × support_tickets_summary, diff_doc2 × numwords). The same holds when the original runs on past it to its own end (bullet_list × calibri_bold_italic: "Calibri bold italic …" opens "Apples", then "Bananas" to "Grapes" deleted). A new title facing an unrelated old one is inserted whole before the old one's deletion (pirates × table_left_indent, as Word does).
  - **Final empty paragraphs.** When both documents end on an empty paragraph, those two final marks stay paired behind a trailing deletion.
  - **Result.** Paragraph-structure agreement with Word's own redlines over 747 pool pairs rose from 0.9276 to 0.9331: 50 pairs improved and 9 dropped, none by more than 0.08.

- Redlines keep the bookmarks of both documents, as Word's Compare does. The WmlComparer port dropped every bookmark, so each updated TOC line, `PAGEREF` and `REF` printed "Error! Bookmark not defined." (file_21 × file_22 lost all 582). A bookmark in both documents appears once, at its place in the revised text. A bookmark only in the original stays beside its deleted text. Ids never collide with revision ids, and Word's hidden `_GoBack` is dropped, as Word does. Pool pairs with more broken references than Word's own redline went from 29 to 0. Comment anchors gain two fixes from the same pass: moved text now counts once on each side, and a second anchor inside a run that holds several text pieces no longer reorders that text.

- The redline's Normal style now follows Word's rules for merging the two
  documents' defaults. Word writes B's docDefaults indents, justification,
  line-unit spacing and borders into Normal (and neutralizes the ones only A
  sets), writes B's run defaults whenever either Normal stores paragraph or
  run properties, writes only the language attributes that change, writes the
  implicit 10pt complex-script size, reads a document with no docDefaults at
  Word's factory values (after 160, line 278, kern 2), and finds a
  LibreOffice Normal (`style0`) by its name. Across 738 corpus pairs, Normal
  run properties that differ from Word's drop from 79 to 38 and paragraph
  properties from 54 to 46.

- Point comments (a comment reference with no range markers) survive the
  redline. The carryover only mapped ranges, so a point comment was dropped,
  and with it the whole comments part when it was the only one. It is now
  written as an empty range right after the text it follows, the form
  Word's own redline uses (comments.docx comment 2, in comments ×
  complex_style_attr and clear_formatting × comments). An empty range is
  written as one group: a lone start used to land in the next paragraph.

- A list copied from the revised document keeps its picture bullets: the
  bullet definitions and their images now travel with the list. The list's
  levels had named a picture bullet the merged numbering never defined, and
  Word refused to open the redline (italic_rstyle_combos ×
  paragraph_indent_normal_styles: harness 0 → 53.2; Docxodus 0). When both
  documents define the same bullet id, the revised bullet takes a fresh id
  and keeps its own image. A part's first new relationship is now `rId1`, as
  Word numbers them, not `rId0`.

- The final paragraph marks are also paired when the replaced original has a
  single paragraph: its text is deleted into the revised empty final
  paragraph. The deleted text had joined the revised document's last content
  paragraph and the revised final paragraph was dropped, so accepting the
  redline lost that paragraph (fields_attrs1 × cli_legacy sample: harness
  23.4, Docxodus 73.1).
- A paragraph property the revised document adds to a kept paragraph is
  recorded in `w:pPrChange` whatever it is, as Word records it; only added
  alignment or spacing were recorded, so rejecting the redline kept, for
  example, an added outline level.
- When an unrelated document replaces the original and ends on an empty
  paragraph, the two final paragraph marks are paired as Word pairs them:
  the original's last paragraph is deleted into the revised final paragraph,
  which keeps the revised properties. The redline had given that paragraph
  the original's bare properties, and Word's taller final line, which moves
  the redline onto a second page, was missing (diff_after8 ×
  doc_with_spacing: harness 2.8, Docxodus 93.9). The revised document's last
  content paragraph keeps its own inserted mark instead of folding into the
  first deleted paragraph; a head junction on a shared word still folds.
- A run of changed body paragraphs is compared as one stream of words and
  paragraph marks, as Word does (port of Docxodus's DocxDiff in-gap pairing
  and cross-paragraph segmenter, `comparer::cross_para`): a kept word may now
  sit across a paragraph mark, and rejecting the redline restores the
  original instead of repeating a re-kept phrase. Word-faithful gates keep a
  gap unpaired where Word does: a region with no paragraph pair streams only
  when the revised side has no more paragraphs and its first kept word opens
  a paragraph on both sides; a lopsided same-slot pair needs three shared
  content words; a window of function words alone carries nothing across a
  mark; a table between two changed runs keeps the LCS pairing; and the next
  paragraph takes a same-slot pair when it shares at least twice the content
  words in order (bold_rstyle × bold_vals).
- An inserted tail ahead of a deleted tail pairs the two final paragraph
  marks, as Word does: the last inserted paragraph joins the first deleted
  one with a deleted mark, so accepting the redline no longer leaves an
  empty paragraph the revised document never had (bullet_list_bold ×
  bullet_list).
- Word-mode table margins follow Word: `tblInd`/`tblCellMar` of 10 twips is
  stamped on a bordered table only when the document the table comes from
  (the original for a wholly deleted table) has no default table style. 94
  bordered tables from `TableNormal` documents stay bare in Word's redlines;
  stamping them shifted every row below (file_46 × file_47).
- Synthesized numbering for a dangling `numId` copies Word's level geometry:
  a `num` tab at the text indent and a full 720-twip hanging indent, so level
  0 puts its number at the margin.
- Redlines no longer cross or pack complex fields, which crashed Word
  ("Connection is invalid", English pair 57f96361×3832d290) once field codes
  were kept: a field result only matches text inside a field with the same
  code, the insert-before-delete swap leaves `fldChar` wrappers in place,
  and `SetAfterUnids` aligns the two ancestor chains at the paragraph instead
  of the top (an SDT-wrapped paragraph against a bare one gave every revised
  run one Unid, packing a footer's text, field begins, codes and tabs into
  one run).
- Redlines Word refused to open now open (14 of 19 English failures):
  relationship and content-type attributes no longer gain an `&amp;` per
  round trip, a case-duplicate `Default` extension is merged, and
  relationship targets are relative to the source part's folder (904e989).
- A deleted text box is deleted whole, as Word's Compare deletes it: every
  run of its story sits in a `w:del` of its own and its paragraph marks are
  deleted. A text box is a story of its own, so the `w:del` around its anchor
  does not reach into it, and a field there kept its field code in a run no
  deletion wrapped (`w:delInstrText` outside any `w:del`). Word refused both
  English redlines that had one (30ff840c, bb113e88); the OpenXmlValidator
  passed them. Both open now and score 0.673 and 0.557 jaccard against Word's
  own redline, where every other tool scores 0.
- A carried bookmark stays out of a content control its source bookmark was
  not in. Placement follows the text, so B's body-level `_Toc` bookmarks
  around a data-bound title control landed inside it (7b649361), and a
  bookmark landed in a dropdown cell (57c181da). Word refused both redlines
  and opened each once those bookmarks were dropped. An endpoint now leaves
  the control: a start before it, an end after it.
- Scratch `pt:Unid` attributes no longer leak into restored deleted-paragraph
  spacing, and every extension namespace (w14/w15/w16*/wp14) is listed in
  `mc:Ignorable` on each part root (b7fedc7); w16 serializes under Word's 2018
  wordml namespace (722de2a).
- Changed field codes keep their instruction text. An inserted or deleted
  `w:instrText` was re-emitted empty, so tracked PAGE, REF and TOC fields
  rendered blank in Word (8ab1df8).
- Footnote ids renumbered by the comparer stay clear of the revised
  document's `continuationNotice` id (fee9411).
- The revised document's fonts join the output font table. A font used only
  by B had no `w:font` entry, so Word substituted Times New Roman (2eb24d5).
- A base made only of display equations counts as content, so an unrelated
  pair keeps Word's order (inserted text first, deleted equations after)
  instead of merging the first equation into the first inserted paragraph
  (e4610b0).
- `get_revisions` and `compare` return an error on bad input instead of
  panicking (cb33d11).
- sha1 0.11 drops the vulnerable block-buffer 0.10.4 (GHSA-qwgh-2vcv-g2f7)
  from the engine, Python and WASM locks (9cddeb3).
- convert: a cell holding only a nested table splits without panicking
  (4dda25e); rotated oval pictures, comments on vertical pages and scaled
  vertical runs follow Word (832eac9, c8a878e, b41cb35); pie and gear text
  rectangles follow Word (2e4cd8f); altChunk decodes base64 MHT parts and
  tolerates omitted end tags (93643b5, 30d595f).
- Redline and other rewritten packages are byte-reproducible: zip entries are
  written in the source package's order, added parts after them by name,
  instead of in hash-map order that changed on every run.
- Changed paragraphs that all correspond position by position (each shares a
  content word with its counterpart, docxodus's same-slot rule decoded from
  Word) are paired in place, as Word pairs them, instead of one body pairing
  with the next on a shared trailing word and the rest falling out
  whole-inserted and whole-deleted. file_111×file_112 scores 0.108 → 0.538
  Jaccard against Word's redline; +0.35 summed over the 1,195 pool and
  English pairs (8 changed).
- When the revised document names its default fonts by theme
  (`w:asciiTheme="minorHAnsi"`, as Word writes them), the redline's Normal
  style now carries those theme fonts instead of keeping the original's named
  face, so the text renders in Calibri as Word's redline does rather than in
  Times New Roman. +2.96 Jaccard summed over the 1,195 pool and English pairs
  (126 changed, 26 better, 2 worse); instrtext_angled_brackets_bug ×
  table_merged_cells 0.13 → 0.82.
- A paragraph whose layout changes on both sides (double spacing replacing
  heading spacing) now keeps the new layout and records the old one as a
  paragraph-property change, as Word does. Equal properties had looked
  different because of the comparer's internal `pt14` bookkeeping attributes;
  those no longer count. A revised paragraph with layout of its own no longer
  inherits the original's small `after` spacing. +0.59 Jaccard summed over the
  1,195 pool and English pairs (197 changed, 0 worse by more than 0.005);
  document_100 × double_spacing_bold 0.48 → 0.85, file_111 × file_112
  0.54 → 0.75. Paragraph-property changes that disagree with Word's count
  fall from 1,784 to 1,420.
- Recorded old run and paragraph properties no longer carry a stray
  `xmlns:ns0="http://powertools.codeplex.com/2011"` declaration left over from
  the comparer's internal bookkeeping (349 of the 1,195 pool and English
  redlines had one in the body; now none). Rendering is unchanged.
- When two unrelated documents both end in an empty paragraph, the wholesale
  replacement now keeps that story-final paragraph live after the deletions,
  as Word does, instead of welding the revised document's trailing empty
  paragraph onto the original's first deleted paragraph (which then kept a
  live mark and the revised styling). +0.86 Jaccard summed over the 1,195
  pool and English pairs (96 changed, 13 better, 4 worse);
  line_break × line_space_table 0.29 → 0.51.
- A redline no longer invents two empty inserted paragraphs before a deleted
  title when the table after that title is deleted wholesale. Word adds those
  spacers only when it pairs the table cell by cell. quarterly report table ×
  red bold heading demo 0.24 → 0.26; no other pair of the 1,195 changes.
- Unrelated documents too short for the wholesale shortcut (for example a
  title and a table against three headings) now get Word's junction when they
  share no word of four letters or more. The revised document's last
  paragraph joins the original's first paragraph, whose mark is deleted.
  Previously full LCS paired a stray digit and kept the two apart. +1.24
  Jaccard summed over the 1,195 pool and English pairs (33 changed, 4 better,
  1 worse); sd_1494 table left indent × sdpr title-only 0.22 → 0.84,
  quarterly report table × red bold heading 0.26 → 0.68. The one loss
  (−0.025) is an NDIS footer whose junction now matches Word's redline
  paragraph for paragraph.
- A single shared word of four or more letters or digits that opens or
  closes a paragraph on both sides now stays an anchor in a long unrelated
  window, as in Word's redline ("Second" opening both "Second green
  underlined item" and "Second page"). The detail threshold had voided it at
  one word in 54. green underline bullet list × header without relationships
  +0.09; no other pair of the 1,195 changes. A general lone-word anchor was
  tried and rejected (−3.07 summed, 30 worse).
- A wholesale replacement between unrelated documents now anchors on a word
  that ends a paragraph on both sides and applies Word's seam on each side of
  it: "2026" closes both "Product Roadmap 2026" and "Date: February 1, 2026",
  so that paragraph keeps "2026" and its mark and replaces the words before
  it. +0.31 Jaccard summed over the 1,195 pool and English pairs (2 changed,
  both better); product roadmap × project plan +0.28.
- When the revised document ends inside a stretch the original continues past
  (a Greek alphabet list against "Meeting Agenda" and a table), the two final
  paragraph marks now pair as Word pairs them: the revised last paragraph joins
  the original's first deleted paragraph under the original's properties with
  a deleted mark, and the paragraph after the deleted table takes the revised
  properties. The table and everything after it no longer sit a line high.
  +0.91 Jaccard summed over the 1,195 pool and English pairs (2 changed, both
  better).
- Short unrelated documents that both end on an empty paragraph no longer weld
  the original's first paragraph onto the revised document's last one. Word
  inserts the revised document whole, deletes the original, and pairs the two
  final empties (a titled table against an item list). +0.93 Jaccard summed
  over the 1,195 pool and English pairs (7 changed: 5 better, 2 slightly
  worse).
- Stamped demo bodies zip positionally only when a body pair shares a word of
  five letters or more. Sentences that share just "This" and a full stop
  (Calibri heading × underline) now take Word's shape: the revised first body
  inserted whole, the revised last body joined to the original's first.
  +0.14 Jaccard on the one pair that changed.
- Custom style ids derived from style names keep only letters and digits, as
  Word's do. "Normal (Web)" became `Normal(Web)`, which orphaned the revised
  document's live `NormalWeb` paragraphs onto a custom style. +0.82 Jaccard
  summed over the pool and English pairs (8 better, none worse).
- A list whose `numId` both documents use but define differently (a circle
  bullet against a disc) now moves the unchanged items to the revised
  definition and records the original `numId` in a `w:pPrChange`, as Word's
  own redline does. They used to keep the original's bullet with no change
  recorded. The PDF converter does not yet paint Word's struck-old /
  inserted-new marker pair, so its own proxy scores the pair mixed
  (circle × disc +0.36, disc × square −0.47; +0.05 summed).
- Redefined paragraph styles now carry the revised document's effective
  fonts, sizes and spacing as a delta against the output's own chain and
  docDefaults, which is Word's rule (mined over 4,924 tracked styles in the 747
  pool redlines; our style values now match Word's on 42,542 of 42,993, up
  from 33,681). A revised heading that inherited Arial and line 276 from its
  own docDefaults used to render in the original's theme font and line pitch.
  +3.73 Jaccard summed over the pool and English pairs (45 better, 15 worse,
  every loss under 0.04).
- A revised document without any `w:sectPr` now gets Word's default section
  (Letter, one-inch margins, one column) as the live body section, with the
  original's section recorded in a `w:sectPrChange`. Word's redline does the
  same in all six such pool pairs. The body used to keep the original's
  section with no change record, so a two-column original stayed two columns
  and 0.5-inch margins stayed narrow (invalid_list_def × tiff +0.08; sd_1480
  × missing_sectpr now paginates like Word, but the converter's proxy scores
  it −0.07 because it does not yet paint Word's formatting change bar on the
  second page).
- Direct `line=276` spacing is dropped as a restated default only when the
  paragraph's own source document (the original for deleted paragraphs, the
  revised one otherwise) resolves line 276 for an unstyled paragraph. A 276
  over a single-spaced Normal is a real value and Word keeps it; the strip
  used to remove it regardless (sd_2517_localized_heading_styles: 30 inserted
  paragraphs in each of four pool pairs lost their line pitch). Our paragraph
  line spacing now matches Word's on 8,895 of 8,944 inserted pool paragraphs,
  up from 8,779. +1.03 Jaccard summed (16 better, 5 worse, every loss under
  0.03).
- `convert --revisions word` now paints the change bar beside paragraphs,
  including paragraphs in table cells, whose only revision is a formatting
  change (`w:pPrChange` or `w:rPrChange`), as Word does. Such paragraphs used
  to get no bar. Converting the same redlines, 177 pool PDFs changed:
  +1.41 Jaccard summed, 27 better, 1 worse (-0.026). The worse pair carries a
  `firstLine="0"` paragraph change that Word's compare does not record.
- A trailing empty paragraph that ends both documents with the same spacing
  keeps that spacing live, unrevised, as Word does. It used to be moved into
  a `w:pPrChange` over an empty paragraph, which also drew a change bar Word
  does not show (super_editor complex2×complexexport1, +0.009).
- A deleted Title or Heading that opens a run of deleted paragraphs is no
  longer mistaken for a checklist cell ("Table Widths" has only two short
  words). Word mixes it with the last inserted paragraph, and the mixed
  paragraph keeps the deleted heading's properties and deleted mark. It used
  to stay a separate paragraph and push every line below it down
  (file_134×file_135 0.12 → 0.41). +0.37 Jaccard summed over 14 changed
  redlines, 3 better, 2 worse. The larger loss comes from an inserted
  text-box paragraph that we already mixed wrongly, where Word keeps it
  inserted.
- The installed-font index (`font-index.tsv`) is rebuilt after an upgrade.
  It stores the answers of the family search, and a new release that changes
  the matching rules used to keep the old release's faces, or its misses,
  for as long as the font folders stayed unchanged.
- An empty page break after a TextHeading now keeps the blank page Word
  keeps. The break paragraph is about a line plus its space after (sd_2517's
  breaks are line 276 and after 200, roughly 23pt on the 12pt face). When
  less than that remains, Word moves the paragraph to the next page and the
  break still fires. Those breaks had been kept on the current page, so
  sd_2517 and its randomized copy came out 99 pages against Word's 107. A
  full page of overflow still skips for every manual break. The wider
  leftover applies only to TextHeading, so a short gap no longer invents a
  blank page on the three-page and thirteen-page fixtures.
- A bottom border on a paragraph inside a table cell is drawn once, on the
  last paragraph of a run that shares that border, including when the
  paragraph still has text. Deleted cell bottoms in file_146 — `bun run dev`,
  the npm and github lines, and the code cell's closing brace — were missing,
  so the pale rules wider than 200pt went from 30 to Word's 33. An empty
  paragraph in the middle of that run no longer draws a rule of its own.
- Tab Alignment against Tab Tests is mixed paragraph by paragraph, in order.
  Word mixes the title and the three following tab lines with Tab Tests'
  four paragraphs and deletes the rest. Flat word matching mixed only the
  title and one empty line.
- A folded Demo title keeps "Demo" inside the deletion. It used to be peeled
  into a live " Demo", which is in neither document, so accepting the redline
  left the word behind (double spacing × eigenpal, and document 100 × the
  comments addition).
- A sentence period shared by both documents stays on both sides when one
  side continues into the next paragraph. The period goes to the side that
  already finished its sentence, and the continuation gains its own period
  at the end (font color × font family, italic underline × justified
  underline, justified underline × justify alignment).
- A deleted section that repeats the heading inserted in front of a table is
  placed back before that heading. The comments redline had moved the
  section to after the capability table, so the original's characters were
  all present and in the wrong order.
- An empty paragraph-property shell is left out. The mixed title on blue
  centered title × blue italic carried an empty `pPrChange`, and center
  bold × clear formatting carried three empty `w:pPr` elements. Word's
  redlines of those pairs have neither. A pilcrow mark that is the only
  thing in the property stays.
- The 207-pair parity sweep reports 0 NEW keys, and
  `tools/parity_baseline.tsv` is unchanged. Spellcheck marks, the
  pagination cache, and header or footer references Word writes on its own
  stay outside the histogram.
- A merged table keeps `w:tblPrChange` last in `w:tblPr`. The default table
  look is written before the change, and the old properties stored inside
  the change are left without a synthesized look.
- A bare `w:cantSplit` or `w:rtl` stays. On those elements a missing value
  means on, and only an explicit false is dropped.
- A table that already sets a nonzero top or bottom cell margin keeps it.
  Row exceptions of zero are written only when the table did not set one.
- PowerTools-faithful compares no longer receive Word's table look, row
  margin exceptions, or the other Word-visual cleanup. That preset's
  contract is that those passes stay off.
- A paragraph whose deleted mark also holds inserted text keeps its
  `line=276`, as Word's redline keeps it (simple_ordered_list ×
  sublist_issue, "Lvl 1 – a"). Only a paragraph that inserts nothing drops
  that restated Normal spacing.

### Added

- `jubarte debug FILE [FILE2]`: a short Word-validity triage of a redline,
  for the shapes Word refuses and the OpenXmlValidator passes. It prints
  counts by kind with a few examples (`-n`), and with two files only what
  differs. Checks (`-c`): `orphans` (deleted text outside its story's
  deletion), `fields` (nesting per story, partly deleted fields),
  `bookmarks` (unpaired, duplicate, crossing a control, cell, text box or
  revision, or inside a plain-text or list control), `package` (content
  types, relationships, dangling references, undeclared `mc:Ignorable`
  prefixes), `structure` (blank field codes, cells not ending in a
  paragraph, nested same-kind revisions), `ids` (revision and `docPr` ids
  used twice; Word opens such files, so it is opt-in), `chains`, `elements`
  and `textbox`. `--list` lists the entries, or with two files the ones that
  differ. On the 450 English redlines of c6307ac, orphaned field codes and
  bookmarks in single-value controls flag only files Word refused.
- convert: Korean page and list number formats (b8a19b1).
- RESULTS.md tables scored against Word truth: Word's redline of the pair,
  converted by Word. Rows split redlining (tool redline → Word PDF) from
  conversion (Word redline → tool PDF) (6b72be2, a6953f1).

### Changed

- RESULTS.md applies one rule to every tool. Each table keeps a tool's
  latest run: jubarte no longer shows its best run of each week. The pooled
  docx→pdf tables rank only on the corpora every ranked tool converted, over
  the same documents, and a document a tool has no score for counts 0. Each
  corpus gets its own column, so jubarte's clean ranking now covers 1,204
  documents (English a + b and docxide's suite) instead of 2,102.
  - Competitors join fixtures_500 from their 2026-09-22 run: docxide 0.2673,
    soffice 0.3365, jubarte 0.6524.
  - A run over 7 days older than its table's newest is marked †.
  - Harness Docs counts every attempted document; failures already scored 0.
  - Redline speed is split by pair set, so a 5,000-pair run is never ranked
    against a 90-pair one, and 2-pair probes are dropped.
  - `--compress` is listed unranked.
  - Tables with no jubarte-redlines run, and the retired TypeScript ports
    (jubarte-first, -native, -lossless, dist/jubarte-final), are left out.

### Performance

- LCS keys follow their word hash in one walk; `group_by_key_stable` hashes
  each key once; move detection counts words and tokens in one walk (3d4ef7f,
  4a05c21, 241e9ac).
- PDF conversion parses `settings.xml` once per document instead of once per
  setting it reads, and the XML name interner hashes with foldhash. Cold
  one-shot conversion is 4.4% faster over 80 fixtures_500 documents; the
  output PDFs are byte-identical on 150.
- A font face looks its glyphs up in the cmap when asked instead of listing
  every mapped codepoint when it loads: a further 4.2% off cold conversion
  over 80 documents, PDFs byte-identical on all 500 fixtures_500 documents.
- An on-disk font index (`font-index.tsv` beside jubarte's font folder)
  remembers which files each font family resolved to. Later processes read
  those files directly instead of listing and opening every candidate in the
  system, Word and cloud-font folders; a changed folder or file sends the
  family back to the full search. Cold conversion median 46.9 → 21.4 ms over
  80 fixtures_500 documents, PDFs byte-identical on all 500.
  `JUBARTE_FONT_INDEX` moves it, `off` disables it.
- Each document remembers which face a catalogue font lands on instead of
  string-matching the family for every glyph, and the PDF writer prints glyph
  ids, pen moves and text state without per-glyph allocation. Together they cut
  another 9.5% off cold conversion (median 22.1 → 18.0 ms over 80 documents);
  PDFs byte-identical on all 500 fixtures_500 documents.
- An image repeated across pages (a header logo) is compressed once, not once
  per page before its duplicates were dropped: 2.1% off cold conversion over
  80 documents, PDFs byte-identical on all 500.
- The first-descendant lookups style and run-property reads make stop at the
  first match instead of walking the rest of the subtree: 1.6% off cold
  conversion over 80 documents, PDFs byte-identical on all 500 and redlines
  identical on 300 pool pairs.

## [0.9.2] - 2026-09-26

> **Summary.** A Word-fidelity pass on jubarte convert: layout rules reconstructed from live-Word probes, painted revision marks (--revisions conventional|word|custom), Word's comment balloons and change bars, East Asian layout, and smaller PDFs (--compress averages 0.70x Word's size on 2,102 documents). The redline engine is unchanged from 0.9.0.

A Word-fidelity pass on `jubarte convert`, driven by a 500 + 500-file corpus
of Word-exported PDFs (HF `superdoc-dev/docx-corpus`, plus 451 Word redlines
of them). Every rule was reconstructed from a live-Word probe — synthetic
document, Word export, measured numbers — and is written down with its probe
and implementing commit in [`docs/WORD_LAYOUT_RULES.md`](docs/WORD_LAYOUT_RULES.md).
The per-document bake-off on the 500-file set moved from 129/500 to 284/500
wins between milestone merges (mean Jaccard 0.134 → 0.360 at the first).

The redline engine is unchanged: `compare_documents` emits the same bytes as
0.9.1. All work is in `src/convert`, the CLI, and the OPC package layer.

### Performance

- Smaller PDFs, same pages. Subset faces now also drop glyph names
  (`post` format 3), unused metrics and every `cmap`/`name` record the PDF
  does not read; a collection (`.ttc`) face subsets like a lone font instead
  of embedding the whole collection. A CID font's `/W` lists only the glyph
  ids the pages use (a CJK face's widths were 212 KB of a 319 KB PDF). A
  picture repeated on many pages is embedded once. Plain glyph runs in the
  same font, size, colour and tracking share one text object and move by
  relative `Td`, so a page is no longer one `BT … ET` per glyph. Word-device
  glyphs (11.04/16.08pt body sizes) keep their own `q … cm … Q`: MuPDF hints
  them differently when the 0.24 scale sits in the text matrix.
- With `--compress` the average PDF is smaller than Word's on every corpus:
  145 KB against Word's 208 KB (0.70×) over the 2,102 clean documents and
  377 KB against 449 KB (0.84×) over the 1,416 redlines. Glyph positions are unchanged and the docxide-metrics
  scores with them: mean Jaccard 0.6469 clean / 0.4654 redlines against
  0.6468 / 0.4656 without `--compress`.

### Added

- **Painted revision marks.** `jubarte convert --revisions conventional|word|custom`
  with `--revision-palette` for `custom` (library: `PdfOptions::revisions`,
  `RevisionStyle` / `RevisionPalette`; Python:
  `docx_to_pdf(…, revisions=…, revision_palette=…)`; wasm/npm:
  `docxToPdf(bytes, compress?, revisions?, revisionPalette?)`).
  `conventional` (the default) is the legal-redline convention — deletions
  red struck through, insertions blue double-underlined, moves green;
  `word` reproduces Microsoft Word's own markup, including the per-author
  colour palette by first appearance, and is what the fidelity gates
  measure.
- **Word's redline chrome.** A document with comments — or an inserted or
  deleted table cell — gets Word's balloon pane, scaled by the right margin;
  change bars stand where Word draws them.
- **Fonts install beside the binary.** `scripts/install.sh` (Windows:
  `scripts\install.ps1`) copies the bundled open faces — Roboto Condensed
  (the exact files Word's cloud cache draws) and Selawik, Microsoft's
  metric-compatible stand-in for Segoe UI — into `$JUBARTE_FONT_DIR` or the
  per-user font directory instead of embedding them. Word's own cloud-font
  cache is read too, including localized family names and folders with
  numeric file names; macOS fonts are found by family name and Mac Roman
  names, not only file names.
- **East Asian layout.** Word's CJK fallback faces and East Asian families
  named outside `fontTable.xml`; vertical `w:textDirection="tbRl"` sections;
  ideographic line breaking with kinsoku; closing CJK punctuation hangs past
  the right edge; Latin inside East Asian text takes the `ascii` face and a
  quarter-em gap; Chinese and Taiwanese counting list formats; East Asian
  faces take Word's 1.3× line.
- **Right-to-left.** `w:bidi` paragraphs mirror their alignment and indents;
  `w:bidiVisual` tables run from the right margin; RTL lines paint in visual
  order; RTL runs take their `w:szCs` size; `minorBidi`/`majorBidi` theme
  slots take the complex-script face; Thaana fallback; a missing Arabic or
  Hebrew charset face is Arial.
- **`w:altChunk`.** HTML and MHT alternate content renders.
- **Frames.** `w:framePr` — including a style's `framePr`, and in headers
  and footers — floats its paragraph; page-anchored body frames float as
  boxes; auto-height frames grow to their paragraphs; a frame narrows every
  paragraph beside it and needs more than an inch of column to do so; with
  room on both sides its label sits on the left.
- **Text boxes and shapes.** Header/footer text boxes; shapes and text boxes
  anchored in table cells and inside content controls; list paragraphs in
  boxes keep their marker and indent; `sizeRel` percentages take their own
  frame and `spAutoFit` boxes fit their text; VML groups place children in
  group coordinates and honour `grpFill`; VML lines stroke where Word draws
  them; `w10:wrap`; VML pictures honour the `imagedata` crop; drawing groups
  and canvases; custom geometry paths in EMU with bound-arc tessellation;
  `a:prstClr`/`a:sysClr` colours; block arrows point their own way.
- **Effects.** `a:softEdge` fades, `a:duotone` recolouring, Word's "Washout"
  picture watermark, ellipse picture crops, text outlines with no fill, and
  `w:w` horizontal text scale measured as well as painted.
- **Metafiles and rasters.** EMF paths, Béziers and window/viewport mapping;
  WMF `META_POLYPOLYGON` with 0-based object handles; BMP and GIF pictures;
  CMYK photos embed as RGB.
- **Fields and forms.** `FORMCHECKBOX` legacy form fields; `PAGE` counts the
  page, in header/footer text boxes too; `w:fldData` stays binary.
- **Page furniture.** `tblCellSpacing` gaps; continuous sections that switch
  columns and side margins mid-page; page borders, including the medium-gap
  inward band and page-relative placement; `vAlign=center` centres the
  layout box; `titlePg` without a first-page part leaves page one bare; the
  page background lays out a header where the section has none; each
  SmartArt diagram paints its own drawing; a chart without `c:title` paints
  no title.
- **Word's layout rules** (probed, then implemented): `docGrid` charSpace in
  4096ths of a point, halved for half-width; compat mode 15; widow/orphan
  control on by default as in Word; `keepNext`/`keepLines` holding table
  rows together; `pageBreakBefore` never skipping a page; `contextualSpacing`
  per paragraph; baselines on the 0.24pt device grid and the left margin on
  the 1/300in grid; justified Word 2013 lines squeezing their spaces to keep
  a word.
- **`/ToUnicode` on every Identity-H font**, so text copies and searches out
  of the exported PDF; a glyph shaped from several characters maps to all of
  them.

### Fixed

- **List markers with an `hAnsi`-only face.** A numbering level whose
  `w:rFonts` names only `hAnsi` (e.g. Arial Unicode MS) painted its ASCII
  number in that face and took its taller line; `hAnsi` covers non-ASCII
  characters only, so `1.` now stays in the text's ASCII face and line
  pitch, as in Word.
- **Floating objects and wrapping.** Square/tight/through wraps carve the
  wrap polygon — a blocked line steps past the polygon, not the extent;
  `topAndBottom` floats push later lines; tight page banners move the
  paragraph above their anchor; behind-text boxes paint under the body;
  `distT`/`distB` stay off the body for header floats; front floats stack by
  `relativeHeight` across pictures and boxes; `wp:align` aligns within its
  `relativeFrom` frame; a float anchored after a page-spanning paragraph
  lands on its last page; floating tables keep the lines above their offset,
  break across pages when taller than the page, and lift to keep a page of
  rows.
- **Tables.** Autofit measures a word by the pieces it may break into and
  widens to the longest word — a too-narrow column takes the saved grid, a
  spanning cell widens its columns, vertical text keeps its width; `pct` vs
  `dxa` widths under `noWrap`; a zero `dxa` width is auto; `gridBefore` /
  `gridAfter` leave their grid columns empty; `tblPrEx`; cell margins and
  `tcBorders` edges apply per Word's order; vertically merged cells grow the
  last row they span; nested tables add no tail to their cell; `w:hideMark`
  rows and cells; PHPWord tables without `tblInd` stay at the margin; a word
  wider than its cell breaks at the edge.
- **Headers and footers.** Floats, text boxes and pictures — wrapping into
  rows, beside text, behind text, framed — sit where Word puts them; tabs
  past the margin wrap and wrapped lines drop the logo height; lines take
  the paragraph's style, `jc`, line rule and spacing; bottom borders take
  room; tracked changes and ptabs paint.
- **Lines, fonts and measurement.** A line is as tall as its tallest painted
  face, marker included; `hhea` — not OS/2 typo — metrics for the single
  line box, with typo-metrics faces putting the gap above the text; GDI
  external leading above the first baseline; glyphs laid out at the authored
  size and painted at the device size; hidden-mark paragraphs, cells and
  rows take no line; letter spacing in headers/footers and character spacing
  in the wrap measure; a missing font paints in its installed `altName`.
- **Tabs, lists and indents.** A hanging indent is an implicit tab stop; a
  list marker lifts only its own line; numbering-level `pPr`/`jc` and
  `lvlOverride` apply; the label's tab lands on the hanging indent, not a
  later right stop; a typed lone symbol does not hang like a bullet; `w:cr`
  ends the line and a line feed inside `w:t` is a space.
- **Spacing and pagination.** Space before collapses at page and column
  tops the way the break leaves it; a page break inside a paragraph moves
  the rest to the next page; column-break marks take a line; a section break
  takes the type of the section it starts; odd/even parity survives a
  numbering restart; a page nothing was placed on never breaks; a tracked
  section change is not a section break.
- **Package layer.** Relationship targets are XML-unescaped before
  resolving, so part names containing `&amp;` resolve to the right part.
- **PDF output.** Horizontally scaled runs squeeze their glyphs; every
  Identity-H font carries `/ToUnicode` (see Added).
- **Harness.** Six tests added since `fb241e2` were missing `#[test]` and
  never ran — restored. The 76/398 sweep and the 50-row smoke now convert
  with `--revisions word`, matching the markup the gate scores.

### Performance

- Font lookups and shaped words are memoised, PDF content ops are written in
  place, shape plans are reused, and faces load only for the families a run
  paints.
- Embedded TrueType faces are subsetted (glyph ids kept) and font programs
  and image samples always deflate — `--compress` remains about the page
  content streams.

## [0.9.1] - 2026-09-22

### Added

- **`word/fontTable.xml` altName.** Unknown requested families now follow
  Word's recorded `w:altName` (installed faces still win). Calibri stays the
  Carlito catalogue slot so an altName on Calibri cannot steal Cambria.
- **Quoted CSS family lists are not Times.** Word splits `w:ascii` on comma
  but does not CSS-unquote, so `"Times New Roman", Times, serif` is unknown
  (Quartz used Cambria) while `Verdana, Geneva, sans-serif` is Verdana.
- **Theme minor slot for every face.** `minorHAnsi` → theme minor (Cambria
  included). The Aptos-only gate is gone. `file_2` / `file_41` may drop
  until the line-box formula lands; explicit `w:ascii` still wins.
- **Word-substitution evidence table.** Unknown / quoted CSS-list families
  resolve to Cambria (Word Quartz). `DejaVu Sans Mono` → Verdana;
  `Liberation Serif` → Times New Roman; empty request → Times New Roman;
  `w:family`/`w:pitch` generics (roman/swiss/modern/fixed) after altName.
- **`FaceKey` catalogue shim.** Physical family + bold/italic, with lazy
  `OnceLock` loads so a conversion does not parse all 47 faces up front.
- **Embedded `.odttf` fonts.** `w:embedRegular` (and bold/italic slots) are
  de-obfuscated with the `w:fontKey` GUID (ECMA-376-2 §11) and registered as
  extra faces for that conversion. Unknown families such as case8
  `Press Start 2P` no longer fall through to Cambria.

- **Convert fidelity gate.** `scripts/convert_sweep.py` (wrapper
  `scripts/convert-sweep.sh`) scores `jubarte convert` against Word PDFs on the
  76 docxide fixtures and the 398 corpus, path-referencing the sibling
  checkouts rather than vendoring 46 MB of cases. `planning/sample50_check.py`
  is the 50-row smoke after every engine change. Baselines from jubarte 0.9.0:
  76 mean Jaccard **13.74** (median 8.86), 398 mean Jaccard **53.10**
  (median 43.50). The gate itself does not change converter output.

### Fixed

- **A space-only run keeps its space.** `birds.` + `<w:t xml:space="preserve"> </w:t>`
  + `We` painted "birds.We": whitespace collapsing squeezed a lone-space run to
  nothing, and the pretty-print filter treated whitespace inside `w:t` as
  XML indentation. Body and header/footer text both keep the gap now.
- **CI on GitHub Actions.** `cargo fmt` wraps two `finalize.rs` lines; tests
  that `std::fs::read` sibling `../neurotic_docx_bench` fixtures now skip when
  that checkout is absent (Actions has no sibling). `deny.toml` ignores
  RUSTSEC-2026-0206 (`rustybuzz` unmaintained) and RUSTSEC-2026-0192
  (`ttf-parser` via rustybuzz) until the FaceKey/harfbuzz swap (plan 2e).
  Clippy 1.98 `chunks_exact_to_as_chunks` on the WMF gray-fill helper.
  Convert tests that look for `/Calibri` in the PDF also accept bundled
  `/Carlito` (Actions has no Word DFonts). Aptos/Calibri wrap-width oracles
  skip when Word DFonts are absent (PDF still names the logical face).
  Font overlay no longer scans `/System/Library/Fonts` (Apple Symbol ≠ Word
  SymbolMT; GitHub macOS runners have it).

## [0.9.0] - 2026-09-05

Ring 3 is green for the first time: all 207 corpus redlines open in Microsoft
Word with no warning, error or repair offer (was 202/5).

### Fixed

- **`w:instrText` under `w:del` is now `w:delInstrText`.** This was the cause of
  every one of the five Ring 3 open failures. Word wants `w:delInstrText` in a
  deletion for the same reason it wants `w:delText` instead of `w:t`, and offers
  to repair the file when it does not get it. The shape came from deleting
  content containing a field — a wholly deleted header or footer carrying
  `PAGE`/`NUMPAGES` is the usual case — on a path that wraps existing runs in
  `w:del` rather than rebuilding them. Only 5 of the 207 corpus outputs carried
  it, and they were exactly the 5 Word rejected.

  **The OpenXmlValidator reported all five as clean.** `w:instrText` is
  schema-valid in a run whatever the run's parent is; the del/delInstrText
  correspondence is a semantic rule Word applies at load. A green Ring 2 does
  not mean Word will open the file.

- **`w:del` no longer swallows `w:hyperlink`.** `w:hyperlink` is not in
  CT_RunTrackChange's content model, so deleting a whole header/footer produced
  markup Word rejects. Word's shape is the inverse — the hyperlink stays put and
  the revision moves inside it — and the revision is now split around each
  hyperlink, preserving document order and minting fresh `w:id`s.

- **No more bare `<w:sz/>` / `<w:szCs/>` in the merged Normal style.** The
  style-merge loop created the element and then passed `None`, which strips
  `w:val` but leaves the element behind. `w:val` is required on CT_HpsMeasure;
  ECMA-376 spells "no value" as the element's absence.

- **Invalidity inherited from a source is repaired instead of shipped.**
  Unqualified attributes on `w:` elements (`paragraphProperties="[object
  Object]"` — wml.xsd is `attributeFormDefault="qualified"`, so these are invalid
  by construction), `w:shd` with no `w:val` (supplied as `clear`, keeping the
  author's fill), and `w:highlight` under `w:lvl/w:rPr`. Deletion-or-default
  only, so a document that was already valid cannot change. Worth stating
  plainly: **all these sources open fine in Word** — it tolerates their
  invalidity and rejected our output, so this was never the reason those
  documents failed. It is repaired anyway, because a source's corruption
  shipped inside our redline gets blamed on us.

### Added

- `finalize::enforce_deleted_text_kinds`, `finalize::hoist_hyperlinks_out_of_revisions`
  and `finalize::repair_inherited_invalidity`, each run both at the end of the
  body pipeline and in the package-level validity sweep — headers and footers
  deleted wholesale never reach the body finalize path, which is why the three
  defects above survived there.
- `tools/validate-docx/` is **committed** for the first time. Ring 2 could not
  run in a fresh checkout: the C# project existed only in a sibling worktree and
  was on no remote. It now also reports the part URI and XPath of each finding,
  so a Ring 2 result names a location instead of only a rule. The ratchet keys
  on the first two columns and is unaffected.

### Changed

- `tools/validity_baseline.tsv` re-blessed: 1183 findings / 46 pairs / 60 keys,
  from 1294 / 54 / 74. The previous bless keyed stems as `<stem>.ours`, which
  `scripts/redline-sweep.sh` never emits, so the ratchet compared two disjoint
  key sets and reported 74 regressions plus 75 fixes on an unchanged corpus.

### Known

- **Ring 1 is red and was already red before this release**: `parity_ladder.py
  sweep` reports 41 NEW findings (4 at L0) against the checked-in baseline on an
  unmodified tree. This release introduces none of them — the identical 41 appear
  when the same sweep runs against a binary built without these changes — and
  fixes one. They are not blessed away, because L0 is the losslessness contract
  and hiding those rows is the one thing the ladder exists to prevent.
- The `w:Unid` leak (KNOWN_ISSUES 4) is narrowed but still open: 12 of 207
  outputs, 0 of 199 sources.

## [0.8.0] - 2026-09-05

Two losslessness fixes in the redline core, and the DOCX → PDF converter that
0.7.1 prepared but never shipped to crates.io.

### Fixed

- **Redline no longer drops sentence-final punctuation (M463).** The
  fold that attached a trailing bare `.` EQ run onto the preceding
  `w:ins`/`w:del` moved a run that belongs to *both* documents into a
  one-sided revision and then deleted it, so the period left the other
  side's stream — onto a `w:del` it vanished from the modified document,
  onto a `w:ins` from the original. 16 corpus pairs stopped reconstructing
  their own inputs. The whole `fold_boiler_eq_between_ins` pass is removed:
  keeping only its `INS|EQ|INS` half measured worse (19 vs 18 L0 rows).
- **Redline no longer rewrites letter case (M328d).** The free-mesh word
  rehash hashed `text.to_ascii_lowercase()`, so `"Green"` and `"green"`
  compared Equal — but the emitted EQ run carries only one side's casing,
  turning `Green highlights…` into `green highlights…`. The extra spurious
  matches also let two output paragraphs claim the same source atom. The
  hash is case-sensitive again at all 20 call sites; M328d's unrelated
  stamped-pair (`file_N.docx`) free-mesh exclusion is kept. Case-insensitive
  matching is sound only once the emitted run keeps each side's own text.

Ring 1 (`tools/parity_ladder.py sweep`, 207 pairs): L0 reconstruction
failures 34 → 17 rows; regressions against `042089c` 19 → 2. The two
survivors are a distinct free-mesh double-consumption defect — see
KNOWN_ISSUES.md issue 3. 15 of the 17 predate `042089c`.

### Added

- **`jubarte convert` / `convert::docx_to_pdf`.** Emit a real multi-page PDF
  from DOCX bytes without LibreOffice: paragraphs, lists, tables, JPEG/PNG,
  headers/footers, and WMF/EMF rasterization. Prepared in 0.7.1, which was
  tagged but never published — 0.7.0 is the newest version on crates.io, so
  this is the first release in which `convert` is installable.
- **`convert::PdfOptions { compress }`** (CLI: `jubarte convert --compress`)
  — `/FlateDecode` the content streams. Off keeps page content greppable
  with `strings`; on is substantially smaller.
- Converter fidelity work against Word: OMML `m:f` `noBar` fractions, tab
  stop resolution (`w:tabs`, `defaultTabStop`, ISO-Strict `ST_TabJc`),
  Word's `size >= w:kern/@val / 2` kerning threshold, `GridTable4-Accent5`
  and `MediumShading` table styles, DrawingML stroke width/colour from
  `a:ln/@w` and `lnRef`, textbox `w:ind`/`w:spacing` and
  `relativeFrom=margin`, `w14:reflection` flattening as Word's PDF export
  does, and small-caps shrinking lowercase only (ECMA-376 17.3.2.5).

## [0.7.1] - 2026-08-16

Independent DOCX → PDF converter. Redline output is unchanged from 0.7.0.

### Added

- **`jubarte convert` / `convert::docx_to_pdf`.** Emit a real multi-page PDF
  from DOCX bytes without LibreOffice: paragraphs, lists, tables, JPEG/PNG,
  headers/footers, and WMF/EMF rasterization. Layout aims at soffice visual
  parity — Carlito/Liberation (the metric-compatible faces LibreOffice
  embeds), rustybuzz shaping, `sectPr` page geometry, and named-style
  resolution.

### Fixed

- Clippy 1.97 `question_mark` on the theme `lastClr` fallback.
- rustfmt across comparer / `document_comparer`.

## [0.7.0] - 2026-08-13

**Jubarte now wins on speed as well as quality.** 0.6.0 already led every
fidelity metric on the 763-document `script_redlines` benchmark; 0.7.0 closes
the last gap to the C# incumbent on generation time. Measured interleaved
(both engines on the same pair back-to-back, so concurrent machine load hits
both equally) over the 4880 pairs both engines complete:

| speed measure | jubarte 0.7.0 | docxodus 9.0.0 |
|---|---|---|
| median / doc | **5.3 ms** | 7.2 ms |
| mean / doc | **22.2 ms** | 24.1 ms |
| p95 / doc | **94.8 ms** | 96.2 ms |
| p99 / doc | **139.7 ms** | 179.9 ms |
| throughput | **45.0/s** | 41.4/s |
| generation failures | **0** | 120 |

Jubarte leads all six. Every performance change below is output-identical to
0.6.0 (verified by LibreOffice render parity, XML c14n equivalence, and LCS
fuzz/collision tests) — no fidelity was traded for speed.

### Performance

- **Killed the superlinear tail** in the word-level relatedness detector: the
  worst-case pair dropped 837 → 678 ms with no output change.
- **Detector fast-path + by-reference descendants walk.** `detect_unrelated_
  sources_word_mode` now short-circuits the full-document word-LCS when all
  keep-LCS cases are provably impossible (O(n+m) rolling-hash pre-check), and
  `Dom::for_each_descendant_element` compares element names by reference
  instead of cloning an `XName` (2 Arc bumps) per element across the ~84
  finalize passes. Together these flipped mean and throughput to jubarte.
- **Poststep re-parse elimination.** The Word-validity poststeps parsed
  `styles.xml` four times per compare; they now cache the defined-style-id set
  from the styles-copy pass and reuse a single styles arena in the M-PAG
  Normal-merge. This closed the p95 gap (6/6). A 128-bit FNV-1a fingerprint
  (`sha1_key128`) replaces the per-step 40-byte hex compare in the LCS extend
  step (~2⁻¹²⁸ collision), and the LCS bucket index uses an identity hasher on
  its already-hashed u64 keys.

### Fixed

- **Mesh & revision ordering** — M468 (yields to the M322 head-junction; no
  fold across trailing empty pure-I separators), M469 (splits a short inserted
  title MIX from a long unrelated deletion), M471 (rotates the impossible
  ins-mark del-only paragraph), M472 (re-asserts ins-before-del order after a
  comment carry), M473/M474 (restamps a stranded deletion mark; field-residue
  gate), M491 (B's document-final paragraph mark never inserts mid-document).
- **Spacing** — M487 bakes B's effective paragraph spacing onto inserted
  paragraphs, gated to B-implicit values only and never onto empty or
  declared-value paragraphs; M492 keeps deleted paragraphs' A-original direct
  spacing; M479 lets spacing `before` join the Normal merge under the B-chain
  gate.
- **Styles** — M476 gates the S2 copied-style bake on ascii font-family change;
  M477 adds a per-attribute B-chain bake gate and Word-complete Normal
  promotion; M478 materializes implicit `kern`/`ligatures` neutralizers; M480b
  adds docDefaults-delta disabling neutralizers on both-sides merged styles;
  M483 re-caches themed color hexes against the shipped theme.
- **Numbering** — M481/M482 repair the core relationship part and remap
  `numId` collisions; `w15:restartNumberingAfterBreak` is ignored in
  `abstractNum` identity.
- **Images** — M495 keeps an image-only paragraph M491 had misjudged as empty;
  M496 carries over the revised image on an inserted-reference `rId` collision.
- **Sections** — M494 emits no spurious `sectPrChange` for implicit-default
  section properties.
- **Fields** — M470 keeps Word's field form for deleted anchor hyperlinks.

## [0.6.0] - 2026-08-11

**Jubarte is now the best redline engine on the market**, leading every
headline metric on the 763-document `script_redlines` benchmark against the
Microsoft Word oracle (neurotic-docx-bench, LibreOffice 26.2.4.2 renderer):

| metric | jubarte 0.6.0 | docxodus 9.0.0 |
|---|---|---|
| mean fidelity | **83.27** | 80.55 |
| median fidelity | **91.67** | 91.19 |
| generation failures | **0** | 4 |
| documents ≥ 90 | **403** | 392 |
| generate time, median/doc | **20.7 ms** | 82.2 ms (4.0× slower) |
| generate time, mean/doc | **59.5 ms** | 601.4 ms (10.1× slower) |

### Fixed

- **M460** — the heading `line=240` stamp (M79) now fires only when the merged
  Normal is itself Word-normalized single-line 240; a Normal carrying B's
  non-240 line left headings line-less in the oracle (basic_comment ×
  cli_legacy +15).
- **M461** — the Normal rPr merge carries B's stored `w:kern` and
  `w14:ligatures`; dropping `kern=0` left kerning ON from A's docDefaults and
  narrowed every long paragraph one line short (basic_comment 50.5 → 97.7).
- **M462** — when A has no styles part, scaffold from Word's FACTORY
  docDefaults + Aptos theme and bake each copied B style's effective metrics,
  instead of adopting B's docDefaults wholesale (tiff_image pairs 38–40 →
  96–99.8).
- **M463** — inserted/deleted OMML math serializes Word-style: revision marks
  INSIDE `m:r` with a materialized Cambria Math rPr, `m:t` never delText,
  applied as a final pass so mesh reasoning is undisturbed (math family
  pairs +40, page-level pixel parity with the oracle).
- **M464** — the S2 copied-style bake covers pPr spacing deltas between the
  two docDefaults, not just run metrics (file_13 × file_14 class).
- **M465** — the mid-stream demo-title fold (M143) is gated off for anchored
  pairs (matched leading title): Word keeps A's deleted document intact at
  the end; also stops the M179 " Demo" EQ from stripping deletion marks
  (file_13 +22, file_145 +9).
- **M466** — the trailing bare-period attach skips runs carrying `rPrChange`;
  merging a format-changed period into delText dropped the tracked change
  (file_168 back to 100.00).
- **M467** — the merged Normal keeps only per-attribute deltas vs the output
  docDefaults, pruning kern/sz/szCs/rFonts/ligatures the context already
  supplies (tab_test × table_autofit; restored 34 exact-100 documents).

## [0.5.1] - 2026-07-24

### Fixed

- **CI was red and the test suite did not compile.** Two integration tests
  had drifted out of sync with the helpers they call: `m35_comments` passed
  `optional_bench_docx`'s `Option<Vec<u8>>` (file *bytes*) to a
  `require_path(&str)` guard, and `word_package_notes_settings_coherence`
  destructured an `Option` with a `Result` pattern after its loader moved to
  `.ok()`. The `m35_comments` guards were also redundant — the preceding
  `let Some(…) else { return }` already skips the missing-fixture case.
- `clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`
  both pass again: `items_after_test_module` in `comparer::preprocess` (the
  `escape_xml_tests` module now sits at end of file), plus accumulated lint
  drift in `examples/` — `const Z; [Z; N]` atomic-array splats replaced with
  inline `const {}` blocks, four descending `sort_by` comparators expressed as
  `sort_by_key(Reverse)`, and one `Default` field reassignment folded into
  struct-update syntax.
- Residual Word-visual peels on the finalize path (M216–M233): empty pure-D
  folds, MIX Heading/spacing/numPr parks (gated), mid pure-D live spacing
  promotion, schema-default `jc` left/start strip, jc-only `pPrChange` removal.
  Full main ledger **mean 90.04 / median 95.67** (n=164) at `d094de0`.

### Continuous integration

- Coverage is now measured and published. A `coverage` job runs
  `cargo llvm-cov --all-features --workspace --lcov` and uploads to Codecov via
  `codecov/codecov-action@v7`; [`codecov.yml`](codecov.yml) sets an `auto`
  project target with a 1% tolerance and an 80% patch target, and excludes
  `tests/`, `benches/`, `examples/`, `tools/`, `parity/` and `scripts/` from the
  denominator. The README's Codecov badge and coverage row have existed for
  some time but had never received an upload.

### Performance

- M232/M233: single-pass spacing+jc cleanup and lazy pure-del/mixed paragraph
  classification cache for multi-pass peels.

### Documentation

- `docs/BENCHMARK_M233.md` — full quality + speed stamp (main, randomized
  `file_i_v_file_{i+1}`, 5k-pair speed bench, criterion, expanded ABBA).
- `tools/perf/run_abba_matrix.sh` — optional sample expansion with consecutive
  file pairs (`FILE_SAMPLE=1`).

## [0.5.0] - 2026-07-15

Product line alignment with the desktop app: same **0.5.0** minor for the
shipped engine that powers the Mac App Store build. Includes everything from
0.2.0 (package-wide Word validity, notes/settings coherence, parity restore,
measured Q0 performance stack) plus release tooling (`VERSIONING.md`,
`scripts/bump-version.mjs`).

## [0.2.0] - 2026-07-15

### Fixed

- **Word package validity is package-wide**, not `document.xml` alone: strip
  PowerTools `pt:*` markup across OPC parts and re-sync settings after the
  validity sweep so Microsoft Word does not report unreadable content.
- **Notes / settings coherence:** keep structural note types
  (`continuationNotice` id=1, etc.), renumber user notes around reserved ids,
  and ensure `settings.xml` footnotePr/endnotePr special-note ids ⊆ the notes
  parts (Word opens the full OPC package).
- **Parity restore after ATOM-STACK / IDENTICAL-INPUT work:** footnote and
  endnote definitions stay on the atomize path stack so deleted-note produce
  no longer panics; identical-package short-circuit still runs drawing id
  fixups (`wp:docPr`) so pre-existing source collisions do not reappear as
  `S-dup-docpr-id` on the ladder.

### Performance

- Large Q0 wall stack (measured; see `LCS_PERF_PLAN.md`): atomize path stack,
  serialize direct buffer writes, SHA-1 streaming digests, simple-p/tc hash
  without clone DOM, accept clean-subtree reuse, accept skip when transforms
  cannot fire (rsid, empty cells, fields, A.3 move ranges, A.5 deleted marks,
  …), OnceLock `XName` caches (NAME-01 / 01b / 01c).
- Banked experiments kept as exact cleanup where full permanent ABBA matrix
  did not win every load-bearing slot (ACCEPT-SKIP-A3/A5, NAME-01c, …).

### Added

- `VERSIONING.md` + `scripts/bump-version.mjs` for one-shot Cargo version
  codemod and neurotic binary install steps.
- Focused perf exact tests under `tests/perf_*.rs` for the Q0 gates above.

### Quality

- Parity ladder re-blessed to zero NEW keys after the notes/stack/docPr fixes.
- Full neurotic visual ledger class retained (historical floor ~83.8 mean /
  ~88.5 median on script_redlines sample/full runs during the stack).

## [0.1.0] - 2026-07-12

### Added

- Initial release, extracted from the `ooxmlsdk-redline` development crate.
- `document_comparer::compare_documents` (+ `_with_options`,
  `_with_settings`): compare two `.docx` documents into a tracked-changes
  (redline) `.docx`.
- `document_comparer::get_revisions`: list tracked revisions (type, author,
  date, part, move group, format-change details, text).
- `document_comparer::accept_revisions` / `reject_revisions`: flatten a
  redline package-wide.
- `comparer::WmlComparerSettings`: author/date stamping, detail threshold,
  Word-visual alignment passes (default) or the PowerTools-faithful preset.
- `jubarte` CLI (default `cli` feature): plain compare plus `revisions` and
  `accept` subcommands.

### Fixed

- External hyperlinks no longer lose their targets in the default
  (Word-visual) mode: `unwrap_hyperlinks_to_styled_runs` now preserves
  `r:id`-bearing `w:hyperlink` wrappers and unwraps only anchor-based
  internal (TOC) hyperlinks, so relationship reconciliation keeps the
  hyperlink relationship (with `TargetMode="External"`) in the output.

### Known issues

- See [KNOWN_ISSUES.md](KNOWN_ISSUES.md); the covering tests are marked
  `#[ignore]` with matching reasons.

[0.16.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.16.0
[0.15.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.15.0
[0.14.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.14.0
[0.13.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.13.0
[0.12.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.12.0
[0.11.3]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.11.3
[0.11.2]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.11.2
[0.11.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.11.0
[0.10.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.10.1
[0.10.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.10.0
[0.9.3]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.3
[0.9.2]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.2
[0.9.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.1
[0.9.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.0
[0.8.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.8.0
[0.7.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.7.1
[0.7.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.7.0
[0.6.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.6.0
[0.5.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.5.1
[0.5.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.5.0
[0.2.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.2.0
[0.1.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.1.0
