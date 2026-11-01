<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# What jubarte does with a hostile package

This page says what the code guarantees against hostile `.docx` packages, and
what was measured. Two runs were made on 2026-10-31, against the same 28
fixtures:

- **before**: `jubarte 0.16.0+6bf2efb43`, the 0.17.0 pre-release.
- **after**: the bomb-hardening branch (#2019: #2013–#2016, #2018).

Microsoft Word for Mac (16.115) opened the same files through the bench's
`word_pdf.py`. The full journal, fixtures and scripts are in
`jubarte-loop/agents/bombs/word_exp/`. Word's side is written up in
`editors_trial/reports/Report E - What Word Protects Against (2026).md`.

The jubarte commands were `read FILE -o view.md` (page markers on, the
default), `read --no-page-markers`, and `validate`. Each ran under a 6 GB
RSS and 300 s guard. No limit override was passed.

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
- **A DOCTYPE is refused** (#2015), by admission, `validate`, `compare` and
  `read` alike: `INVALID_XML`, "DTD declarations are unsupported". Word
  refuses every DTD too, a bare `<!DOCTYPE w:document>` included. The
  refusal therefore loses no input Word opens. Entities were never expanded.
- **`JUBARTE_TIMEOUT=SECONDS`** (#2016) bounds any command of the binary,
  `python -m jubarte_redlines` and the npm CLI. On expiry it exits 124
  (`DEADLINE_EXCEEDED`) and writes no output.

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
| B1 | one 66 MiB `w:t` (68 KB zipped) | refused (part cap); `read` 698 MB, nm 2 MB | AFTER_B1 | CLEAN, 711 MB |
| B2 | 25,000 parts | refused (entries) | AFTER_B2 | CLEAN |
| B3 | 500,000 attributes | refused (1,024 attributes) | AFTER_B3 | refused |
| B4 | 5,000-deep `w:sdt` + 250-deep table | refused (nesting) | AFTER_B4 | refused |
| B5, X1–X3 | DOCTYPE: entity bomb + external entities; bare; internal entity; `>` in the subset | exit 0; references kept literal, never expanded | AFTER_X | refused (every one) |
| D_sdt200 | 200-deep `w:sdt` | `read` writes it; `validate` refuses at 256 | AFTER_D200 | CLEAN |
| D_sdt300/600 | 300/600-deep `w:sdt` | refused (nesting) | AFTER_D300 | CLEAN |
| D_tbl60 | 60-deep nested tables (948 bytes) | read, 1 s, 4 MB | AFTER_T60 | WORD_T60 |
| D_tbl200/300 | 200/300-deep tables | refused (nesting) | AFTER_T200 | WORD_T200 |
| B6 | zip-slip entry + traversal relationships | body; warns on the name; `validate` refuses | AFTER_B6 | alert: unreadable content |
| B7, N1–N4 | remote template, UNC subdocument, linked picture, `INCLUDEPICTURE`, subdocument | body only; no connection | AFTER_N | CLEAN, and **fetched** N1, N2, N4 |
| B8 | `javascript:` link + altChunk script | body only; `validate` exits 2 | AFTER_B8 | refused |
| B9 | `gridSpan` 1,000,000 + 2,000 field begins + 60,000 sources | span capped at 64; 145 MB | AFTER_B9 | CLEAN, 77 pages |
| B10 | non-UTF-8 styles, truncated comments, fake PNG | body; warns; `validate` refuses | AFTER_B10 | alert: unreadable content |
| E1 | 1 MiB element flood (4 KB zipped) | exit 0, 115 MB | AFTER_E1 | CLEAN, 1,187 pages, 765 MB |
| E2 | 8 MiB flood (25 KB zipped) | exit 0, 783 MB | AFTER_E2 | CLEAN, 9,490 pages, 89 s, 2.36 GB |
| E3 | 60 MiB flood (215 KB zipped) | **exit 0 at 5.5 GB** | AFTER_E3 | WORD_E3 |
| E4 | 70 MiB flood | refused (part cap) after **4.4 GB**; nm 2 MB | AFTER_E4 | WORD_E4 |

## Where jubarte differs from Word

- **Word refuses it, jubarte refuses it.** DTDs, attribute floods, B4, active
  content.
- **Word opens it, jubarte refuses it.** A 66 MiB part (B1), 25,000 entries
  (B2), and content controls 300 to 600 deep. These caps are jubarte's own
  resource choices and are not Word-faithful. A Word-valid document past
  them is refused with `INPUT_LIMIT` or a nesting error, never misread.
  An override lifts the size and element caps.
- **Word is hurt by it, jubarte is not.** Element floods cost Word minutes
  and gigabytes. A 948-byte nest of tables 60 deep wedged it. jubarte
  refuses a flood past its budget in a few MB, and reads the nested tables
  in a second.
- **Word reaches the network, jubarte cannot.** Word fetched a remote
  template (N1: OPTIONS, HEAD, GET), a linked picture eight times (N2), and
  probed a subdocument (N4: HEAD), silently, with no prompt. jubarte has no
  network code on any document path.
