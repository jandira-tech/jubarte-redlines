# Functions and methods in `src/convert/font_table.rs`

11 non-test, 17 test. Part of the [symbol manifest](../README.md).

#### `FontTable::get`  (L46-53, pub(crate))
`pub(crate) fn get(&self, name: &str) -> Option<&FontEntry>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.find`, `.get`, `.iter`, `.map`, `.or_else`
- **Called by:** `Fonts::cjk_fallback_index` (convert/font.rs:1523); `Fonts::resolve_walk` (convert/font.rs:1818); `add_installed_faces` (convert/font.rs:3395); `FontTable::alt_name` (convert/font_table.rs:55)

#### `FontTable::alt_name`  (L55-59, pub(crate))
`pub(crate) fn alt_name(&self, name: &str) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `FontTable::get` (convert/font_table.rs:46) L56
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.filter`, `.is_empty`
- **Called by:** `Fonts::resolve_walk` (convert/font.rs:1818); `tests::font_table_ignores_invalid_rows_and_defaults_invalid_metadata` (convert/font_table.rs:307) (?); `tests::load_font_table_missing_part_is_empty` (convert/font_table.rs:357) (?); `tests::load_font_table_follows_the_font_table_relationship` (convert/font_table.rs:370) (?); `residual_font_payload_contract_tests::missing_xml_root_has_no_font_entries_or_aliases` (convert/font_table.rs:482) (?)

#### `FontTable::iter`  (L61-63, pub(crate))
`pub(crate) fn iter(&self) -> impl Iterator<Item = &FontEntry>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.values`
- **Called by:** `add_installed_faces` (convert/font.rs:3395); `load_embedded_fonts` (convert/font_table.rs:205)

#### `font_table_part`  (L66-68)
`fn font_table_part(pkg: &PartFs) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `main_rel_part` (convert/mod.rs:21310) L67
- **Called by:** `load_font_table` (convert/font_table.rs:70); `load_embedded_fonts` (convert/font_table.rs:205)

#### `load_font_table`  (L70-74, pub(crate))
`pub(crate) fn load_font_table(pkg: &PartFs) -> FontTable`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L71; `font_table_part` (convert/font_table.rs:66) L71; `parse_font_table_xml` (convert/font_table.rs:76) L72
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`
- **Called by:** `tests::load_font_table_missing_part_is_empty` (convert/font_table.rs:357); `tests::load_font_table_follows_the_font_table_relationship` (convert/font_table.rs:370); `tests::case8_odttf_parses_after_deobfuscation_when_present` (convert/font_table.rs:448); `with_layout` (convert/mod.rs:769)

#### `parse_font_table_xml`  (L76-130, pub(crate))
`pub(crate) fn parse_font_table_xml(xml: &str) -> FontTable`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L77; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L78; `Dom::root` (xmllinq/mod.rs:822) L79; `Dom::elements` (xmllinq/mod.rs:640) L83; `attr_any` (convert/mod.rs:19993) L84,88,103,106,119; `direct_named` (convert/mod.rs:4186) L88,100,117
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.enumerate`, `.insert`, `.into_iter`, `.map_or`, `.map`, `.ok`, `.to_string`, `.unwrap_or`, `FontTable::default`, `HashMap::new`, `Some`, `W::name`, `child_val`, `u32::from_str_radix`
- **Called by:** `tests::resolve_uses_altname_when_requested_face_missing` (convert/font.rs:4157); `tests::resolve_keeps_installed_face_over_altname` (convert/font.rs:4172); `tests::a_missing_script_family_is_calibri_not_the_unknown_cambria` (convert/font.rs:4240); `tests::a_missing_swiss_entry_is_calibri_not_arial` (convert/font.rs:4282); `tests::a_missing_roman_entry_is_cambria_not_times` (convert/font.rs:4298); `tests::a_missing_fixed_pitch_entry_is_calibri_not_courier` (convert/font.rs:4314); `tests::a_bare_missing_entry_takes_the_default_not_its_generic` (convert/font.rs:4330); `tests::a_missing_entry_with_an_absent_alt_name_keeps_its_generic` (convert/font.rs:4349); `tests::word_s_own_stand_in_for_an_absent_family_beats_its_alt_name` (convert/font.rs:4435); `tests::an_absent_face_claiming_thaana_draws_in_mv_boli` (convert/font.rs:4458); `tests::missing_east_asian_family_falls_to_word_s_cjk_face` (convert/font.rs:4685); `tests::missing_korean_family_falls_to_batang_or_malgun_gothic_by_class` (convert/font.rs:4718); +22 more

#### `parse_family`  (L132-141)
`fn parse_family(val: &str) -> FontFamilyClass`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `parse_font_key`  (L152-168, pub(crate))
`pub(crate) fn parse_font_key(guid: &str) -> Option<[u8; 16]>`

- **Does:** ECMA-376-2 §11 / Word `w:fontKey` GUID → 16-byte XOR key.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.enumerate`, `.filter`, `.is_ascii_hexdigit`, `.iter_mut`, `.len`, `.ok`, `.reverse`, `Some`, `u8::from_str_radix`
- **Called by:** `tests::obfuscated_embed_docx` (convert/font.rs:4851); `deobfuscate_odttf` (convert/font_table.rs:179); `tests::deobfuscate_odttf_leaves_a_plain_ttf_alone` (convert/font_table.rs:391); `tests::parse_font_key_reverses_mixed_endian_guid` (convert/font_table.rs:403); `tests::parse_font_key_case8_press_start` (convert/font_table.rs:412); `tests::deobfuscate_roundtrip_restores_ttf` (convert/font_table.rs:419); `residual_font_payload_contract_tests::invalid_guid_does_not_guess_a_key_or_corrupt_obfuscated_font_bytes` (convert/font_table.rs:494)

