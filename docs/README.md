<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Documentation

Start with the guide for the way you run jubarte. Each guide is complete on
its own.

## Guides

| Guide | For |
|---|---|
| [rust.md](rust.md) | The `jubarte` command line and the Rust crates |
| [python.md](python.md) | `jubarte-docx` on PyPI: the Python API, command and MCP server |
| [javascript.md](javascript.md) | `jubarte-wasm` on npm: Node, the browser, and `npx jubarte-redlines` |
| [adoption/](adoption/README.md) | Using jubarte from an agent: skills, MCP, and what other tools do differently |

## Reference

| Page | Covers |
|---|---|
| [flags_datetime.md](flags_datetime.md) | Every command-line flag |
| [locations.md](locations.md) | Ids and locations: how a paragraph, change or comment is named |
| [ID_CONTRACT.md](ID_CONTRACT.md) | The short and long id contract |
| [MARKDOWN.md](MARKDOWN.md) | Markdown and CriticMarkup, in and out |
| [DIALECT.md](DIALECT.md) | The Markdown dialect on one page |
| [IDS_AND_MARKDOWN_ROUNDTRIP.md](IDS_AND_MARKDOWN_ROUNDTRIP.md) | What the Markdown round trip guarantees |
| [TEXT_DIFF.md](TEXT_DIFF.md) | Reviewing document differences as text |
| [CAPABILITIES.md](CAPABILITIES.md) | Asking jubarte which edits it supports |
| [FILESYSTEM_PUBLICATION.md](FILESYSTEM_PUBLICATION.md) | How output files are written safely |
| [SELF_UPDATE.md](SELF_UPDATE.md) | `jubarte self-update` and release signatures |
| [bomb_prevention.md](bomb_prevention.md) | What jubarte does with a hostile package |
| [api/](api/README.md) | The public API listing of each release |

## Microsoft Word parity

Word is the reference for every redline and every PDF.

| Page | Covers |
|---|---|
| [WORD_DIFFERENCES.md](WORD_DIFFERENCES.md) | Where jubarte's redline differs from Word's, and which side each mode picks |
| [WORD_COMPARE_RULES.md](WORD_COMPARE_RULES.md) | Word's Compare rules, as reconstructed |
| [WORD_LAYOUT_RULES.md](WORD_LAYOUT_RULES.md) | Word's layout rules, as reconstructed |
| [WORD_COMMENT_BALLOONS.md](WORD_COMMENT_BALLOONS.md) | When Word draws no comment balloon |
| [C4_preexisting_revisions_decision.md](C4_preexisting_revisions_decision.md) | An open decision: documents that already hold tracked changes |

## For maintainers

| Page | Covers |
|---|---|
| [maintainers/MAINTENANCE.md](maintainers/MAINTENANCE.md) | How the repository is kept tidy, and the backlog |
| [VERSIONING.md](VERSIONING.md) | Versioning and the release process |
| [OPENSSF.md](OPENSSF.md), [OPENSSF_VERIFICATION.md](OPENSSF_VERIFICATION.md) | OpenSSF best-practice evidence |
| [TUNING_AUDIT.md](TUNING_AUDIT.md) | The renderer's tuning constants |
| [bench_classes.md](bench_classes.md) | Benchmark defect classes |
| [CSHARP_MARKDOWN_PROJECTION_MAPPING.md](CSHARP_MARKDOWN_PROJECTION_MAPPING.md) | The C# Markdown projection mapped to Rust |
| [design/cli-v2/](design/cli-v2/README.md) | The command-line redesign and its evidence |
| [rtd/](rtd/README.md) | The ReadTheDocs projects, one per crate |

Contributing starts at [CONTRIBUTING.md](../CONTRIBUTING.md); agents read
[AGENTS.md](../AGENTS.md).
