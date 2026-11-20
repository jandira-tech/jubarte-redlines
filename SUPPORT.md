<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Getting help

| You have | Go to |
|---|---|
| A question about using jubarte | [GitHub Discussions](https://github.com/jandira-tech/jubarte-redlines/discussions) |
| A bug, or output that differs from Microsoft Word | [A new issue](https://github.com/jandira-tech/jubarte-redlines/issues/new/choose) |
| A feature request | [A new issue](https://github.com/jandira-tech/jubarte-redlines/issues/new/choose) |
| A vulnerability | The private form described in [SECURITY.md](SECURITY.md); never a public issue |

Before opening an issue:

- Search the open and closed issues, and read
  [KNOWN_ISSUES.md](KNOWN_ISSUES.md) and
  [docs/WORD_DIFFERENCES.md](docs/WORD_DIFFERENCES.md): a difference from
  Word may be a recorded choice.
- Run `jubarte --version` and upgrade if a newer release exists. Fixes go
  into the latest release only.
- Reduce the document to the smallest one that still shows the problem, and
  attach only documents you may publish. Remove personal information first:
  `jubarte scrub` removes authors and metadata.

The guides are in [docs/](docs/README.md): the command line and Rust API
([docs/rust.md](docs/rust.md)), Python ([docs/python.md](docs/python.md)) and
JavaScript ([docs/javascript.md](docs/javascript.md)).

There is no paid support channel and no response-time guarantee. Maintainers
aim to answer a new issue within three working days.
