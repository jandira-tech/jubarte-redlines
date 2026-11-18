<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# The command model

`cli::top_level_command()` returns the clap `Command` the binary parses
with: every task, its flags, defaults, aliases and conflicts. Walking it
lists the tasks; parsing an argument list with it checks flags exactly as
the binary does, without running anything.

```rust,ignore
{{#include ../../../../crates/jubarte-cli/examples/commands.rs:example}}
```

Run it with `cargo run -p jubarte-cli --example commands`. It prints each
visible task with its one-line description, then `parsed task:
Some("compare")`.

The Python wheel's and the npm package's command lines share this parser
for help, defaults, aliases and usage errors, so a flag means the same thing
on every surface. The repository's `examples/cli_flags.rs` reads the same
model to list every flag, hidden ones included, in `docs/flags_datetime.md`.
