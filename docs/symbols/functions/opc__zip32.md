# Functions and methods in `src/opc/zip32.rs`

3 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `deflate_bound`  (L32-37)
`fn deflate_bound(n: u64) -> u64`

- **Does:** The largest deflate stream `n` uncompressed bytes can produce: zlib's `compressBound` formula (`n + n/4096 + n/16384 + n/33554432 + 13`), which covers the RFC 1951 §3.2.4 stored-block worst case — a block header, a two-byte LEN and a two-byte NLEN per 65,535-byte block, five bytes of overhead each — with margin.
- **Calls (std/external/unresolved):** `.saturating_add`
- **Called by:** `zip32_plan` (opc/zip32.rs:71); `tests::deflate_bound_covers_stored_blocks` (opc/zip32.rs:181)

#### `<Zip32Limit as fmt::Display>::fmt`  (L47-49, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `zip32_plan`  (L71-106, pub)
`pub fn zip32_plan(entries: &[(&str, u64)]) -> Result<(), Zip32Limit>`

- **Does:** Refuse the write `entries` describes — `(name, uncompressed length)` per ZIP entry — when ZIP32 cannot hold it: - more than 65,534 entries (65,535 is the ZIP64 count marker); - an entry whose uncompressed size reaches `0xFFFFFFFF`, or whose worst-case compressed size ([`deflate_bound`]) does; - a local-header offset,...
- **Calls (crate):** `deflate_bound` (opc/zip32.rs:32) L81,89
- **Calls (std/external/unresolved):** `.len`, `.saturating_add`, `Err`, `Ok`, `Zip32Limit`
- **Macros:** `format!`
- **Called by:** `PartFs::to_zip` (opc/mod.rs:749) (?); `tests::a_small_package_plans_zip32` (opc/zip32.rs:114); `tests::the_refusal_leads_with_its_stable_code` (opc/zip32.rs:125); `tests::the_entry_count_ends_below_the_zip64_marker` (opc/zip32.rs:132); `tests::a_part_at_the_size_boundary_is_refused` (opc/zip32.rs:141); `tests::an_offset_at_the_boundary_is_refused` (opc/zip32.rs:155); `tests::a_central_directory_at_the_boundary_is_refused` (opc/zip32.rs:170)

## Test functions (7)

- `tests::a_small_package_plans_zip32` (L114): _no doc comment_
- `tests::the_refusal_leads_with_its_stable_code` (L125): _no doc comment_
- `tests::the_entry_count_ends_below_the_zip64_marker` (L132): _no doc comment_
- `tests::a_part_at_the_size_boundary_is_refused` (L141): _no doc comment_
- `tests::an_offset_at_the_boundary_is_refused` (L155): _no doc comment_
- `tests::a_central_directory_at_the_boundary_is_refused` (L170): _no doc comment_
- `tests::deflate_bound_covers_stored_blocks` (L181): _no doc comment_
