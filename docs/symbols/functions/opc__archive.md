# Functions and methods in `src/opc/archive.rs`

16 non-test, 15 test. Part of the [symbol manifest](../README.md).

#### `open_archive_unchecked`  (L20-32, pub(crate))
`pub(crate) fn open_archive_unchecked<R: Read + Seek + Clone>( reader: R, ) -> ZipResult<ZipArchive<R>>`

- **Does:** Open at offset zero first, retaining support for prepended data — APPNOTE 4.3.6 lets a self-extracting archive carry a stub ahead of the first local header, and jubarte admits one.
- **Calls (std/external/unresolved):** `.clone`, `ArchiveOffset::Known`, `Ok`, `ZipArchive::new`, `ZipArchive::with_config`
- **Called by:** `open_archive_within` (opc/archive.rs:50); `tests::open_archive_refuses_a_dual_header_package` (opc/archive.rs:931); `tests::every_committed_docx_opens_under_header_consistency` (opc/archive.rs:1082); `refusal_message` (opc/mod.rs:88) (?)

#### `open_archive`  (L42-44, pub(crate))
`pub(crate) fn open_archive<R: Read + Seek + Clone>(reader: R) -> ZipResult<ZipArchive<R>>`

- **Does:** The package open every reader shares: [`open_archive_unchecked`], then [`zip_defects`].
- **Calls (crate):** `open_archive_within` (opc/archive.rs:50) L43
- **Called by:** `tests::open_archive_does_not_scan_payload` (opc/archive.rs:850); `tests::open_archive_reads_prepended_data` (opc/archive.rs:878); `tests::open_archive_refuses_non_zip_input` (opc/archive.rs:907); `tests::open_archive_refuses_a_dual_header_package` (opc/archive.rs:931); `tests::open_archive_accepts_a_local_time_mismatch` (opc/archive.rs:956); `tests::open_archive_refuses_a_central_record_without_its_local_header` (opc/archive.rs:973); `tests::open_archive_refuses_a_local_method_mismatch` (opc/archive.rs:992); `tests::open_archive_compares_bit_11_only_for_non_ascii_names` (opc/archive.rs:1014); `tests::open_archive_admits_a_zip64_local_extra_wide_descriptor` (opc/archive.rs:1053); `tests::every_committed_docx_opens_under_header_consistency` (opc/archive.rs:1082)

#### `open_archive_within`  (L50-66, pub(crate))
`pub(crate) fn open_archive_within<R: Read + Seek + Clone>( reader: R, refuse_duplicates: bool, ) -> ZipResult<ZipArchive<R>>`

