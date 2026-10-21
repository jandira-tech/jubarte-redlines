# Functions and methods in `src/admission.rs`

30 non-test, 74 test. Part of the [symbol manifest](../README.md).

#### `<InputLimits as Default>::default`  (L64-77, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1452); `tests::a_dual_header_package_is_refused` (admission.rs:1537); `tests::a_plain_docx_is_admitted` (admission.rs:1568); `tests::budgets_admit_their_exact_boundary` (admission.rs:1580); `tests::a_highly_compressed_part_stops_at_the_budget` (admission.rs:1621); `tests::an_understated_size_is_caught_while_inflating` (admission.rs:1655); `tests::unbounded_part_and_package_budgets_admit_a_docx` (admission.rs:2067); `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (admission.rs:2103); `tests::rtf_is_unsupported_not_a_zip_error` (admission.rs:2125); `tests::zip17_rejects_local_header_disagreement` (admission.rs:2176); `tests::zip17_compares_the_utf8_flag_only_for_non_ascii_names` (admission.rs:2212); `tests::zip17_accepts_legitimate_descriptor_and_layout_shapes` (admission.rs:2326); +24 more

#### `InputLimits::compare`  (L87-100, pub)
`pub const fn compare() -> Self`

- **Does:** The redline comparer's budget: the same entry and depth caps as [`Self::default`], with room for the embedded media legal corpora carry (512 MiB per file and per part, 2 GiB inflated in all).
- **Calls:** none
- **Called by:** `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (admission.rs:2103); `limit_test_support::limits` (admission.rs:2705); `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121); `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493); `Package::open` (debug.rs:180); `admit_package` (document_comparer.rs:4926); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10115); `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (document_comparer.rs:11373); `coverage_package_source_matrix_tests::public_comparison_preserves_complete_sources_across_package_features` (document_comparer.rs:11713); `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (document_comparer.rs:11932); `PartFs::open` (opc/mod.rs:585); `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (opc/mod.rs:1085); +5 more

#### `InputLimitOverrides::from_json`  (L137-139, pub)
`pub fn from_json(json: &str) -> Result<Self, String>`

- **Does:** Parse a JSON object such as `{"max_part_bytes": 67108864}`.
- **Calls (std/external/unresolved):** `.map_err`, `serde_json::from_str`
- **Macros:** `format!`
- **Called by:** none found in src

#### `InputLimitOverrides::apply`  (L143-165, pub)
`pub fn apply(self, base: InputLimits) -> InputLimits`

- **Does:** `base` with every given key replaced.
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** none found in src

#### `AdmissionErrorKind::code`  (L197-207, pub)
`pub const fn code(self) -> &'static str`

- **Does:** The stable error code agents and bindings see.
- **Calls:** none
- **Called by:** `AdmissionError::code` (admission.rs:307)

#### `code_first`  (L228-245, pub)
`pub fn code_first(message: &str) -> Option<String>`

- **Does:** A refusal's message with its code first (`CODE: context: detail`): the comparer's opaque `I/O error: ` wrapper is dropped, any other context before the code (`document B: `) stays after it.
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L233
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.copied`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.join`, `.position`, `.skip`, `.split_at`, `.split`, `.take`, `Some`
- **Called by:** `tests::code_first_puts_the_code_first_and_keeps_the_context` (admission.rs:2134); `refusal` (bin/jubarte.rs:3112)

#### `sniff`  (L259-273, pub)
`pub fn sniff(bytes: &[u8]) -> Result<(), AdmissionError>`

