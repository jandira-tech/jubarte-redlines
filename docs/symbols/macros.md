# Macros

3 items (non-test). Part of the [symbol manifest](README.md).

| Name | Defined at | Vis | Does | Detail |
|---|---|---|---|---|
| `outln` | src/bin/jubarte.rs:42 | private | `println!` for the listings: a reader that stops early (`jubarte read FILE \| head`) ends the process quietly, as `write_stdout` ends a view. |  |
| `ns_struct` | src/namespaces.rs:29 | private | _no doc comment_ |  |
| `cached_xname` | src/namespaces.rs:181 | private | ── hottest WML names (extend as needed) ────────────────────────────────────── NAME-01: cache the hottest XNames in process-wide OnceLocks so every compare does not re-enter the interning table for the same syntactic names. _(from a code comment)_ |  |
