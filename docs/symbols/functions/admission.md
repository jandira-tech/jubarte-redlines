# Functions and methods in `src/admission.rs`

22 non-test, 42 test. Part of the [symbol manifest](../README.md).

#### `<InputLimits as Default>::default`  (L56-65, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1052); `tests::a_plain_docx_is_admitted` (admission.rs:1116); `tests::a_highly_compressed_part_stops_at_the_budget` (admission.rs:1168); `tests::an_understated_size_is_caught_while_inflating` (admission.rs:1202); `tests::unbounded_part_and_package_budgets_admit_a_docx` (admission.rs:1614); `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (admission.rs:1649); `tests::rtf_is_unsupported_not_a_zip_error` (admission.rs:1671); `tests::a_word_gfxdata_header_is_admitted_on_a_real_docx` (admission.rs:1710); `open` (append.rs:316); `capabilities` (capabilities.rs:355); `document_text_view` (debug.rs:2011); `Opened::open` (inspect.rs:525); +1 more

#### `InputLimits::compare`  (L75-84, pub)
`pub const fn compare() -> Self`

- **Does:** The redline comparer's budget: the same entry and depth caps as [`Self::default`], with room for the embedded media legal corpora carry (512 MiB per file and per part, 2 GiB inflated in all).
- **Calls:** none
- **Called by:** `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (admission.rs:1649); `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1475); `admit_package` (document_comparer.rs:4924); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10008); `PartFs::open` (opc/mod.rs:456); `strict_to_transitional_docx` (strict_translation.rs:743); `tests::oversized_part_is_left_untouched_under_a_small_budget` (strict_translation.rs:1064); `WmlDocument::from_bytes` (wml_document.rs:50)

#### `InputLimitOverrides::from_json`  (L113-115, pub)
`pub fn from_json(json: &str) -> Result<Self, String>`

- **Does:** Parse a JSON object such as `{"max_part_bytes": 67108864}`.
- **Calls (std/external/unresolved):** `.map_err`, `serde_json::from_str`
- **Macros:** `format!`
- **Called by:** none found in src

#### `InputLimitOverrides::apply`  (L119-131, pub)
`pub fn apply(self, base: InputLimits) -> InputLimits`

- **Does:** `base` with every given key replaced.
- **Calls (std/external/unresolved):** `.unwrap_or`
- **Called by:** none found in src

#### `AdmissionErrorKind::code`  (L156-165, pub)
`pub const fn code(self) -> &'static str`

- **Does:** The stable error code agents and bindings see.
- **Calls:** none
- **Called by:** `AdmissionError::code` (admission.rs:264)

#### `code_first`  (L185-202, pub)
`pub fn code_first(message: &str) -> Option<String>`

- **Does:** A refusal's message with its code first (`CODE: context: detail`): the comparer's opaque `I/O error: ` wrapper is dropped, any other context before the code (`document B: `) stays after it.
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L190
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.copied`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.join`, `.position`, `.skip`, `.split_at`, `.split`, `.take`, `Some`
- **Called by:** `tests::code_first_puts_the_code_first_and_keeps_the_context` (admission.rs:1680); `refusal` (bin/jubarte.rs:2275)

#### `sniff`  (L216-230, pub)
`pub fn sniff(bytes: &[u8]) -> Result<(), AdmissionError>`

