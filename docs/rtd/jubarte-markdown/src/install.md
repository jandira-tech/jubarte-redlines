<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Install and features

```sh
cargo add jubarte-markdown
```

The library is `jubarte_markdown`. It has no default features.

| Feature | Adds |
|---|---|
| `render` | the layout pass (jubarte-render, and jubarte-doc's `render`): `page_texts`, `with_page_markers`, and `<!-- page N of M -->` lines in reads. Without it those items do not exist |
| `test-support` | helpers the other crates' tests share; forwards jubarte-doc's `test-support`; no stability promise |
