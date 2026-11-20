<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Names, ids and limits

## Namespaces

`namespaces` holds one unit struct per namespace the engine reads (`W`
for WordprocessingML, `W14`, `WP`, `R`, `MC`, ...). Each has its URI,
`Ns::ns()` for the namespace, `Ns::name(local)` for any name in it, and a
cached constructor for each name the engine uses, such as `W::p()`,
`W::r()`, `W::t()`, `W::ins()` and `W::del()`.

## Ids

`ids` is the one grammar of the short ids jubarte prints and takes: a
paragraph (`p12`), a story or a paragraph in it (`header1`,
`header1.p2`), a table cell (`t0.row1.cell2`), a change (`r12`, `r1+2`,
`header1.r1`), a comment (`c27`) and the text a comment covers
(`c27.text`). The CLI, the bindings and the edit plans all read ids
through it.

`unid` gives the comparer unique, content-independent ids from a
process-wide counter instead of random GUIDs, so the same inputs always
give the same output.

## Limits

`limits` is hidden from the API docs (`#[doc(hidden)]`): it is public so
the other jubarte crates can share it, with no stability promise. It holds the input budgets (`InputLimits`) and the typed refusals
(`AdmissionError`) that XML parsing, package admission and the comparer
share. Code above this crate reaches them as `admission::InputLimits`;
the [jubarte-opc book](https://jubarte.readthedocs.io/projects/jubarte-opc/en/latest/admission.html)
shows them in use.