- **Does:** Refuse, by its signature, a file that is a document but not a `.docx` package: an OLE compound file (`LEGACY_DOC`, with the save-as hint) or RTF (`UNSUPPORTED_PACKAGE`).
- **Calls (crate):** `AdmissionError::new` (admission.rs:255) L218,224
- **Calls (std/external/unresolved):** `.starts_with`, `Err`, `Ok`
- **Called by:** `sniff_package` (admission.rs:240); `admit` (admission.rs:310); `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (admission.rs:1649); `tests::rtf_is_unsupported_not_a_zip_error` (admission.rs:1671); `run_convert_any` (bin/jubarte.rs:2477); `refused_input` (cli.rs:1703); `docx_to_markdown` (markdown/mod.rs:641)

#### `sniff_package`  (L240-243, pub)
`pub fn sniff_package(bytes: &[u8]) -> Result<(), AdmissionError>`

- **Does:** [`sniff`], and a Word input's ZIP signature: what a file must be before it is read as a `.docx`.
- **Calls (crate):** `sniff` (admission.rs:216) L241; `declared_entry_count` (admission.rs:470) L242
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `refused_input` (cli.rs:1703)

#### `AdmissionError::new`  (L255-260)
`fn new(kind: AdmissionErrorKind, message: impl Into<String>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `sniff` (admission.rs:216); `admit` (admission.rs:310); `declared_entry_count` (admission.rs:470); `check_part_name` (admission.rs:628); `check_xml_depth` (admission.rs:678); `main_part` (admission.rs:717); `tests::error_codes_are_stable` (admission.rs:1630)

#### `AdmissionError::code`  (L264-266, pub)
`pub const fn code(&self) -> &'static str`

- **Does:** The stable error code (`INPUT_LIMIT`, …).
- **Calls (crate):** `AdmissionErrorKind::code` (admission.rs:156) L265
- **Called by:** none found in src

#### `<AdmissionError as fmt::Display>::fmt`  (L270-272, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L271
- **Macros:** `write!`
- **Called by:** none found in src

#### `admit`  (L310-466, pub)
`pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, AdmissionError>`

- **Does:** Admit `bytes` as a DOCX within `limits`, returning facts about it on success and a typed refusal on the first budget or rule the package breaks.
- **Calls (crate):** `sniff` (admission.rs:216) L313; `AdmissionError::new` (admission.rs:255) L316,327,336,339,349,357...; `declared_entry_count` (admission.rs:470) L325; `zip_entry_name` (admission.rs:595) L352,404; `unicode_path_fields` (admission.rs:563) L355,407; `check_part_name` (admission.rs:628) L358; `check_xml_depth` (admission.rs:678) L422,459; `main_part` (admission.rs:717) L446
- **Calls (std/external/unresolved):** `.as_u16`, `.by_index_raw`, `.by_index`, `.by_name`, `.cloned`, `.compression`, `.encrypted`, `.ends_with`, `.extra_data_fields`, `.flags`, `.insert`, `.is_dir`, `.len`, `.map_err`, `.min`, `.name_raw`, `.read_to_end`, `.saturating_add`, `.saturating_sub`, `.size`, `.take`, `.to_ascii_lowercase`, `.to_string`, `BTreeMap::new`, `Cursor::new`, `Err`, `HashSet::new`, `Ok`, `Vec::new`, `ZipArchive::new`, `std::io::copy`, `std::io::sink`
- **Macros:** `format!`, `matches!`
- **Called by:** `tests::kind` (admission.rs:1043); `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1052); `tests::a_plain_docx_is_admitted` (admission.rs:1116); `tests::budgets_admit_their_exact_boundary` (admission.rs:1128); `tests::a_highly_compressed_part_stops_at_the_budget` (admission.rs:1168); `tests::an_understated_size_is_caught_while_inflating` (admission.rs:1202); `tests::unsafe_names_are_refused` (admission.rs:1248); `tests::encryption_and_other_compression_are_refused` (admission.rs:1274); `tests::a_main_part_without_an_xml_extension_is_still_depth_checked` (admission.rs:1369); `tests::the_main_part_follows_the_relationship` (admission.rs:1399); `tests::mismatched_unclosed_and_stray_end_tags_are_refused` (admission.rs:1432); `tests::pathological_tags_and_namespaces_are_refused_at_admission` (admission.rs:1453); +11 more

#### `declared_entry_count`  (L470-515)
`fn declared_entry_count(bytes: &[u8]) -> Result<u64, AdmissionError>`

- **Does:** The entry count the end-of-central-directory record declares (ZIP64 aware), which the reader's name map can undercount.
- **Calls (crate):** `AdmissionError::new` (admission.rs:255) L475; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:447); `invalid` (markdown/from_docx/ooxml.rs:27)) L484,489,497,501,509,513
- **Calls (std/external/unresolved):** `.and_then`, `.checked_add`, `.checked_sub`, `.filter`, `.find`, `.get`, `.is_some_and`, `.len`, `.map`, `.ok_or_else`, `.ok`, `.rev`, `.saturating_add`, `.saturating_sub`, `.starts_with`, `::try_from`, `Ok`, `u64::from_le_bytes`, `u64::from`, `usize::from`, `usize::try_from`
- **Macros:** `format!`
- **Called by:** `sniff_package` (admission.rs:240); `admit` (admission.rs:310); `tests::a_zip64_end_record_is_read_at_the_locator_offset` (admission.rs:1553); `tests::a_zip64_locator_offset_past_the_end_is_refused_not_indexed` (admission.rs:1564); `tests::a_zip64_locator_offset_near_usize_max_does_not_overflow` (admission.rs:1576); `tests::a_truncated_zip64_end_record_is_refused` (admission.rs:1595)

#### `cp437_name`  (L542-560)
`fn cp437_name(bytes: &[u8]) -> String`

- **Does:** A name decoded as IBM code page 437, as zip 8 decoded flag-clear names: every byte maps (the code page is total), so decoding cannot fail.
- **Calls (std/external/unresolved):** `.and_then`, `.checked_sub`, `.collect`, `.copied`, `.get`, `.into_owned`, `.is_ascii`, `.iter`, `.map`, `.unwrap_or`, `String::from_utf8_lossy`, `char::from`, `usize::from`
- **Called by:** `zip_entry_name` (admission.rs:595)

#### `unicode_path_fields`  (L563-570, pub(crate))
`pub(crate) fn unicode_path_fields( fields: impl Iterator<Item = zip::ExtraField>, ) -> impl Iterator<Item = zip::extra_fields::UnicodeExtraField>`

- **Does:** The Info-ZIP Unicode Path (0x7075) extra fields of an entry, in file order.
- **Calls (std/external/unresolved):** `.filter_map`, `Some`
- **Called by:** `admit` (admission.rs:310); `read_package` (opc/mod.rs:266)

#### `zip_entry_name`  (L595-624, pub(crate))
`pub(crate) fn zip_entry_name( raw: &[u8], flags: u16, unicode_paths: impl Iterator<Item = zip::extra_fields::UnicodeExtraField>, ) -> Result<String, zip::result::ZipError>`

- **Does:** An entry name decoded exactly as the zip 8 reader did, from the raw name bytes, the general-purpose flag field and the Info-ZIP Unicode Path (0x7075) extra fields: bit 11 set means UTF-8 (lossy), clear means CP437, and a Unicode Path field whose CRC32 matches the header name replaces it (`zip-8.6.0/src/read.rs` 523-52...
- **Calls (crate):** `cp437_name` (admission.rs:542) L622
- **Calls (std/external/unresolved):** `.as_ref`, `.as_slice`, `.into_owned`, `.into`, `.to_vec`, `.unwrap_valid`, `Err`, `Ok`, `String::from_utf8_lossy`, `zip::result::ZipError::InvalidArchive`
- **Called by:** `admit` (admission.rs:310); `read_package` (opc/mod.rs:266)

#### `check_part_name`  (L628-652)
`fn check_part_name(name: &str) -> Result<(), AdmissionError>`

- **Does:** A ZIP name must be a relative part name: no leading slash or drive, no backslash, no empty, `.` or `..` segment.
- **Calls (crate):** `AdmissionError::new` (admission.rs:255) L630
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.contains`, `.get`, `.is_empty`, `.split`, `.starts_with`, `.strip_suffix`, `.unwrap_or`, `Err`, `Ok`, `Some`, `refuse`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:310)