- **Does:** Refuse, by its signature, a file that is a document but not a `.docx` package: an OLE compound file (`LEGACY_DOC`, with the save-as hint) or RTF (`UNSUPPORTED_PACKAGE`).
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L261,267
- **Calls (std/external/unresolved):** `.starts_with`, `Err`, `Ok`
- **Called by:** `sniff_package` (admission.rs:283); `admit` (admission.rs:439); `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (admission.rs:2103); `tests::rtf_is_unsupported_not_a_zip_error` (admission.rs:2125); `run_convert_any` (bin/jubarte.rs:3458); `refused_input` (cli.rs:2042); `docx_to_markdown` (markdown/mod.rs:651)

#### `sniff_package`  (L283-286, pub)
`pub fn sniff_package(bytes: &[u8]) -> Result<(), AdmissionError>`

- **Does:** [`sniff`], and a Word input's ZIP signature: what a file must be before it is read as a `.docx`.
- **Calls (crate):** `sniff` (admission.rs:259) L284; `declared_entry_count` (admission.rs:571) L285
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `refused_input` (cli.rs:2042)

#### `AdmissionError::new`  (L298-303, pub(crate))
`pub(crate) fn new(kind: AdmissionErrorKind, message: impl Into<String>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `sniff` (admission.rs:259); `input_over_compressed` (admission.rs:346); `input_over_entries` (admission.rs:357); `part_declared_over` (admission.rs:369); `part_inflated_over` (admission.rs:385); `admit` (admission.rs:439); `declared_entry_count` (admission.rs:571); `check_part_name` (admission.rs:808); `check_xml_depth` (admission.rs:858); `main_part` (admission.rs:901); `tests::error_codes_are_stable` (admission.rs:2083); `ZipDefect::refusal` (opc/archive.rs:366); +1 more

#### `AdmissionError::code`  (L307-309, pub)
`pub const fn code(&self) -> &'static str`

- **Does:** The stable error code (`INPUT_LIMIT`, …).
- **Calls (crate):** `AdmissionErrorKind::code` (admission.rs:197) L308
- **Called by:** none found in src

#### `<AdmissionError as fmt::Display>::fmt`  (L313-315, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L314
- **Macros:** `write!`
- **Called by:** none found in src

#### `input_over_compressed`  (L346-354, pub(crate))
`pub(crate) fn input_over_compressed(size: u64, limits: &InputLimits) -> AdmissionError`

- **Does:** `INPUT_LIMIT` for an input past the compressed-size budget: one wording shared by [`admit`] and the bounded package constructor in [`crate::opc`], so a refusal reads the same wherever it fires (#645).
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L347
- **Macros:** `format!`
- **Called by:** `preindex_budget` (admission.rs:399)

#### `input_over_entries`  (L357-365, pub(crate))
`pub(crate) fn input_over_entries(declared: u64, limits: &InputLimits) -> AdmissionError`

- **Does:** `INPUT_LIMIT` for a declared entry count past the entry budget.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L358
- **Macros:** `format!`
- **Called by:** `preindex_budget` (admission.rs:399)

#### `part_declared_over`  (L369-381, pub(crate))
`pub(crate) fn part_declared_over( name: &str, declared: u64, limits: &InputLimits, ) -> AdmissionError`

- **Does:** `INPUT_LIMIT` for an entry whose declared inflated size already passes the per-part budget: refused before anything inflates.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L374
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:439); `Package::open_within` (debug.rs:191); `read_package` (opc/mod.rs:325); `leaks_within` (scrub.rs:410)

#### `part_inflated_over`  (L385-395, pub(crate))
`pub(crate) fn part_inflated_over(name: &str, cap: u64, limits: &InputLimits) -> AdmissionError`

- **Does:** `INPUT_LIMIT` for an entry that inflated past `cap`, the smaller of the per-part budget and the package budget left.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L391
- **Macros:** `format!`
- **Called by:** `tally_inflated` (admission.rs:411)

#### `preindex_budget`  (L399-408, pub(crate))
`pub(crate) fn preindex_budget(bytes: &[u8], limits: InputLimits) -> Result<u64, AdmissionError>`

- **Does:** Check the input size and EOCD/ZIP64 entry count without building an index.
- **Calls (crate):** `input_over_compressed` (admission.rs:346) L401; `declared_entry_count` (admission.rs:571) L403; `input_over_entries` (admission.rs:357) L405
- **Calls (std/external/unresolved):** `.len`, `Err`, `Ok`
- **Called by:** `admit` (admission.rs:439); `Package::open_within` (debug.rs:191); `read_package` (opc/mod.rs:325); `leaks_within` (scrub.rs:410)

#### `tally_inflated`  (L411-426, pub(crate))
`pub(crate) fn tally_inflated( total: &mut u64, read: u64, name: &str, cap: u64, limits: &InputLimits, ) -> Result<(), AdmissionError>`

- **Does:** Tally actual inflation, with the same refusal wherever a reader stops.
- **Calls (crate):** `part_inflated_over` (admission.rs:385) L419,424
- **Calls (std/external/unresolved):** `.checked_add`, `.filter`, `.ok_or_else`, `Err`, `Ok`
- **Called by:** `admit` (admission.rs:439); `Package::open_within` (debug.rs:191); `read_package` (opc/mod.rs:325); `leaks_within` (scrub.rs:410)

#### `admit`  (L439-567, pub)
`pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, AdmissionError>`

- **Does:** Admit `bytes` as a DOCX within `limits`, returning facts about it on success and a typed refusal on the first budget or rule the package breaks.
- **Calls (crate):** `sniff` (admission.rs:259) L442; `preindex_budget` (admission.rs:399) L444; `AdmissionError::new` (admission.rs:298) L449,473,481,485,494,512...; `ZipDefect::refusal` (opc/archive.rs:366) (by name) L466; `zip_entry_name` (admission.rs:775) L476,516; `unicode_path_fields` (admission.rs:664) L479,519; `check_part_name` (admission.rs:808) L482; `part_declared_over` (admission.rs:369) L503; `check_xml_depth` (admission.rs:858) L534,560; `tally_inflated` (admission.rs:411) L544; `main_part` (admission.rs:901) L547
- **Calls (std/external/unresolved):** `.as_u16`, `.by_index_raw`, `.by_index`, `.by_name`, `.clone`, `.cloned`, `.compression`, `.encrypted`, `.ends_with`, `.extra_data_fields`, `.flags`, `.insert`, `.into_iter`, `.len`, `.map_err`, `.min`, `.name_raw`, `.next`, `.push`, `.read_to_end`, `.saturating_add`, `.saturating_sub`, `.size`, `.take`, `.to_ascii_lowercase`, `.to_string`, `BTreeMap::new`, `Cursor::new`, `Err`, `Ok`, `Vec::new`, `Vec::with_capacity`, `crate::opc::open_archive_unchecked`, `crate::opc::zip_defects`, `std::io::copy`, `std::io::sink`
- **Macros:** `format!`, `matches!`
- **Called by:** `tests::kind` (admission.rs:1443); `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1452); `tests::a_dual_header_package_is_refused` (admission.rs:1537); `tests::a_plain_docx_is_admitted` (admission.rs:1568); `tests::budgets_admit_their_exact_boundary` (admission.rs:1580); `tests::a_highly_compressed_part_stops_at_the_budget` (admission.rs:1621); `tests::an_understated_size_is_caught_while_inflating` (admission.rs:1655); `tests::unsafe_names_are_refused` (admission.rs:1701); `tests::encryption_and_other_compression_are_refused` (admission.rs:1727); `tests::a_main_part_without_an_xml_extension_is_still_depth_checked` (admission.rs:1822); `tests::the_main_part_follows_the_relationship` (admission.rs:1852); `tests::mismatched_unclosed_and_stray_end_tags_are_refused` (admission.rs:1885); +23 more

#### `declared_entry_count`  (L571-616)
`fn declared_entry_count(bytes: &[u8]) -> Result<u64, AdmissionError>`

- **Does:** The entry count the end-of-central-directory record declares (ZIP64 aware), which the reader's name map can undercount.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L576; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:458); `invalid` (markdown/from_docx/ooxml.rs:31)) L585,590,598,602,610,614
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_sub`, `.filter`, `.find`, `.get`, `.is_some_and`, `.len`, `.map`, `.ok_or_else`, `.ok`, `.rev`, `.saturating_add`, `.saturating_sub`, `.starts_with`, `::try_from`, `Ok`, `u64::from_le_bytes`, `u64::from`, `usize::from`, `usize::try_from`
- **Macros:** `format!`
- **Called by:** `sniff_package` (admission.rs:283); `preindex_budget` (admission.rs:399); `tests::a_zip64_end_record_is_read_at_the_locator_offset` (admission.rs:2006); `tests::a_zip64_locator_offset_past_the_end_is_refused_not_indexed` (admission.rs:2017); `tests::a_zip64_locator_offset_near_usize_max_does_not_overflow` (admission.rs:2029); `tests::a_truncated_zip64_end_record_is_refused` (admission.rs:2048)

#### `cp437_name`  (L643-661)
`fn cp437_name(bytes: &[u8]) -> String`

- **Does:** A name decoded as IBM code page 437, as zip 8 decoded flag-clear names: every byte maps (the code page is total), so decoding cannot fail.
- **Calls (std/external/unresolved):** `.and_then`, `.checked_sub`, `.collect`, `.copied`, `.get`, `.into_owned`, `.is_ascii`, `.iter`, `.map`, `.unwrap_or`, `String::from_utf8_lossy`, `char::from`, `usize::from`
- **Called by:** `zip_entry_name_raw` (admission.rs:699); `zip_entry_name` (admission.rs:775)

#### `unicode_path_fields`  (L664-671, pub(crate))
`pub(crate) fn unicode_path_fields( fields: impl Iterator<Item = zip::ExtraField>, ) -> impl Iterator<Item = zip::extra_fields::UnicodeExtraField>`

- **Does:** The Info-ZIP Unicode Path (0x7075) extra fields of an entry, in file order.
- **Calls (std/external/unresolved):** `.filter_map`, `Some`
- **Called by:** `admit` (admission.rs:439); `read_package` (opc/mod.rs:325)

#### `crc32`  (L675-688, pub(crate))
`pub(crate) fn crc32(bytes: &[u8]) -> u32`

- **Does:** CRC-32 (IEEE, reflected), as the ZIP format and the Info-ZIP Unicode Path extra field checksum use it.
- **Calls (std/external/unresolved):** `u32::from`
- **Called by:** `zip_entry_name_raw` (admission.rs:699); `zip_names::unicode_path_field` (admission.rs:1044); `zip_names::assembled` (admission.rs:1302)

#### `zip_entry_name_raw`  (L699-750, pub(crate))
`pub(crate) fn zip_entry_name_raw( raw: &[u8], flags: u16, extra: &[u8], ) -> Result<String, zip::result::ZipError>`

- **Does:** The same name [`zip_entry_name`] resolves, decoded from a raw central record's fields — the header name bytes, the general-purpose flags and the extra-field block — where no [`zip::read::ZipFile`] exists to ask (the pre-index identity walk of ZIP-15/#600).
- **Calls (crate):** `crc32` (admission.rs:675) L729,739; `cp437_name` (admission.rs:643) L748
- **Calls (std/external/unresolved):** `.and_then`, `.as_slice`, `.copied`, `.first_chunk`, `.get`, `.into_owned`, `.into`, `.is_ok`, `.last_chunk`, `.len`, `.ok`, `.push`, `.split_at_checked`, `.to_vec`, `.unwrap_or_default`, `::try_from`, `Err`, `Ok`, `String::from_utf8_lossy`, `Vec::new`, `std::str::from_utf8`, `u16::from_le_bytes`, `u32::from_le_bytes`, `usize::from`, `zip::result::ZipError::InvalidArchive`
- **Called by:** `duplicate_parts` (opc/archive.rs:743)

#### `zip_entry_name`  (L775-804, pub(crate))
`pub(crate) fn zip_entry_name( raw: &[u8], flags: u16, unicode_paths: impl Iterator<Item = zip::extra_fields::UnicodeExtraField>, ) -> Result<String, zip::result::ZipError>`

- **Does:** An entry name decoded exactly as the zip 8 reader did, from the raw name bytes, the general-purpose flag field and the Info-ZIP Unicode Path (0x7075) extra fields: bit 11 set means UTF-8 (lossy), clear means CP437, and a Unicode Path field whose CRC32 matches the header name replaces it (`zip-8.6.0/src/read.rs` 523-52...
- **Calls (crate):** `cp437_name` (admission.rs:643) L802
- **Calls (std/external/unresolved):** `.as_ref`, `.as_slice`, `.into_owned`, `.into`, `.to_vec`, `.unwrap_valid`, `Err`, `Ok`, `String::from_utf8_lossy`, `zip::result::ZipError::InvalidArchive`
- **Called by:** `admit` (admission.rs:439); `read_package` (opc/mod.rs:325)

#### `check_part_name`  (L808-832)
`fn check_part_name(name: &str) -> Result<(), AdmissionError>`

- **Does:** A ZIP name must be a relative part name: no leading slash or drive, no backslash, no empty, `.` or `..` segment.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L810
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.contains`, `.get`, `.is_empty`, `.split`, `.starts_with`, `.strip_suffix`, `.unwrap_or`, `Err`, `Ok`, `Some`, `refuse`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:439)

#### `part_text`  (L838-852)
`fn part_text(xml: &[u8]) -> Cow<'_, str>`

- **Does:** A part's text for scanning.
- **Calls (crate):** `deeper_boundary_tests::utf16` (convert/font.rs:5762) (by name) L848,849
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.iter`, `.map`, `.unwrap_or`, `Cow::Owned`, `String::from_utf8_lossy`, `char::decode_utf16`, `unit`
- **Called by:** `check_xml_depth` (admission.rs:858); `elements` (admission.rs:967)

#### `check_xml_depth`  (L858-893)
`fn check_xml_depth(name: &str, xml: &[u8], limits: InputLimits) -> Result<(), AdmissionError>`

- **Does:** Well-formed enough to scan, no deeper than `max_depth` elements, and inside the tag/namespace policy (`xmllinq::parse::TagPolicy`) so no part reaching the permissive DOM parser can carry pathological attribute or namespace-declaration widths.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L860; `part_text` (admission.rs:838) L861; `TagPolicy::tolerant` (xmllinq/parse.rs:261) L865; `TagPolicy::open` (xmllinq/parse.rs:273) L870; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:458); `invalid` (markdown/from_docx/ooxml.rs:31)) L873,882,886,890; `TagPolicy::empty` (xmllinq/parse.rs:295) L877; `TagPolicy::close` (xmllinq/parse.rs:306) L883
- **Calls (std/external/unresolved):** `.checked_sub`, `.config_mut`, `.map_err`, `.ok_or_else`, `.read_event`, `.saturating_add`, `.to_string`, `Err`, `Ok`, `Reader::from_str`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:439)

#### `main_part`  (L901-943)
`fn main_part(names: &[String], kept: &BTreeMap<String, Vec<u8>>) -> Result<String, AdmissionError>`

- **Does:** The main document part from the package relationships (falling back to `word/document.xml`, as the reader does), checked against its declared content type.
- **Calls (crate):** `AdmissionError::new` (admission.rs:298) L905,929,937; `elements` (admission.rs:967) L907; `content_type_of` (admission.rs:947) L934
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.cloned`, `.contains`, `.eq_ignore_ascii_case`, `.find_map`, `.find`, `.flatten`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.ok_or_else`, `.then`, `.to_string`, `.trim_start_matches`, `.unwrap_or_else`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:439)

#### `content_type_of`  (L947-964)
`fn content_type_of(types: &[u8], part: &str) -> Option<String>`

- **Does:** The content type `[Content_Types].xml` declares for `part`: its Override, else the Default for its extension.
- **Calls (crate):** `elements` (admission.rs:967) L949,957
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.eq_ignore_ascii_case`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.or_else`, `.rsplit_once`
- **Macros:** `format!`
- **Called by:** `main_part` (admission.rs:901)

#### `elements`  (L967-992)
`fn elements(xml: &[u8], local: &str) -> Vec<BTreeMap<String, String>>`

- **Does:** Attributes of every element with local name `local` (namespace ignored).
- **Calls (crate):** `part_text` (admission.rs:838) L968; `XName::local_name` (xmllinq/mod.rs:208) (by name) L973,978; `Dom::attributes` (xmllinq/mod.rs:899) (by name) L974
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.flatten`, `.into_inner`, `.into_owned`, `.normalized_value`, `.ok`, `.push`, `.read_event`, `.to_string`, `Reader::from_str`, `Some`, `Vec::new`
- **Called by:** `main_part` (admission.rs:901); `content_type_of` (admission.rs:947)

## Test functions (74)

- `RawEntry::raw` (L1021): An entry with the given raw name, bit 11 clear, no extra field.
- `RawEntry::utf8` (L1031): Set bit 11 (the name is UTF-8, whatever the bytes say).
- `RawEntry::unicode_path` (L1037): Add a 0x7075 field overriding `name` with `content`.
- `zip_names::unicode_path_field` (L1044): The 0x7075 payload for a field that replaces `name` with `content`.
- `zip_names::docx_package` (L1055): A minimal package — `[Content_Types].xml`, `_rels/.rels`, `word/document.xml` and `word/_rels/document.xml.rels` holding `rels` — plus `extra` entries.
- `zip_names::crc32` (L1080): CRC-32 (IEEE, reflected) as the ZIP format and zip's Unicode extra field checksum use.
- `zip_names::zip` (L1101): A Stored-method ZIP of `entries`.
- `zip_names::replace_all` (L1148): _no doc comment_
- `zip_names::set_utf8_flags` (L1164): OR bit 11 into the flag field of every header whose name is an entry with `utf8_flag` set: local headers (flags at +6, name at +30) and central headers (flags at +8, name at +46).
- `zip_names::local_header` (L1199): A local file header (`PK\3\4`) for `name`, as a record in a hand-built package reads: version 20, `flags`, stored method, no time, and the given CRC-32, sizes and extra bytes.
- `zip_names::zip64_extra` (L1224): A ZIP64 extended information extra field (0x0001) carrying both sizes, the shape a local header always writes.
- `AssembledEntry::plain` (L1270): A plain stored entry.
- `AssembledEntry::with_descriptor` (L1282): Trail the payload with a data descriptor of the given shape.
- `AssembledEntry::zip64` (L1288): Force the entry's headers into ZIP64 form.
- `zip_names::assembled` (L1302): A package assembled record by record: `prefix`, each entry's local header, payload and descriptor in order, the central records — in `central_order` indices, entry order when `None` — and the EOCD with `comment`.
- `tests::zip_of` (L1423): _no doc comment_
- `tests::docx_with` (L1433): _no doc comment_
- `tests::kind` (L1443): _no doc comment_
- `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (L1452): `admit` and `PartFs::open` decode entry names with the same rule (bit 11 / CP437 / the 0x7075 override) and refuse the same decoded duplicates, so a package one reader accepts the other accepts with the same part names (#524, #1069 zip9 review rows 1-2).
- `tests::rename_local_header` (L1518): Change only a local name in the synthetic writer's package.
- `tests::a_dual_header_package_is_refused` (L1537): GHSA-m665-v7pm-7grj (zip <= 9.0.4): a local header may name an entry differently than the central directory, so a scanner trusting local headers reviews a different package than the one readers open.
- `tests::patch_u16` (L1548): Overwrite the 2-byte field at `local_off` / `central_off` in every local and central header.
- `tests::patch` (L1552): _no doc comment_
- `tests::a_plain_docx_is_admitted` (L1568): _no doc comment_
- `tests::budgets_admit_their_exact_boundary` (L1580): _no doc comment_
- `tests::a_highly_compressed_part_stops_at_the_budget` (L1621): _no doc comment_
- `tests::an_understated_size_is_caught_while_inflating` (L1655): _no doc comment_
- `tests::duplicate_names_are_refused` (L1675): _no doc comment_
- `tests::unsafe_names_are_refused` (L1701): _no doc comment_
- `tests::encryption_and_other_compression_are_refused` (L1727): _no doc comment_
- `tests::a_corrupt_entry_is_refused` (L1754): _no doc comment_
- `tests::a_package_that_is_not_a_word_document_is_refused` (L1779): _no doc comment_
- `tests::a_main_part_without_an_xml_extension_is_still_depth_checked` (L1822): _no doc comment_
- `tests::the_main_part_follows_the_relationship` (L1852): _no doc comment_
- `tests::deep_or_malformed_xml_is_refused` (L1870): _no doc comment_
- `tests::mismatched_unclosed_and_stray_end_tags_are_refused` (L1885): _no doc comment_
- `tests::pathological_tags_and_namespaces_are_refused_at_admission` (L1906): N01/N02: parts reaching the DOM parser carry bounded tag width and namespace budgets — the admission scan refuses them before any node or attribute is constructed, whatever the part is named.
- `tests::utf16_and_non_utf8_parts_are_scanned_not_refused` (L1942): _no doc comment_
- `tests::with_zip64_tail` (L1973): Replace the trailing end-of-central-directory record of `zip` by a ZIP64 locator pointing at `record_offset`, an EOCD declaring 0xFFFF entries, and `comment` as the archive comment.
- `tests::eocd64` (L1991): A 56-byte ZIP64 end-of-central-directory record declaring `entries`.
- `tests::a_zip64_end_record_is_read_at_the_locator_offset` (L2006): _no doc comment_
- `tests::a_zip64_locator_offset_past_the_end_is_refused_not_indexed` (L2017): _no doc comment_
- `tests::a_zip64_locator_offset_near_usize_max_does_not_overflow` (L2029): _no doc comment_
- `tests::a_truncated_zip64_end_record_is_refused` (L2048): _no doc comment_
- `tests::unbounded_part_and_package_budgets_admit_a_docx` (L2067): _no doc comment_
- `tests::error_codes_are_stable` (L2083): _no doc comment_
- `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (L2103): _no doc comment_
- `tests::rtf_is_unsupported_not_a_zip_error` (L2125): _no doc comment_
- `tests::code_first_puts_the_code_first_and_keeps_the_context` (L2134): _no doc comment_
- `tests::header_offsets` (L2161): An entry's local-header and central-record offsets, located through the reader rather than by scanning for signatures.
- `tests::zip17_rejects_local_header_disagreement` (L2176): ZIP-17 item 1 (#644): a local header whose content fields disagree with the central record describes a different entry to a scanner trusting local headers — the same class of defect as a name mismatch (#594).
- `tests::patch_flag` (L2199): Set or clear one flag bit in one header of `name`'s entry.
- `tests::zip17_compares_the_utf8_flag_only_for_non_ascii_names` (L2212): #644 fix round: bit 11 only matters when the raw name is not ASCII — it selects UTF-8 over CP437 decoding, which an ASCII name decodes the same either way.
- `tests::zip17_rejects_nested_entry_with_matching_names` (L2235): ZIP-17 item 2: a central record whose local header lives inside another entry's payload is refused even though every compared field — name, method, flags, CRC-32, sizes — matches.
- `tests::zip17_rejects_entry_inside_central_directory` (L2263): ZIP-17 item 2: a local header placed after the central directory — inside the archive comment — is outside every legal extent.
- `tests::assembled_docx` (L2283): The mandatory parts plus `extra`, assembled record by record.
- `tests::zip17_rejects_descriptor_disagreement` (L2304): ZIP-17 item 3: a descriptor whose CRC-32 disagrees with the central record is refused — today the seeking reader never validates the descriptor it walks past.
- `tests::zip17_accepts_legitimate_descriptor_and_layout_shapes` (L2326): ZIP-17 items 2-3 controls: signed and unsigned ZIP32 descriptors, a small forced-ZIP64 descriptor, a descriptor followed immediately by the next local header, a package with a prefix (APPNOTE 4.3.6 allows a self-extracting stub; jubarte admits one) and one whose central order differs from its physical order all admit.
- `tests::zip17_descriptor_width_follows_the_local_zip64_extra` (L2372): #644 fix round: APPNOTE 4.3.9.2 sizes a descriptor by the local header's 0x0001 field — present at all, whatever its values — not by 0xFFFFFFFF placeholders or 4-GiB sizes alone.
- `tests::zip17_checks_directory_crc_and_accounting` (L2409): ZIP-17 item 4: a directory entry's payload goes through the same bounded CRC stream — a corrupt one is INVALID_PACKAGE — and its actual bytes charge both budgets, not an early skip.
- `tests::a_word_gfxdata_header_is_admitted_on_a_real_docx` (L2447): Word's own `o:gfxdata` VML attribute lives in `word/header*.xml` and reaches ~366 KiB in the bench corpus.
- `tests::set_central_attributes` (L2465): Overwrite `entry`'s central record: `version_made_by` at +4 (host byte high, spec low) and the external attributes at +38.
- `tests::zip16_refuses_special_entry_types` (L2480): ZIP-16 (#639): an entry recorded as a non-regular Unix type, and a directory type whose name is not slash-terminated, are refused like encryption and exotic compression (`UNSUPPORTED_PACKAGE`), with the message naming the entry and its type.
- `tests::zip16_admits_recorded_entry_types` (L2527): ZIP-16 controls (#639): the shapes real writers emit still admit — an Info-ZIP regular file and directory, an OS X host record, a DOS-host archive bit, no attributes at all, and the macOS Archive Utility shape (regular `0o644` files, `0o755` directories that carry the DOS directory bit).
- `tests::duplicate_zip_identities_refuse_with_both_spellings` (L2550): ZIP-15 (#600): two central records naming one part — byte-equal or an ASCII-case alias — is `DUPLICATE_PART` on every route, found by a raw walk of the central directory before any index collapses the second record.
- `tests::directory_entries_are_not_part_identities` (L2593): A directory entry is not a part: `word/` beside `word/document.xml` (probe p8, which Word opens) and two directory spellings of one folder admit, where two FILE entries of one identity refuse.
- `tests::a_duplicate_refusal_reads_as_the_admission_detail` (L2617): The message a text surface prints for the constructor's typed `DuplicatePartName`: the admission refusal for the same package — the `DUPLICATE_PART` code and both spellings — while any other error's text passes through unchanged (#600).
- `tests::a_case_variant_name_is_the_part` (L2656): One case-variant name is the part (ECMA-376 Part 2 §6.2.2.3 M1.12): Word opens `word/Document.xml`, `[content_types].xml` and `_RELS/.RELS` as the only spellings of the reserved and main parts (probes p3, p4, p5), so admission resolves them by identity and reports the stored spelling as the main part.
- `limit_test_support::package` (L2691): _no doc comment_
- `limit_test_support::limits` (L2705): _no doc comment_
- `limit_test_support::compressed` (L2714): _no doc comment_
- `limit_test_support::count` (L2720): _no doc comment_
- `limit_test_support::part` (L2726): _no doc comment_
- `limit_test_support::total` (L2732): _no doc comment_
