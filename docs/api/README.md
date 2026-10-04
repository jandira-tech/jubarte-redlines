<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# API snapshots

The files here (`*.api.txt`, `*.json.gz`, `*.d.ts`) are generated per
release by [`scripts/api_snapshot.py`](../../scripts/api_snapshot.py) as
part of the release process (see [VERSIONING.md](../../VERSIONING.md)) and
pinned to the tag named in each filename.

HEAD is expected to drift ahead of the newest snapshot. As of 2026-10-01,
`jubarte::markdown` and `diffDocuments` exist on main but are not in the
v0.10.1 snapshots; the next release's snapshot will carry them.

## Reading the drift between two releases

    python3 scripts/api_snapshot.py --drift v0.11.0 v0.11.2
    python3 scripts/api_snapshot.py --drift v0.11.0 v0.11.2 --private

prints the public surface (what a user of the crate can name) in full:
first what can break a caller, removed (`-`) and changed (`~`) items, then
what was added (`+`). The crate-private rest follows, counted by module, or
line by line with `--private`. Both sides are rebuilt from their `*.json.gz`,
so the report does not depend on the format of the `*.api.txt` files.
`scripts/release.sh` shows it at step 6.

## The `*.api.txt` listing

One item a line, sorted: `surface<TAB>kind<TAB>visibility<TAB>path signature`.
`surface` is `api` for the public surface and `internal` for everything else
`--document-private-items` documents. Impls every type gets without the crate
writing them (blanket impls such as `Into<U>`, auto traits such as `Send`) are
left out, and a trait impl is one line (`Type::(impl Clone)`), without its
methods.

The listings up to v0.11.2 are in the earlier three-column format
(`kind<TAB>visibility<TAB>path`), blanket impls included. They stay as they
were released; compare across the format change with `--drift`, not `diff`.

Never hand-edit snapshot contents — regenerate them through the release
process instead.
