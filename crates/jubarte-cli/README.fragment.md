<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

The command-line layer of the [jubarte]({{repository}}) DOCX engine: the
clap declarations of every `jubarte` command (compare, changes, accept,
reject, convert, diff, read, edit, add, comments, append, validate, check,
scrub, visual-diff, capabilities, debug, self-update), their argument
parsing, the typed inline edit actions, and the `self-update` installer.

This crate is a library. It has no binary: the `jubarte` executable is built
by the facade crate. To install the command line, run
`cargo install jubarte`, or see the
[install options](../../README.md#cli) for prebuilt binaries.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-cli` sits on every other jubarte crate and is the top layer below
the facade `jubarte`, whose binary drives it.

## When to use this crate

Depend on `jubarte-cli` to embed jubarte's command grammar in another tool:
parse the same arguments, render the same help, or build edit plans from the
same flags. To call the engine from Rust, depend on the facade crate [`jubarte`](https://crates.io/crates/jubarte) or a product
crate instead.

## Install

```sh
cargo add jubarte-cli@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-cli = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default (the facade's binary turns on both).

| Feature | What it adds |
|---|---|
| `render` | The layout-dependent surface through `jubarte-render` (and the `render` features of `jubarte-doc`, `jubarte-markdown` and `jubarte-edit`): the `fields` and `jubarte_render` re-exports and parse-time checking of `--revision-palette` specs. |
| `self-update` | The `update` module behind `jubarte self-update`: finds a GitHub release, checks its SHA-256 and signature, and replaces the binary. Pulls in `self_update` and `ed25519-dalek`. |

## Example

```rust
use jubarte_cli::cli::top_level_command;

fn main() {
    // The full command tree needs more stack than the 1 MiB Windows gives
    // a main thread in a debug build; the `jubarte` binary also parses on
    // an 8 MiB thread.
    let parsed = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            top_level_command()
                .try_get_matches_from([
                    "jubarte", "compare", "old.docx", "new.docx", "-o", "redline.docx",
                ])
                .map(|m| m.subcommand_name().map(str::to_owned))
        })
        .expect("spawn the parsing thread")
        .join()
        .expect("the parsing thread does not panic");
    assert_eq!(parsed.expect("the arguments parse").as_deref(), Some("compare"));
}
```

## Modules

- `cli`: the command declarations (`Cli`, `Command`, `top_level_command`) and
  their parsing helpers.
- `inline`: compiles one typed inline action (`-p p12 --anchor …`) into an
  edit plan.
- `update`: `jubarte self-update` (`self-update` feature).
- Re-exported for the command implementations: `edit`, `markdown`,
  `capabilities`, `result`, `text_diff` and `publication` (`jubarte-edit`);
  `legacy_doc` (`jubarte-markdown`); `changes`, `comments`,
  `comment_settlement`, `validate`, `audit`, `scrub`, `append`, `fields`,
  `inspect` and `debug` (`jubarte-doc`); `document_comparer`,
  `revision_processor` and `wml_document` (`jubarte-redlines`);
  `jubarte_render` (`render` feature); `opc`, `admission`, `settings` and
  `strict_translation` (`jubarte-opc`); `xmllinq`, `namespaces`, `ids`,
  `unid`, `perf` and `util` (`jubarte-dom`).

## License

[AGPL-3.0-only](LICENSE). Review the license's requirements before you embed
the crate in a product or a network service.
