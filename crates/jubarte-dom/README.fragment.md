<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

The XML floor of the [jubarte]({{repository}}) DOCX engine: an
arena-backed, namespace-aware XML DOM in the style of LINQ to XML, the
WordprocessingML namespace and element-name constants, the short ids jubarte
prints (`p12`, `r3`, `c27`), and the small utilities every layer above shares.
It knows XML, not Word packages: zip containers and parts start one layer up,
in `jubarte-opc`.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-dom` is the bottom layer. It depends on no other jubarte crate, and
every jubarte crate depends on it.

## When to use this crate

Depend on `jubarte-dom` when you want the XML tree, the name constants or the
id grammar alone: to read or write WordprocessingML parts you already hold as
strings, or to parse the ids jubarte's CLI and bindings print. To compare,
render or edit documents, depend on the crate for that task, or on
the facade crate [`jubarte`](https://crates.io/crates/jubarte), which re-exports every layer.

## Install

```sh
cargo add jubarte-dom@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-dom = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `perf-profile` | Turns on the stage counters and coarse timers in `perf`, which are no-ops otherwise. Diagnostic only: do not time a profiled build. |
| `schema` | Derives `schemars::JsonSchema` for the serializable id types. |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust
use jubarte_dom::namespaces::W;
use jubarte_dom::xmllinq::Dom;

let mut dom = Dom::new();
let doc = dom.parse_xdocument(concat!(
    r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    "<w:r><w:t>Hello</w:t></w:r><w:r><w:t>, world</w:t></w:r></w:p>",
));
let paragraph = dom.root(doc).expect("one root element");
assert_eq!(dom.elements(paragraph, Some(&W::r())).len(), 2);
assert_eq!(dom.value(paragraph), "Hello, world");
println!("{}", dom.serialize_document(doc));
```

## Modules

- `xmllinq`: the arena DOM (`Dom`, `NodeId`, `XName`, `XNamespace`), its
  parser and its serializer.
- `namespaces`: namespace URIs and interned element and attribute names
  (`W::p()`, `W::name("tbl")`, …) for WordprocessingML, DrawingML, VML and
  the rest.
- `ids`: the grammar of the ids jubarte prints and reads: changes (`r12`),
  comments (`c27`), paragraphs (`p12`), stories (`header1`) and table cells.
- `unid`: the unique per-element ids the comparer assigns.
- `perf`: compiled-out stage counters and timers (see `perf-profile`).
- `util`: shared helpers: SHA-1 and FNV fingerprints, dates, word tokens.

## License

[AGPL-3.0-only](LICENSE). Review the license's requirements before you embed
the crate in a product or a network service.
