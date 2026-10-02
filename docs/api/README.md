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

Never hand-edit snapshot contents — regenerate them through the release
process instead.