#### `part_text`  (L658-672)
`fn part_text(xml: &[u8]) -> Cow<'_, str>`

- **Does:** A part's text for scanning.
- **Calls (crate):** `deeper_boundary_tests::utf16` (convert/font.rs:5762) (by name) L668,669
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.iter`, `.map`, `.unwrap_or`, `Cow::Owned`, `String::from_utf8_lossy`, `char::decode_utf16`, `unit`
- **Called by:** `check_xml_depth` (admission.rs:678); `elements` (admission.rs:782)

#### `check_xml_depth`  (L678-712)
`fn check_xml_depth(name: &str, xml: &[u8], max_depth: usize) -> Result<(), AdmissionError>`

- **Does:** Well-formed enough to scan, no deeper than `max_depth` elements, and inside the tag/namespace policy (`xmllinq::parse::TagPolicy`) so no part reaching the permissive DOM parser can carry pathological attribute or namespace-declaration widths.
- **Calls (crate):** `AdmissionError::new` (admission.rs:255) L680; `part_text` (admission.rs:658) L681; `TagPolicy::tolerant` (xmllinq/parse.rs:224) L684; `TagPolicy::open` (xmllinq/parse.rs:236) L689; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:447); `invalid` (markdown/from_docx/ooxml.rs:27)) L692,701,705,709; `TagPolicy::empty` (xmllinq/parse.rs:248) L696; `TagPolicy::close` (xmllinq/parse.rs:257) L702
- **Calls (std/external/unresolved):** `.checked_sub`, `.config_mut`, `.map_err`, `.ok_or_else`, `.read_event`, `.saturating_add`, `.to_string`, `Err`, `Ok`, `Reader::from_str`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:310)

#### `main_part`  (L717-758)
`fn main_part<R: Read + std::io::Seek>( archive: &ZipArchive<R>, kept: &BTreeMap<String, Vec<u8>>, ) -> Result<String, AdmissionError>`

- **Does:** The main document part from the package relationships (falling back to `word/document.xml`, as the reader does), checked against its declared content type.
- **Calls (crate):** `AdmissionError::new` (admission.rs:255) L724,744,752; `elements` (admission.rs:782) L726; `content_type_of` (admission.rs:762) L749
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.contains`, `.find_map`, `.flatten`, `.get`, `.index_for_name`, `.into_iter`, `.is_none`, `.is_some_and`, `.map`, `.ok_or_else`, `.then`, `.to_string`, `.trim_start_matches`, `.unwrap_or_else`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `admit` (admission.rs:310)

