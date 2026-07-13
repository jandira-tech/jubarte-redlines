# jubarte

[![crates.io](https://img.shields.io/crates/v/jubarte.svg)](https://crates.io/crates/jubarte)
[![docs.rs](https://docs.rs/jubarte/badge.svg)](https://docs.rs/jubarte)
[![CI](https://github.com/arthrod/jubarte-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/arthrod/jubarte-rs/actions/workflows/ci.yml)

Lossless DOCX redline engine for Rust.

`jubarte` compares two Word documents and produces a tracked-changes
(redline) `.docx`: the original document with every difference against the
modified one expressed as native Word revisions — insertions, deletions,
moves, and format changes — so the result opens cleanly in Microsoft Word.
It also lists, accepts, and rejects tracked revisions.

## CLI

```sh
cargo install jubarte
```

```text
jubarte contract.docx contract-rev2.docx
    → writes contract_v_contract-rev2.docx next to the original

jubarte -b old.docx -m new.docx -o redline.docx --author "Legal"
jubarte revisions redline.docx --json     # list tracked revisions
jubarte accept redline.docx -o final.docx # accept every revision
```

Run `jubarte --help` for the full surface (author/date stamping,
`--detail-threshold`, the PowerTools-faithful compatibility mode, …).

## Library

```sh
cargo add jubarte --no-default-features   # skip the CLI's clap dependency
```

```rust,no_run
let original = std::fs::read("original.docx").unwrap();
let modified = std::fs::read("modified.docx").unwrap();
let redline =
    jubarte::document_comparer::compare_documents(&original, &modified, "Reviewer")
        .expect("compare");
std::fs::write("original_v_modified.docx", &redline).unwrap();
```

For finer control build a `comparer::WmlComparerSettings` (author, revision
timestamp, detail threshold, paragraph-merge behavior) and call
`document_comparer::compare_documents_with_settings`. To inspect a redline
use `document_comparer::get_revisions`; to flatten one use
`document_comparer::accept_revisions` / `reject_revisions`.

### Feature flags

| feature | default | effect |
|---------|---------|--------|
| `cli`   | yes     | builds the `jubarte` binary (pulls in `clap`) |

Minimum supported Rust version: **1.88**.

## How it compares

The comparer is atom-based: both documents are decomposed into comparison
units (runs, paragraph marks, table cells, …), correlated with an LCS pass,
and the differences are re-expressed as Word revision markup on top of the
original package — preserving parts, relationships, headers/footers,
footnotes, and styles. The default mode adds Word-visual alignment passes on
top of the PowerTools algorithm; `--powertools-faithful` (or
`WmlComparerSettings::powertools_faithful()`) reproduces the classic
behavior instead.

## Known issues

Open engine defects and unresolved Word-behavior conflicts are tracked in
[KNOWN_ISSUES.md](KNOWN_ISSUES.md); their covering tests are marked
`#[ignore]` and run with `cargo test -- --ignored`.

## Provenance & attribution

The comparison engine is a Rust port of the `WmlComparer` /
`DocumentComparer` engine from [Docxodus](https://github.com/JSv4/Docxodus)
(MIT), itself a fork of Microsoft's
[Open-Xml-PowerTools](https://github.com/OfficeDev/Open-Xml-PowerTools)
(MIT). The MIT attribution for the ported portions is preserved; see the
license section below.

## License

Licensed under the [GNU Affero General Public License v3.0](LICENSE)
(AGPL-3.0-only).

Portions are derived from MIT-licensed works (Docxodus,
Open-Xml-PowerTools); their original copyright notices apply to those
portions.
