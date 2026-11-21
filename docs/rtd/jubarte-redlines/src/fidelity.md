<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# What the comparer normalizes

The redline is Word-valid, and differences in text and formatting are
revisions. To open cleanly and to match what Word's own Compare writes, the
comparer also normalizes some markup. The main cases:

- Both inputs lose non-standard `w:`-namespace children of `w:sdtPr`, and
  have `mc:AlternateContent` resolved to a single branch.
- ISO Strict inputs are rewritten to Transitional first.
- A breaking and a non-breaking space compare equal
  (`conflate_breaking_and_nonbreaking_spaces`, on by default in both
  presets).

In the default Word mode only:

- the original's own tracked changes are flattened and compared like text;
- internal anchor `w:hyperlink` wrappers (no `r:id`) become runs styled as
  hyperlinks;
- content controls in a paragraph that carries a revision are unwrapped;
- some redundant default spacing is stripped.

`WmlComparerSettings::powertools_faithful()` skips the Word-mode passes.
`docs/WORD_DIFFERENCES.md` in the repository lists where the redline still
differs from Word's, which side is better, and the mode that picks each
behaviour.