#### `content_type_of`  (L762-779)
`fn content_type_of(types: &[u8], part: &str) -> Option<String>`

- **Does:** The content type `[Content_Types].xml` declares for `part`: its Override, else the Default for its extension.
- **Calls (crate):** `elements` (admission.rs:782) L764,772
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.eq_ignore_ascii_case`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.or_else`, `.rsplit_once`
- **Macros:** `format!`
- **Called by:** `main_part` (admission.rs:717)

#### `elements`  (L782-807)
`fn elements(xml: &[u8], local: &str) -> Vec<BTreeMap<String, String>>`

- **Does:** Attributes of every element with local name `local` (namespace ignored).
- **Calls (crate):** `part_text` (admission.rs:658) L783; `XName::local_name` (xmllinq/mod.rs:208) (by name) L788,793; `Dom::attributes` (xmllinq/mod.rs:885) (by name) L789
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.flatten`, `.into_inner`, `.into_owned`, `.normalized_value`, `.ok`, `.push`, `.read_event`, `.to_string`, `Reader::from_str`, `Some`, `Vec::new`
- **Called by:** `main_part` (admission.rs:717); `content_type_of` (admission.rs:762)

## Test functions (42)

- `RawEntry::raw` (L836): An entry with the given raw name, bit 11 clear, no extra field.
- `RawEntry::utf8` (L846): Set bit 11 (the name is UTF-8, whatever the bytes say).
- `RawEntry::unicode_path` (L852): Add a 0x7075 field overriding `name` with `content`.
- `zip_names::unicode_path_field` (L859): The 0x7075 payload for a field that replaces `name` with `content`.
- `zip_names::docx_package` (L870): A minimal package — `[Content_Types].xml`, `_rels/.rels`, `word/document.xml` and `word/_rels/document.xml.rels` holding `rels` — plus `extra` entries.
- `zip_names::crc32` (L895): CRC-32 (IEEE, reflected) as the ZIP format and zip's Unicode extra field checksum use.
- `zip_names::zip` (L916): A Stored-method ZIP of `entries`.
- `zip_names::replace_all` (L960): _no doc comment_
- `zip_names::set_utf8_flags` (L976): OR bit 11 into the flag field of every header whose name is an entry with `utf8_flag` set: local headers (flags at +6, name at +30) and central headers (flags at +8, name at +46).
- `tests::zip_of` (L1023): _no doc comment_
- `tests::docx_with` (L1033): _no doc comment_
- `tests::kind` (L1043): _no doc comment_
- `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (L1052): `admit` and `PartFs::open` decode entry names with the same rule (bit 11 / CP437 / the 0x7075 override) and refuse the same decoded duplicates, so a package one reader accepts the other accepts with the same part names (#524, #1069 zip9 review rows 1-2).
- `tests::patch_u16` (L1096): Overwrite the 2-byte field at `local_off` / `central_off` in every local and central header.
- `tests::patch` (L1100): _no doc comment_
- `tests::a_plain_docx_is_admitted` (L1116): _no doc comment_
- `tests::budgets_admit_their_exact_boundary` (L1128): _no doc comment_
- `tests::a_highly_compressed_part_stops_at_the_budget` (L1168): _no doc comment_
- `tests::an_understated_size_is_caught_while_inflating` (L1202): _no doc comment_
- `tests::duplicate_names_are_refused` (L1222): _no doc comment_
- `tests::unsafe_names_are_refused` (L1248): _no doc comment_
- `tests::encryption_and_other_compression_are_refused` (L1274): _no doc comment_
- `tests::a_corrupt_entry_is_refused` (L1301): _no doc comment_
- `tests::a_package_that_is_not_a_word_document_is_refused` (L1326): _no doc comment_
- `tests::a_main_part_without_an_xml_extension_is_still_depth_checked` (L1369): _no doc comment_
- `tests::the_main_part_follows_the_relationship` (L1399): _no doc comment_
- `tests::deep_or_malformed_xml_is_refused` (L1417): _no doc comment_
- `tests::mismatched_unclosed_and_stray_end_tags_are_refused` (L1432): _no doc comment_
- `tests::pathological_tags_and_namespaces_are_refused_at_admission` (L1453): N01/N02: parts reaching the DOM parser carry bounded tag width and namespace budgets — the admission scan refuses them before any node or attribute is constructed, whatever the part is named.
- `tests::utf16_and_non_utf8_parts_are_scanned_not_refused` (L1489): _no doc comment_
- `tests::with_zip64_tail` (L1520): Replace the trailing end-of-central-directory record of `zip` by a ZIP64 locator pointing at `record_offset`, an EOCD declaring 0xFFFF entries, and `comment` as the archive comment.
- `tests::eocd64` (L1538): A 56-byte ZIP64 end-of-central-directory record declaring `entries`.
- `tests::a_zip64_end_record_is_read_at_the_locator_offset` (L1553): _no doc comment_
- `tests::a_zip64_locator_offset_past_the_end_is_refused_not_indexed` (L1564): _no doc comment_
- `tests::a_zip64_locator_offset_near_usize_max_does_not_overflow` (L1576): _no doc comment_
- `tests::a_truncated_zip64_end_record_is_refused` (L1595): _no doc comment_
- `tests::unbounded_part_and_package_budgets_admit_a_docx` (L1614): _no doc comment_
- `tests::error_codes_are_stable` (L1630): _no doc comment_
- `tests::an_ole_compound_file_is_legacy_doc_with_a_save_as_hint` (L1649): _no doc comment_
- `tests::rtf_is_unsupported_not_a_zip_error` (L1671): _no doc comment_
- `tests::code_first_puts_the_code_first_and_keeps_the_context` (L1680): _no doc comment_
- `tests::a_word_gfxdata_header_is_admitted_on_a_real_docx` (L1710): Word's own `o:gfxdata` VML attribute lives in `word/header*.xml` and reaches ~366 KiB in the bench corpus.