#### `deobfuscate_font`  (L171-175, pub(crate))
`pub(crate) fn deobfuscate_font(data: &mut [u8], key: &[u8; 16])`

- **Does:** XOR the first 32 bytes with the 16-byte key repeating twice.
- **Calls (std/external/unresolved):** `.enumerate`, `.iter_mut`, `.take`
- **Called by:** `tests::obfuscated_embed_docx` (convert/font.rs:4851); `deobfuscate_odttf` (convert/font_table.rs:179); `tests::deobfuscate_odttf_leaves_a_plain_ttf_alone` (convert/font_table.rs:391); `tests::deobfuscate_roundtrip_restores_ttf` (convert/font_table.rs:419); `residual_font_payload_contract_tests::invalid_guid_does_not_guess_a_key_or_corrupt_obfuscated_font_bytes` (convert/font_table.rs:494)

#### `deobfuscate_odttf`  (L179-194, pub(crate))
`pub(crate) fn deobfuscate_odttf(bytes: &[u8], font_key: &str) -> Vec<u8>`

- **Does:** De-obfuscate an `.odttf`, or leave an already-plain TTF/OTF alone (an obfuscated header matches an sfnt tag only by a 2^-32 chance).
- **Calls (crate):** `parse_font_key` (convert/font_table.rs:152) L190; `deobfuscate_font` (convert/font_table.rs:171) L191
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.starts_with`, `.to_vec`
- **Called by:** `load_embedded_fonts` (convert/font_table.rs:205); `tests::deobfuscate_odttf_leaves_a_plain_ttf_alone` (convert/font_table.rs:391); `tests::deobfuscate_roundtrip_restores_ttf` (convert/font_table.rs:419); `residual_font_payload_contract_tests::invalid_guid_does_not_guess_a_key_or_corrupt_obfuscated_font_bytes` (convert/font_table.rs:494); `residual_font_payload_contract_tests::recognized_plain_sfnt_signatures_never_apply_document_obfuscation` (convert/font_table.rs:527)

#### `load_embedded_fonts`  (L205-232, pub(crate))
`pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::font::EmbeddedFonts`

- **Does:** `(lowercase family, bold, italic) → deobfuscated TTF bytes`.
- **Calls (crate):** `font_table_part` (convert/font_table.rs:66) L207; `PartFs::read_rels_for` (opc/mod.rs:991) L208; `FontTable::iter` (convert/font_table.rs:61) L209; `catalogue_paints_family` (convert/font.rs:3481) L210; `PartFs::resolve_rel_target` (opc/mod.rs:829) L220; `PartFs::part_bytes` (opc/mod.rs:628) L221; `deobfuscate_odttf` (convert/font_table.rs:179) L224
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.insert`, `.into`, `.is_err`, `.iter`, `.to_ascii_lowercase`, `.zip`, `HashMap::new`, `ttf_parser::Face::parse`
- **Called by:** `tests::case8_odttf_parses_after_deobfuscation_when_present` (convert/font_table.rs:448); `with_layout` (convert/mod.rs:769)

## Test functions (17)

- `tests::font_table_parses_altname_family_and_sig` (L258): _no doc comment_
- `tests::font_table_lookup_is_case_insensitive` (L276): _no doc comment_
- `tests::font_table_parses_every_embedded_face_slot` (L282): _no doc comment_
- `tests::font_table_ignores_invalid_rows_and_defaults_invalid_metadata` (L307): _no doc comment_
- `tests::font_table_empty_xml_is_empty` (L333): _no doc comment_
- `tests::package` (L337): _no doc comment_
- `tests::load_font_table_missing_part_is_empty` (L357): _no doc comment_
- `tests::load_font_table_follows_the_font_table_relationship` (L370): _no doc comment_
- `tests::deobfuscate_odttf_leaves_a_plain_ttf_alone` (L391): _no doc comment_
- `tests::parse_font_key_reverses_mixed_endian_guid` (L403): _no doc comment_
- `tests::parse_font_key_case8_press_start` (L412): _no doc comment_
- `tests::deobfuscate_roundtrip_restores_ttf` (L419): _no doc comment_
- `tests::font_table_parses_embed_rel_and_key` (L438): _no doc comment_
- `tests::case8_odttf_parses_after_deobfuscation_when_present` (L448): _no doc comment_
- `residual_font_payload_contract_tests::missing_xml_root_has_no_font_entries_or_aliases` (L482): _no doc comment_
- `residual_font_payload_contract_tests::invalid_guid_does_not_guess_a_key_or_corrupt_obfuscated_font_bytes` (L494): _no doc comment_
- `residual_font_payload_contract_tests::recognized_plain_sfnt_signatures_never_apply_document_obfuscation` (L527): _no doc comment_
