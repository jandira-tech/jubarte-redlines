<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Revision styles

`PdfOptions::revisions` (`RevisionStyle`; the CLI's `--revisions`) picks
how tracked changes are drawn:

- `RevisionStyle::Conventional` (the default): red strike for deletions,
  blue underline for insertions, and moves in green, double-struck where the
  text left and double-underlined where it landed. Every revised run is
  marked.
- `RevisionStyle::Word`: what Word's own Save as PDF paints, per-author
  colours included, and Word's mistakes too.
- `RevisionStyle::Custom(palette)`: your own marks.
  `RevisionPalette::parse("deleted=#AA0000:strike,inserted=#0055FF:double-underline")`
  starts from the conventional palette and changes the kinds you name
  (`deleted`, `inserted`, `moved-from`, `moved-to`); the lines are
  `strike`, `double-strike`, `underline`, `double-underline` and
  `plain`. `RevisionStyle::from_choice` parses the names the CLI and the
  bindings take.

## Which one to use

**Whenever a jubarte PDF is compared with one Word made** (a benchmark
score, a side-by-side check, a test that quotes a Word PDF), render it with
`RevisionStyle::Word`. A PDF in the default style measured against Word's
measures the wrong renderer.

Word's style copies Word even where Word is wrong. A Word behaviour that
makes the output worse, such as a revised page number left unmarked or a
comment Word draws no balloon for, is copied only under
`RevisionStyle::Word`, and recorded in `docs/WORD_DIFFERENCES.md` in the
repository. The other styles keep the sensible output, so a Word quirk never
leaks into the default. Where the Word style is worse than the convention,
the difference is listed there, and both are available: pick per document.

The modes, as `docs/WORD_DIFFERENCES.md` lists them:

{{#include ../../../WORD_DIFFERENCES.md:modes}}
