<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Filesystem publication (D13, D35)

The CLI and plan recorder publish the bytes produced by the engine through
`jubarte::publication`. The MCP adapter uses the same publisher through a
native `_OutputRoot` capability. Package editing and Markdown media collection
remain in memory; archive entry names are never filesystem destinations.

Each output is staged beside its destination, in an exclusively created random
staging directory (0700 on Unix), using exclusive file creation. Publication
writes, flushes, synchronizes and reads back the staged bytes before any output
in the set is committed. This verifies the exact bytes the caller supplied;
it does not claim a new OOXML or Microsoft Word validation pass.

Replacement uses capability-relative OS rename. No error path removes an old
destination to retry a rename. Without overwrite permission, installation uses
a hard link that fails atomically if the destination exists, including a
concurrent creation. Filesystems that cannot support these operations fail;
there is no direct-write fallback. Staging cleanup is attempted on drop;
cleanup errors or an interrupted process may leave staging directories. These
are not outputs.

After each commit the publisher synchronizes the parent directory. An error
includes the committed paths and explicitly marks uncertain durability when
this acknowledgement fails. Callers must inspect the error before retrying:
a failure can mean a complete file is visible with uncertain durability.
Directory synchronization support depends on the OS and filesystem; an
unsupported operation is reported, never silently treated as durable success.

A set of unrelated output files has one commit per file. All staging and
validation complete first, but a later commit can fail after earlier files
become visible. The error lists those files; there is no rollback pretending
that readers did not see them.

CLI edit bundles retain the v2 directory publication protocol: stage every
member and report in a private sibling directory, then rename the directory.
Typed edits require a new destination and include a manifest. Legacy forced
replacement retains its guarded move-aside protocol for an earlier bundle or
empty directory, refusing foreign contents and inputs. That two-rename
replacement is not a single atomic swap. Bundle members are flushed, synced
and byte-verified before publication; the final parent is synced after the
rename. Alias, collision and source-state checks still run before commit.
If a recheck or installation fails after the old bundle was moved aside,
restoration is attempted. A failed restoration adds the original bundle's
aside path and the restore error to the initial error; inspect that path to
recover the original bundle before retrying. Successful restoration retains
the initial error unchanged.
MCP bundles retain per-file commit points.

MCP opens its trusted root once at server construction. Subsequent file reads,
parent creation, staging and publication use directory handles, with confined
relative resolution. Absolute paths provided to MCP are accepted only when
lexically inside its root. Output names reject traversal, percent escapes,
Windows drive/stream syntax, reserved device names, control characters, bidi
controls, U+FEFF and trailing spaces/dots. The adapter does not decode
percent-encoded filenames.
Output symlinks are refused. Parent symlinks cannot resolve outside the granted
root; a pathname swapped after a diagnostic check does not grant new authority.
A held directory remains a granted directory if somebody renames it.

The CLI intentionally grants ambient local authority to explicitly requested
paths. It replaces an output symlink itself, not the link's target. Recording
a plan through a symlink retains its documented local behavior: the link stays
in place and its resolved plan file is atomically replaced. Plan recording
uses exclusive staging but concurrent read/modify/write recordings remain
last-writer-wins, not a serialized append or compare-and-swap.

Do not reuse the CLI's ambient image resolver in a hosted service. A hosted
asset resolver needs its own explicit input authority. The in-memory media
collector already names assets by digest and a controlled raster extension;
the CLI currently does not extract these assets to disk.

Local verification on macOS does not certify Windows runtime behavior. The
portable integration tests are suitable for a Windows runner; symlink tests
are Unix-only because Windows symlink creation requires additional privileges.
