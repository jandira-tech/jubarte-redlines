<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# What jubarte does with a hostile package

This page says what the code guarantees against hostile `.docx` packages, and
what was measured. Two runs were made on 2026-10-31, against the same 33
fixtures (the doc's B1–B10 plus the gaps found while testing them):

- **before**: `jubarte 0.16.0+6bf2efb43`, the 0.17.0 pre-release.
- **after**: `jubarte 0.16.0+c995f2d46`, the bomb-hardening branch (#2019:
  #2013–#2016, #2018, #2023, #2024).

Microsoft Word for Mac (16.115) opened the same files through the bench's
`word_pdf.py`. The full journal, fixtures and scripts are in
`jubarte-loop/agents/bombs/word_exp/`. Word's side is written up in
`editors_trial/reports/Report E - What Word Protects Against (2026).md`.

The jubarte commands were `read FILE -o view.md` (page markers on, the
default), `read --no-page-markers`, `validate`, and `compare FILE C0` against
a one-paragraph control. Each ran under a 6 GB RSS and 300 s guard. No limit
override was passed.

## The limits

`InputLimits::default` (`src/admission.rs`) bounds `read`, `validate`,
`edit`, `accept` and `reject`:

| limit | default | `InputLimits::compare()` |
| --- | --- | --- |
| compressed package | 64 MiB | 512 MiB |
| zip entries | 10,000 | 10,000 |
| one part, inflated | 64 MiB | 512 MiB |
| all parts, inflated | 256 MiB | 2 GiB |
| XML elements, all parts (#2014) | 2,000,000 | 4,000,000 |
| story units per compared side (#2024) | — | 2,000,000 |
| XML depth | 256 | 256 |
| attributes on one element | 1,024 | 1,024 |

- **Bytes are capped while inflating, not trusted from the header.**
  `read_package` (`src/opc/mod.rs`) reads each member through `take(cap + 1)`.
  A member that lies about its size still stops at the cap.
- **Elements are counted as each part inflates** (#2014), before any part is
  parsed. Comments and media do not count. The largest of 2,434 real
  documents holds 365,343 elements.
- **`read` applies its own budget first** (#2013). Before this fix, the
  page-marker layout opened the package under the comparer's budget before
  `read`'s bounded open ran.
- **A DOCTYPE is refused in every XML part** (#2015), by admission,
  `validate`, `compare` and `read` alike: `INVALID_XML`, "DTD declarations
  are unsupported". Word refuses every DTD too: a bare
  `<!DOCTYPE w:document>`, one in `styles.xml` and one in a `customXml` item
  (X1–X5, X4c the control). The refusal therefore loses no input Word opens.
  Entities were never expanded. `debug` still opens such a file to report on
  it.
- **`compare` counts its atoms before making them** (#2024). The comparer
  makes an atom of 2–4 KB for every character: 1 MiB of text peaked at
  2.1–3.9 GB. A side's characters of text plus elements (body and notes
  together, or one header or footer) past `max_story_units` is refused with
  `INPUT_LIMIT` after the parse and before any atom. The largest of 6,645
  real documents holds 925,884. It is a bound, not a cure: E2, under it,
  still takes 78 s and 5 GB.
- **`compare` refuses what its atomizer cannot read** (#2023): a
  `w:subDoc`, `w:altChunk`, `w:contentPart`, custom XML markup, or two
  direct `w:sectPr` in one story is `UNSUPPORTED_CONTENT`, where it was a
  panic (exit 101) that aborts a WASM instance.
- **`JUBARTE_TIMEOUT=SECONDS`** (#2016) bounds any command of the binary,
  `python -m jubarte_redlines` and the npm CLI. On expiry it exits 124
  (`DEADLINE_EXCEEDED`). The output path is never half-written (outputs are
  published by rename), but a hidden staging file beside it can remain, and
  a command that prints its result may have printed part of it.

## What the code guarantees

- **No part name becomes a path.** `read --extract-media` names each file
  `<sha256 prefix>.<ext>` (`src/markdown/from_docx/media.rs`). A zip entry
  such as `../escaped/x.xml` is reported (`check_part_name`) and never written
  anywhere. jubarte has no extraction path that uses an entry name.
- **No document path touches the network.** The only HTTP code is
  `jubarte self-update`, behind the `self-update` feature
  (`src/update.rs`). `scripts/check_no_network.py`, run in CI (#2018), fails
  if a network crate enters the build without that feature, or if Rust source
  outside `src/update` names a network API. A remote template, a linked
  picture, an `INCLUDEPICTURE` field and a subdocument are text to jubarte.
  Word fetched three of them on open (below).
- **Entities are never expanded**, and now a DTD is refused before parsing.

## Results

`read` is the default `read`. *nm* is `read --no-page-markers`. Peaks are
process RSS.

| fixture | shape | jubarte before | jubarte after | Word |
| --- | --- | --- | --- | --- |
| B1 | one 66 MiB `w:t` (68 KB zipped) | refused (part cap); `read` 698 MB, nm 2 MB; `compare` killed at 300 s, 4.7 GB | refused by every command; `compare` by the story budget in 0.9 s, 831 MB | CLEAN, 711 MB |
| B2 | 25,000 parts | refused (entries) | refused (entries), ≤ 2 MB | CLEAN |
| B3 | 500,000 attributes | refused (1,024 attributes) | refused (1,024 attributes), ≤ 2 MB | refused |
| B4 | 5,000-deep `w:sdt` + 250-deep table | refused (nesting) | refused (nesting), ≤ 2 MB | refused |
| B5, X1–X3 | DOCTYPE: entity bomb + external entities; bare; internal entity; `>` in the subset | exit 0; references kept literal, never expanded | refused by every command: DTD declarations are unsupported | refused (every one) |
| X4, X5 | the same DOCTYPE in a `customXml` item; in `styles.xml` | exit 0, compared | refused by every command, naming the part | alert: unreadable content, then refused |
| X4c | X4 without the DOCTYPE (control) | exit 0 | exit 0, compared | CLEAN |
| D_sdt200 | 200-deep `w:sdt` | `read` writes it; `validate` refuses at 256 | unchanged; `compare` refuses at 256 | CLEAN |
| D_sdt300/600 | 300/600-deep `w:sdt` | refused (nesting) | refused (nesting) | CLEAN |
| D_tbl60 | 60-deep nested tables (948 bytes) | read, 1 s, 4 MB | read, validated and compared, ≤ 1 MB | hung, twice (AppleEvent timeout at 375 s) |
| D_tbl200/300 | 200/300-deep tables | refused (nesting) | refused (nesting) | hung at 376 s; D_tbl300 crashed Word on an earlier run |
| B6 | zip-slip entry + traversal relationships | body; warns on the name; `validate` refuses | the same; `compare` refuses the name | alert: unreadable content |
| B7, N1–N4 | remote template, UNC subdocument, linked picture, `INCLUDEPICTURE`, subdocument | body only; no connection; N4 `compare` panicked (exit 101) | body only; no connection; N4 `compare` is `UNSUPPORTED_CONTENT` | CLEAN, and **fetched** N1, N2, N4 |
| B8 | `javascript:` link + altChunk script | body only; `validate` exits 2 | the same | refused |
| B9 | `gridSpan` 1,000,000 + 2,000 field begins + 60,000 sources | span capped at 64; 145 MB | the same; `compare` 126 MB | CLEAN, 77 pages |
| B10 | non-UTF-8 styles, truncated comments, fake PNG | body; warns; `validate` refuses | the same; `compare` refuses | alert: unreadable content |
| E1 | 1 MiB element flood (4 KB zipped) | exit 0, 115 MB | exit 0, 106 MB; `compare` 601 MB, 4 s | CLEAN, 1,187 pages, 765 MB |
| E2 | 8 MiB flood (25 KB zipped) | exit 0, 783 MB | exit 0, 777 MB; `compare` 4,986 MB, 78 s | CLEAN, 9,490 pages, 89 s, 2.36 GB |
| E5 | 1,999,817 elements, just inside the budget (67 KB zipped) | — | exit 0: 2,224 MB, nm 2,916 MB, 18 s; `compare` refused (story budget) in 2 s, 1.8 GB | CLEAN, 25,639 pages, 237 s, 3.9 GB |
| E6 | 2,001,017 elements, just past it | — | refused (elements), 2 MB; nm 64 MB; `compare` refused (story budget), 1.8 GB | — |
| E3 | 60 MiB flood (215 KB zipped) | **exit 0 at 5.5 GB** | refused (elements), 136 MB | failed after 1,641 s at 6.2 GB |
| E4 | 70 MiB flood | refused (part cap) after **4.4 GB**; nm 2 MB | refused (part cap), ≤ 1 MB; `compare` refused (elements), 208 MB | failed after 1,610 s at 7.2 GB |

## Where jubarte differs from Word

- **Word refuses it, jubarte refuses it.** DTDs, attribute floods, B4, active
  content.
- **Word opens it, jubarte refuses it.** A 66 MiB part (B1), 25,000 entries
  (B2), and content controls 300 to 600 deep. These caps are jubarte's own
  resource choices and are not Word-faithful. A Word-valid document past
  them is refused with `INPUT_LIMIT` or a nesting error, never misread.
  An override lifts the size and element caps.
- **Word is hurt by it, jubarte is not.** Element floods cost Word minutes
  and gigabytes, and the 60 and 70 MiB floods made it fail after 27 minutes
  at 6–7 GB. A 948-byte nest of tables 60 deep hung it twice. jubarte
  refuses a flood past its budget in a few MB, and reads the nested tables
  in a second.
- **Word opens it, jubarte cannot compare it.** E5 (25,639 pages in Word)
  and B1 read or refuse cheaply but are past the comparer's story budget;
  a subdocument (N4) is `UNSUPPORTED_CONTENT` for `compare`. The story
  budget is an override.
- **Word reaches the network, jubarte cannot.** Word fetched a remote
  template (N1: OPTIONS, HEAD, GET), a linked picture eight times (N2), and
  probed a subdocument (N4: HEAD), silently, with no prompt. jubarte has no
  network code on any document path.
