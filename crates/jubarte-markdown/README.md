<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte-markdown

Markdown in and out of Word documents for the [jubarte
engine](https://github.com/jandira-tech/jubarte-redlines):
`markdown_to_docx` writes CriticMarkup as tracked changes and comments,
the readers project a package to the agent view's Markdown, and
`diff_markdown` diffs two documents as pandiff-style CriticMarkup.
`legacy_doc` reads Word 97-2003 `.doc` files.

The `render` feature (off by default) adds the layout-dependent items:
`page_texts`, `with_page_markers` and page markers in reads.

License: AGPL-3.0-only (see [LICENSE](LICENSE)).
