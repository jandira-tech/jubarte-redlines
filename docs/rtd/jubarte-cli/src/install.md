<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Install and features

```sh
cargo install jubarte          # the binary
cargo add jubarte-cli          # the library
```

The library is `jubarte_cli`. It has no default features.

| Feature | Adds |
|---|---|
| `render` | the renderer, forwarded to jubarte-doc, jubarte-markdown and jubarte-edit: the `fields` and `jubarte_render` re-exports, `--revision-palette` checking, and `debug --check render` (without it, that check falls back to `orphans`). The `jubarte` binary always builds with it |
| `self-update` | `jubarte self-update` and the `update` module, with the release download and signature check; without it the library has no network code |

The `jubarte` crate's `cli` feature turns on `render` here, and its
`self-update` feature turns on `self-update`.
