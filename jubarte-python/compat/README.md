<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-redlines (renamed to jubarte-docx)

jubarte's Python package is now [`jubarte-docx`](https://pypi.org/project/jubarte-docx/):

```sh
pip install jubarte-docx          # import jubarte; the `jubarte` command
uvx --from jubarte-docx jubarte redline a.docx b.docx -o redline.docx
```

This distribution installs `jubarte-docx` and keeps the old names working:
`import jubarte_redlines` (the same objects as `import jubarte`) and the
`jubarte-redlines` command. New code should import `jubarte`.

PyPI's `jubarte` is a different project; `pip install jubarte` does not
install this engine.
