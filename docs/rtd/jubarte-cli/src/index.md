<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-cli

The jubarte engine's command line as a library: every operation of the
engine behind one clap surface (compare, convert, changes, accept, reject,
edit, add, read, diff, validate, check, scrub, append, comments, fields,
capabilities, visual-diff, debug) plus `self-update` behind its feature.

- `cli` holds the parser (`cli::top_level_command`) and the shared pieces
  of dispatch: output formats, the lines the commands print, input refusals.
- `inline` compiles the typed inline edit actions (`jubarte edit -p ...
  --anchor ...`) into plans; the shared engine does the mutation.
- `update` is `jubarte self-update` (feature `self-update`).

The `jubarte` binary itself is built by the
[`jubarte`](https://jubarte.readthedocs.io/en/latest/) crate on top of this
one; `cargo install jubarte` installs it. The full command reference is in
the [jubarte book](https://jubarte.readthedocs.io/en/latest/cli.html).

`build.rs` stamps `--version` with the source commit when the crate is
built from the jubarte repository, and with nothing when it is built from an
unpacked package inside another repository.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-cli` to embed the jubarte command line in another
program, or to read its clap model (to generate documentation or shell
completions). To run the commands, install the binary instead.