- **Does:** [`open_archive`] without the duplicate-part refusal: [`PartFs`] refuses the same identity itself, with the typed `OpcError::DuplicatePartName` its callers match on (ZIP-15/#600); every other defect still refuses here.
- **Calls (crate):** `open_archive_unchecked` (opc/archive.rs:20) L55; `zip_defects` (opc/archive.rs:494) L56; `ZipDefect::refusal` (opc/archive.rs:366) (by name) L62
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.into_iter`, `Err`, `Ok`, `ZipError::Io`, `std::io::Error::new`
- **Called by:** `open_archive` (opc/archive.rs:42); `read_package` (opc/mod.rs:325) (?)

#### `read_u16`  (L137-141)
`fn read_u16(head: &[u8], at: usize) -> Option<u16>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.map`, `.ok`, `::try_from`
- **Called by:** `header_fields` (opc/archive.rs:152); `zip64_field` (opc/archive.rs:204)

#### `read_u32`  (L143-147)
`fn read_u32(head: &[u8], at: usize) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.map`, `.ok`, `::try_from`
- **Called by:** `header_fields` (opc/archive.rs:152); `descriptor_extent` (opc/archive.rs:257)

#### `header_fields`  (L152-199)
`fn header_fields<R: Read + Seek>( reader: &mut R, at: u64, shape: &HeaderShape, ) -> Option<HeaderFields>`

- **Does:** The fields of the header at `at`, or `None` when no header of that shape is there.
- **Calls (crate):** `read_u16` (opc/archive.rs:137) L164,183,184,185,189,190...; `read_u32` (opc/archive.rs:143) L186,187,188,191
- **Calls (std/external/unresolved):** `.and_then`, `.as_slice`, `.get_mut`, `.get`, `.is_empty`, `.ok`, `.read_exact`, `.read`, `.saturating_add`, `.seek`, `.truncate`, `.unwrap_or`, `SeekFrom::Start`, `Some`, `Vec::new`, `from`, `variable`
- **Macros:** `vec!`
- **Called by:** `zip_defects` (opc/archive.rs:494); `duplicate_parts` (opc/archive.rs:743)

#### `zip64_field`  (L204-220)
`fn zip64_field(extra: &[u8]) -> Option<&[u8]>`

- **Does:** The data of the first 0x0001 (ZIP64 extended information) field in `extra`, of any length.
- **Calls (crate):** `read_u16` (opc/archive.rs:137) L207,208
- **Calls (std/external/unresolved):** `.checked_add`, `.get`, `.is_some_and`, `.len`, `usize::from`
- **Called by:** `zip64_sizes` (opc/archive.rs:225); `zip_defects` (opc/archive.rs:494)

#### `zip64_sizes`  (L225-234)
`fn zip64_sizes(extra: &[u8]) -> Option<(u64, u64)>`

- **Does:** The `(uncompressed, compressed)` sizes a local header's 0x0001 (ZIP64 extended information) extra field carries — always both, in that order (APPNOTE 4.5.3).
- **Calls (crate):** `zip64_field` (opc/archive.rs:204) L226; `own_size::{fn size}` (markdown/from_docx/agent.rs:1116) (by name) L233
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.get`, `.map`, `.ok`, `::try_from`, `Some`
- **Called by:** `resolved_local_size` (opc/archive.rs:240)

#### `resolved_local_size`  (L240-246)
`fn resolved_local_size(field: u32, extra: &[u8], which: usize) -> Option<u64>`

- **Does:** The size a local header's `field` stands for: itself, or the local ZIP64 extra field's value when it is the `0xFFFFFFFF` placeholder (`which` 0 = uncompressed, 1 = compressed).
- **Calls (crate):** `zip64_sizes` (opc/archive.rs:225) L244
- **Calls (std/external/unresolved):** `Some`, `u64::from`
- **Called by:** `zip_defects` (opc/archive.rs:494)

#### `descriptor_extent`  (L257-321)
`fn descriptor_extent<R: Read + Seek>( reader: &mut R, at: u64, zip64: bool, crc32: u32, compressed: u64, uncompressed: u64, ) -> Result<u64, &'static str>`

- **Does:** The length of the data descriptor at `at` — the bytes a bit-3 entry trails its payload with — when exactly one interpretation matches the central record.
- **Calls (crate):** `read_u32` (opc/archive.rs:143) L282; `word` (ambiguous, 5 candidates) L295,302,303,309
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.first`, `.get_mut`, `.get`, `.map_err`, `.map`, `.ok`, `.push`, `.read`, `.saturating_add`, `.seek`, `.unwrap_or_default`, `::try_from`, `Err`, `Ok`, `SeekFrom::Start`, `Some`, `Vec::with_capacity`, `qword`, `triple`
- **Called by:** `zip_defects` (opc/archive.rs:494)

#### `ZipDefect::refusal`  (L366-386, pub(crate))
`pub(crate) fn refusal(&self) -> crate::admission::AdmissionError`

- **Does:** The typed refusal this defect produces on a reading route.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L369,377,380,383
- **Calls (std/external/unresolved):** `.clone`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:439) (?); `analyze` (debug.rs:1220) (?); `open_archive_within` (opc/archive.rs:50) (?); `refusal_message` (opc/mod.rs:88) (?)

#### `ZipDefect::finding_kind`  (L389-399, pub(crate))
`pub(crate) fn finding_kind(&self) -> &'static str`

- **Does:** The `debug` finding kind this defect reports as.
- **Calls:** none
- **Called by:** `analyze` (debug.rs:1220) (?)

#### `special_entry`  (L427-466)
`fn special_entry(index: usize, name: &[u8], central: &HeaderFields) -> Option<ZipDefect>`

- **Does:** The entry type recorded in the central record (ZIP-16/#639).
- **Calls (std/external/unresolved):** `.ends_with`, `.into_owned`, `.to_string`, `.unwrap_or`, `Some`, `String::from_utf8_lossy`, `defect`
- **Macros:** `format!`, `matches!`
- **Called by:** `zip_defects` (opc/archive.rs:494)

#### `zip_defects`  (L494-686, pub(crate))
`pub(crate) fn zip_defects<R: Read + Seek>( archive: &ZipArchive<R>, reader: &mut R, ) -> Vec<ZipDefect>`

- **Does:** Every structural disagreement between what the central directory declares and what the package's bytes carry, in archive order (GHSA-m665-v7pm-7grj, ZIP-17/#644): - the type the central record's version-made-by host byte and external attributes record: only a regular file, or a directory on a `/`-terminated name, is a...
- **Calls (crate):** `header_fields` (opc/archive.rs:152) L507,522; `special_entry` (opc/archive.rs:427) L515; `resolved_local_size` (opc/archive.rs:240) L578; `zip64_field` (opc/archive.rs:204) L621; `descriptor_extent` (opc/archive.rs:257) L623; `duplicate_parts` (opc/archive.rs:743) L681
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.by_index_data`, `.central_directory_start`, `.central_header_start`, `.chain`, `.checked_add`, `.clone`, `.compressed_size`, `.extend`, `.header_start`, `.into_iter`, `.into_owned`, `.is_some`, `.iter`, `.len`, `.map_or_else`, `.map`, `.push`, `.size`, `.skip`, `.sort_unstable_by_key`, `.to_string`, `.zip`, `Some`, `String::from_utf8_lossy`, `Vec::new`, `Vec::with_capacity`, `mismatch`, `u64::from`
- **Macros:** `format!`
- **Called by:** `open_archive_within` (opc/archive.rs:50)

#### `declared_records`  (L693-723)
`fn declared_records<R: Read + Seek>(reader: &mut R) -> Option<u64>`

- **Does:** The number of central records the end of central directory declares — its ZIP64 record's when the 16-bit field is saturated (APPNOTE 4.3.16, 4.3.14) — read from at most the 64 KiB tail an end record can sit in and the 56-byte ZIP64 record, so it never touches entry data.
- **Calls (std/external/unresolved):** `.checked_sub`, `.find`, `.first_chunk`, `.get`, `.is_some_and`, `.len`, `.min`, `.ok`, `.read_exact`, `.rev`, `.seek`, `.starts_with`, `SeekFrom::End`, `SeekFrom::Start`, `Some`, `try_from`, `u16::from_le_bytes`, `u64::from_le_bytes`, `u64::from`
- **Macros:** `vec!`
- **Called by:** `duplicate_parts` (opc/archive.rs:743)

#### `duplicate_parts`  (L743-804, pub(crate))
`pub(crate) fn duplicate_parts<R: Read + Seek>( reader: &mut R, mut at: u64, ) -> Vec<(String, ZipDefect)>`

- **Does:** Two central records naming one OPC part (`DUPLICATE_PART`, ZIP-15/#600, ECMA-376 Part 2 §6.2.2.3 M1.12).
- **Calls (crate):** `declared_records` (opc/archive.rs:693) L749; `header_fields` (opc/archive.rs:152) L753; `zip_entry_name_raw` (admission.rs:699) L770; `identity` (opc/mod.rs:157) L774
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.get`, `.insert`, `.map_while`, `.ok`, `.push`, `.saturating_add`, `.unwrap_or`, `Vec::new`, `std::collections::HashMap::new`, `u64::from`, `usize::try_from`
- **Macros:** `format!`
- **Called by:** `zip_defects` (opc/archive.rs:494); `refusal_message` (opc/mod.rs:88) (?); `read_package` (opc/mod.rs:325) (?)

## Test functions (15)

- `<CountingReader as Read>::read` (L824): _no doc comment_
- `<CountingReader as Seek>::seek` (L832): _no doc comment_
- `tests::stored_package` (L837): _no doc comment_
- `tests::open_archive_does_not_scan_payload` (L850): _no doc comment_
- `tests::open_archive_reads_prepended_data` (L878): _no doc comment_
- `tests::open_archive_refuses_non_zip_input` (L907): _no doc comment_
- `tests::rename_local_header` (L913): Change only the local name, locating the header through the central record rather than searching payload bytes.
- `tests::open_archive_refuses_a_dual_header_package` (L931): GHSA-m665-v7pm-7grj: the shared open refuses a package whose local header names an entry differently than the central directory; the unchecked open accepts it so `admit` and `debug` can see it.
- `tests::retime_local_header` (L941): Change only the local header's last-modified time (`+10`), as a tool that rewrites the central record can leave it.
- `tests::open_archive_accepts_a_local_time_mismatch` (L956): A local time that disagrees with the central record hides no content, and Word opens such files (three real inputs carry one on `word/document.xml`).
- `tests::open_archive_refuses_a_central_record_without_its_local_header` (L973): A central record pointing at bytes that are no local file header is refused too: a scanner walking local headers would not find it.
- `tests::open_archive_refuses_a_local_method_mismatch` (L992): ZIP-17 (#644): the open refuses a compression method the local header records differently than the central directory — the same dual-header class as a name disagreement.
- `tests::open_archive_compares_bit_11_only_for_non_ascii_names` (L1014): #644 fix round: bit 11 is compared only when a raw name byte is 0x80 or above — an ASCII name decodes identically whether the bit is set, so the one-sided flag a real Word-opened file carries is not a mismatch.
- `tests::open_archive_admits_a_zip64_local_extra_wide_descriptor` (L1053): #644 fix round: a bit-3 entry whose local header carries a 0x0001 ZIP64 extra field takes a 24-byte descriptor (APPNOTE 4.3.9.2), whatever its sizes say — Word opens the shape.
- `tests::every_committed_docx_opens_under_header_consistency` (L1082): The header check refuses only headers that disagree: every `.docx` the repository commits under `tests/` (fixtures, corpus, goldens) must still open.
