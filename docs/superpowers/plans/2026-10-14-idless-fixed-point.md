<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# Plan: Markdown without id lines is a fixed point (2026-10-14)

> **Status (reconciled 2026-10-17, `main` `aba48bd5`): largely landed.** The
> squares fix shipped — no comment note icon unless `--comment-notes` is passed
> (`f8932ae1`; opt-in rule in AGENTS.md). L1/L2/G3 (list indents, legend indent
> + `contextual spacing` + keep-lines, inherit-don't-write-out) landed in
> `b65ae450` (`view_styles.rs` now parses `hanging`/`contextual spacing`/`indent`;
> `md_reader_id_lines.rs::an_idless_file_keeps_list_indent_keep_lines_and_a_bare_normal`).
> W1 (run spacing after breaks) landed on `fix/md-run-spaces` (`214888bb`).
> E2 (empty emphasis), E3 (math spacing) and literal-delimiter escaping landed
> in `92001276`. K1 (paren-URL links print `<URL>`), K2 (mailto autolink text)
> and K3 (the lost code span) landed in `762c21be`. E1's code-block CriticMarkup
> escaping was already `e51092ce`. The idless A→B→C→D chain tests live in
> `tests/md_reader_id_lines.rs` (`idless_chain`, `idless_views`) and
> `tests/md_plain_round_trip.rs`. **Open:** the full 75-file G1+G2 gate with
> `visual-diff` page equality (item 6) is not a suite test — the landed gates
> are per-fixture chain assertions, not the 75/75 sweep; L3's nested-bullet
> glyph cycle and K4's re-measure are unverified remnants.

Arthur's chain, measured on the 75 test Markdown files that carry no id line:

    A (Markdown, no id lines) --convert--> B (docx)
    B --read--> C (Markdown view, ids)
    C --convert--> D (docx)

| Gate | Today | Goal |
|---|---|---|
| G1. the view: C == read(D), less `sha256:` | 73 / 75 (not dolphins, eLife.00047) | 75 / 75 |
| G2. the pages: `jubarte visual-diff B D` | 55 / 75 identical (eLife 20 → 22 pages) | 75 / 75 |
| G3. D's styles inherit what B's inherit | D writes every style's font, size, bold and italic out | D writes what differs from `basedOn` and the defaults |

Byte equality of B and D is not a goal: it fails on serialization alone
(`/>` vs ` />`, split runs). G1 + G2 are the gate.

Evidence: `jubarte-loop/idless_report.txt` (G1),
`jubarte-loop/idless/vd/<i>/diff.json` (G2),
`jubarte-loop/idless_side_by_side_jubarte.pdf` (the 128 pages side by side,
made by jubarte alone). The per-page text came from `convert --report`.

## Causes of the 20 differing pairs

| # | Cause | Files | Where it goes wrong |
|---|---|---|---|
| L1 | A bullet or numbered item loses its indent: D's level has no `w:ind` | hard-lists ×3 (the best representatives), structure ×3, all ×3 | writer: `label_numbering` (xml.rs) for `<!-- pN … bullet -->` id lines |
| L2 | `ListParagraph` loses `ind left 720` and `contextualSpacing`, so items gain 8pt after each | the same files | view: the style legend says `ListParagraph: Calibri 11pt` only |
| L3 | A nested bullet's glyph: `◦` becomes `•` | legacy/services | writer: label numbering uses one glyph for every level |
| W1 | A line that starts with a space after a line break loses the space | additions, deletions, hard-adjacent, structure.critic, all.critic, breaks | view or writer: leading whitespace after `w:br` |
| E1 | CriticMarkup delimiters written as text (`++`, `--`) come back changed | hard-delimiters ×2 | view escaping |
| E2 | Empty emphasis leaks as text: `delphis)` becomes `delphis****)` | dolphins | view: an empty bold/italic run prints `****` |
| E3 | Math spacing: `=  𝜆` becomes `= 𝜆` | dolphins | view or writer of OMML text runs |
| K1 | A link whose URL holds `(…)` comes back as literal `[CrossRef](…)` text | eLife | view: the URL is not escaped for Markdown |
| K2 | A mailto link's text gains `mailto:` | git_manuscript | view: autolink form |
| K3 | One `Verbatim Char` code span lost (31 → 30) | math/SOURCES | to diagnose |
| K4 | A hyphenated word split (`3-containing` → `3-` `containing`) and 2 more pages | eLife | follows K1 (longer text) — re-measure after K1 |

G1's two misses (dolphins, eLife) are E2, E3, K1: the view itself does not
come back.

## Arthur's items

1. G1 75/75 — fixes W1, E1–E3, K1–K3.
2. G2 75/75 — the above plus L1–L3.
3. The style legend describes paragraph indents and contextual spacing
   (`ListParagraph: Calibri 11pt, indent 0.5in, contextual spacing`), and
   the writer applies them (L2). Same for keep-lines (Heading1 lost it).
4. Styles inherit instead of writing out (G3): the writer writes a style's
   run and paragraph properties only where they differ from what `basedOn`
   and `docDefaults` already give. D's Normal then matches B's (empty).
5. Bullet and numbered id lines get Word's standard indents for their level:
   `left = 720 × (level + 1)`, `hanging = 360`, and Word's glyph cycle
   `•`, `o`, `▪` (L1, L3). Word-probe the result (one Word job machine-wide).
6. The gate becomes a test: for every id-less test Markdown file,
   C == read(D) AND `visual-diff B D` reports no changed page. It runs in
   the suite once 1–5 land; until then it lists the files that still fail.

## Order (one PR each, red test first)

1. Comment-reference squares (below): fix now.
2. L1 + L3 (indents and glyphs per level), hard-lists as fixtures.
3. L2 + item 3 (legend describes indent, contextual spacing, keep-lines).
4. G3 / item 4 (inherit, don't write out).
5. W1 (leading space after a break).
6. E1, E2, E3 (escaping and math spacing).
7. K1, K2, K3 (links, mailto, code span); re-measure K4.
8. The gate test (item 6), required: 75/75 on G1 and G2.

## Found on the way: squares at comment references — landed

jubarte's PDF draws a yellow square at each comment reference, over the
next letters (`the|m`, `en|d`), in every revision style, Word's included.
Word draws nothing there in a PDF. Fix: draw no box at a comment
reference; the range tint and the balloon stay.
[Shipped as `f8932ae1`: no PDF comment note icon at all unless the opt-in
`--comment-notes` / `PdfOptions::comment_notes` is passed.]
