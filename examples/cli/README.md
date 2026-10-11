<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# The jubarte CLI, for agents

- [`help-cli.md`](help-cli.md): every `--help` screen, verbatim
  (regenerate with `gen-help.sh`).
- [`visualize/VISUALIZE.md`](visualize/VISUALIZE.md): how an agent looks at
  a document, at the result of editing a document that already carries
  tracked changes and comments, and at the differences between two
  documents, with every view's options. Each block is real output;
  `visualize/run.sh` regenerates the guide and everything in `visualize/out/`.

```bash
JUBARTE=target/release/jubarte bash examples/cli/gen-help.sh
JUBARTE=target/release/jubarte bash examples/cli/visualize/run.sh
```
