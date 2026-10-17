<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# ooxmlsdk 0.13.2, ported to zip 9 (#1092)

This directory is the crates.io package `ooxmlsdk` 0.13.2 (upstream
<https://github.com/KaiserY/ooxmlsdk>, commit
`38d2aae6e6790a5587b7beb317a4098239e50039`, `crates/ooxmlsdk`), licensed
MIT OR Apache-2.0 by KaiserY. The root `Cargo.toml` substitutes it for the
registry crate with `[patch.crates-io]`. It is only a dev-dependency: the
integration tests load jubarte's output with `WordprocessingDocument::new` as
an independent oracle. Nothing in it ships in the jubarte package.

## Why

ooxmlsdk 0.13.2 requires `zip ^8.6.0`. zip 8.6.0 carries RUSTSEC-2026-0367,
RUSTSEC-2026-0368 and RUSTSEC-2026-0369, all fixed only from zip 9.0.0. No
8.x release fixes them. ooxmlsdk never calls the affected `extract` APIs, but
the lockfile named the vulnerable crate, and `cargo deny check advisories`
failed.

## The change

`Cargo.toml` requires `zip` 9.0.4 instead of 8.6.0. In
`src/common/package.rs`, four lines follow zip 9's API:

- `ZipFile::name()` returns `ZipResult<Cow<str>>`, where zip 8 returned
  `&str`. `read_archive_entry_names` propagates it with `?`, and the two
  error messages print `unwrap_or_default()`. zip 9 fails only on a name that
  is neither UTF-8 nor CP437, and CP437 decodes every byte, so the result is
  the name zip 8 produced.
- `ZipFile::extra_data()` returns an owned `Vec<u8>`; the Unicode-path check
  borrows it.

Nothing else differs from the published package (`diff -r` against the
registry copy shows only these lines and this file).

## Removal

Delete this directory and the `[patch.crates-io]` entry when an ooxmlsdk
release requires zip 9 or later.
