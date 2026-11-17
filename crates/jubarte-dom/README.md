<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte-dom

The document-object-model floor of the [jubarte redline
engine](https://github.com/jandira-tech/jubarte-redlines): the arena XML
DOM (`xmllinq`), WordprocessingML namespace and `XName` constants
(`namespaces`), short and unique ids (`ids`, `unid`), compiled-out
performance counters (`perf`) and shared small utilities (`util`).

Every jubarte crate above this one (the OPC adapter, the compare and
render products, the doc/markdown/edit layers) depends on it downward;
it depends on no jubarte crate itself.

License: AGPL-3.0-only (see [LICENSE](LICENSE)); the engine's Word
parity and naming are described in the root repository.
