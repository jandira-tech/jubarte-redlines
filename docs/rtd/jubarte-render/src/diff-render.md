<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Page-by-page differences

`diff_render(a, b, &DiffOptions)` renders both documents once each at the
same DPI and with the same `PdfOptions`, and reports the pages that differ
pixel for pixel (`RenderDiff`, one `PageDiff` per page;
`RenderDiff::differs` says whether any page changed).

From the command line, `jubarte visual-diff A.docx B.docx` does the same
and exits 5 when pages differ.
