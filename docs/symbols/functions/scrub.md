# Functions and methods in `src/scrub.rs`

18 non-test, 11 test. Part of the [symbol manifest](../README.md).

#### `<ScrubOptions as Default>::default`  (L69-76, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `<ScrubSelection as ScrubOptions>::options` (bin/jubarte.rs:67)

#### `<ScrubError as std::fmt::Display>::fmt`  (L90-95, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `scrub`  (L109-151, pub)
`pub fn scrub(docx: &[u8], options: &ScrubOptions) -> Result<Vec<u8>, ScrubError>`

- **Does:** Remove the identifying data `options` names from a `.docx`.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:456) L110; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L111; `PartFs::part_bytes` (opc/mod.rs:493) (by name) L113; `validate` (validate.rs:420) L126,142; `remove_comments` (scrub.rs:220) L128; `alias_authors` (scrub.rs:269) L131; `remove_rsids` (scrub.rs:326) L134; `scrub_properties` (scrub.rs:357) L137; `PartFs::to_zip` (opc/mod.rs:548) (by name) L139; `introduced` (scrub.rs:156) L143
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.join`, `.map_err`, `.to_string`, `.trim`, `Err`, `Ok`, `ScrubError::Invalid`, `ScrubError::Package`
- **Macros:** `format!`
- **Called by:** `run_scrub` (bin/jubarte.rs:106); `public_memory_leak_boundary_tests::invalid_aliases_are_rejected_before_any_source_property_is_rewritten` (scrub.rs:688)

#### `introduced`  (L156-166)
`fn introduced( before: &[crate::validate::Finding], after: &[crate::validate::Finding], ) -> Vec<String>`

- **Does:** The findings `after` has more of than `before`, as `CODE in part`, counted by code and part: removing elements shifts the paths of findings that were already there.
- **Calls (crate):** `finding_counts` (scrub.rs:168) L160,161
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.get`, `.into_iter`, `.is_none_or`, `.map`
- **Macros:** `format!`
- **Called by:** `scrub` (scrub.rs:109); `tests::introduced_counts_findings_by_code_and_part_not_path` (scrub.rs:542)

#### `finding_counts`  (L168-178)
`fn finding_counts( findings: &[crate::validate::Finding], ) -> std::collections::BTreeMap<(&str, &str), usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.entry`, `.or_default`, `std::collections::BTreeMap::new`
- **Called by:** `introduced` (scrub.rs:156)

#### `Part::load`  (L189-200)
`fn load(pkg: &PartFs, name: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L190; `Dom::new` (xmllinq/mod.rs:387) L191; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L192; `Dom::root` (xmllinq/mod.rs:822) L193
- **Calls (std/external/unresolved):** `.to_string`, `Some`
- **Called by:** `remove_comments` (scrub.rs:220); `alias_authors` (scrub.rs:269); `remove_rsids` (scrub.rs:326); `scrub_properties` (scrub.rs:357)

#### `Part::store`  (L202-207)
`fn store(self, pkg: &mut PartFs)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::set_part` (opc/mod.rs:510) L203; `Dom::serialize_document` (xmllinq/mod.rs:1322) L205
- **Calls (std/external/unresolved):** `.into_bytes`
- **Called by:** `append_documents` (append.rs:152) (?); `Transaction::write_comments_part` (edit.rs:5190) (?); `threads` (markdown/package.rs:73) (?); `active::reset` (perf.rs:240) (?); `active::set_atom_count` (perf.rs:277) (?); `active::set_unit_count` (perf.rs:281) (?); `renumber` (revision_processor/annotation_ids.rs:22) (?); `prune_orphan_comments` (revision_processor/comments.rs:66) (?); `prune_by_para` (revision_processor/comments.rs:195) (?); `prune_people` (revision_processor/comments.rs:251) (?); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); +4 more

#### `word_parts`  (L211-216)
`fn word_parts(pkg: &PartFs) -> Vec<String>`

- **Does:** The XML parts under `word/`.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L212
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`, `.into_iter`, `.starts_with`
- **Called by:** `alias_authors` (scrub.rs:269); `remove_rsids` (scrub.rs:326)

#### `remove_comments`  (L220-256)
`fn remove_comments(pkg: &mut PartFs)`

- **Does:** Drop every comment reference and range, then the comment parts and `people.xml` that no comment needs any more.
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3403) L221; `Part::load` (scrub.rs:189) L227; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L230; `Dom::parent` (xmllinq/mod.rs:597) (by name) L237; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L240,248; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L244; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L253
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.remove`, `Some`, `W::name`, `W::r_pr`, `W::r`, `crate::revision_processor::prune_orphan_comments`
- **Called by:** `scrub` (scrub.rs:109)

#### `initials`  (L259-265)
`fn initials(alias: &str) -> String`

- **Does:** The alias's initials: the first letter of each word, upper case.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter_map`, `.flat_map`, `.next`, `.split_whitespace`
- **Called by:** `alias_authors` (scrub.rs:269); `tests::initials_take_the_first_letter_of_each_word` (scrub.rs:564)

#### `alias_authors`  (L269-322)
`fn alias_authors(pkg: &mut PartFs, alias: &str)`

- **Does:** Every author in the `word/` parts becomes `alias`; `people.xml` keeps one person, the alias, without presence information.
- **Calls (crate):** `initials` (scrub.rs:259) L272; `word_parts` (scrub.rs:211) L273; `Part::load` (scrub.rs:189) L277,304; `Dom::descendants_and_self` (xmllinq/mod.rs:746) (by name) L281; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L282,290; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L287,295,316; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L301,321; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L310; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L317
- **Calls (std/external/unresolved):** `.is_some_and`, `.remove`, `Some`, `W15::name`, `W::author`, `W::name`
- **Called by:** `scrub` (scrub.rs:109)

#### `remove_rsids`  (L326-341)
`fn remove_rsids(pkg: &mut PartFs)`

- **Does:** Strip `w:rsid*` attributes and `w:rsid` elements everywhere under `word/`, and the settings' `w:rsids` list.
- **Calls (crate):** `word_parts` (scrub.rs:211) L328; `PartFs::part_string` (opc/mod.rs:498) L329; `Part::load` (scrub.rs:189) L332; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L335; `remove_rsid_transform` (markup_simplifier.rs:154) L338; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L339
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.remove`, `Some`, `W::name`
- **Called by:** `scrub` (scrub.rs:109)

#### `remove_children`  (L344-353)
`fn remove_children(part: &mut Part, names: &[XName]) -> bool`

- **Does:** Remove the elements named `names` that are children of `part`'s root.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L347; `Dom::remove` (xmllinq/mod.rs:1102) L348
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `scrub_properties` (scrub.rs:357)

#### `scrub_properties`  (L357-385)
`fn scrub_properties(pkg: &mut PartFs)`

- **Does:** Drop the people and dates of the core and extended properties, and the custom properties part.
- **Calls (crate):** `Part::load` (scrub.rs:189) L358,371; `XName::get` (xmllinq/mod.rs:180) L360,361,362,363,364,365...; `remove_children` (scrub.rs:344) L367,376; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L368,377; `PartFs::part_bytes` (opc/mod.rs:493) L380; `PartFs::remove_part` (opc/mod.rs:527) L381; `PartFs::remove_package_relationships_by_type` (opc/mod.rs:685) L382; `PartFs::remove_content_type_override` (opc/mod.rs:668) L383
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `format!`
- **Called by:** `scrub` (scrub.rs:109)

#### `leaks`  (L396-428, pub)
`pub fn leaks(docx: &[u8], needle: &str) -> Vec<String>`

- **Does:** The parts of `docx` where `needle` still occurs, sorted: in a text node, an attribute value that can hold text, an XML comment, the joined run text of a paragraph (a text split across runs), or the raw bytes of a part that is not XML (UTF-8 or UTF-16LE).
- **Calls (crate):** `xml_holds` (scrub.rs:477) L419; `contains` (scrub.rs:430) L421
- **Calls (std/external/unresolved):** `.as_bytes`, `.by_index`, `.collect`, `.encode_utf16`, `.ends_with`, `.flat_map`, `.insert`, `.into_iter`, `.into_owned`, `.is_empty`, `.is_err`, `.len`, `.name`, `.read_to_end`, `BTreeSet::new`, `String::from_utf8_lossy`, `Vec::new`, `std::io::Cursor::new`, `zip::ZipArchive::new`
- **Called by:** `Transaction::check_redactions` (edit.rs:4453); `public_memory_leak_boundary_tests::public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources` (scrub.rs:617); `public_memory_leak_boundary_tests::decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not` (scrub.rs:669)

#### `contains`  (L430-432)
`fn contains(hay: &[u8], needle: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.len`, `.windows`
- **Called by:** `leaks` (scrub.rs:396)

#### `is_number`  (L472-475)
`fn is_number(value: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_hexdigit`, `.is_empty`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `xml_holds` (scrub.rs:477); `tests::numbers_and_hex_ids_are_numbers` (scrub.rs:570)

#### `xml_holds`  (L477-520)
`fn xml_holds(xml: &str, needle: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L478; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L479; `Dom::root` (xmllinq/mod.rs:822) L480; `Dom::descendant_nodes` (xmllinq/mod.rs:730) L484; `Dom::is_text` (xmllinq/mod.rs:506) L485; `Dom::is_comment` (xmllinq/mod.rs:515) L485; `Dom::text_value` (xmllinq/mod.rs:547) L486; `Dom::is_pi` (xmllinq/mod.rs:519) L489; `Dom::pi_data` (xmllinq/mod.rs:572) L490; `Dom::is_element` (xmllinq/mod.rs:502) L493; `Dom::attr_count` (xmllinq/mod.rs:895) L494; `Dom::attr_at` (xmllinq/mod.rs:906) L495; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L496; `XName::local_name` (xmllinq/mod.rs:208) (by name) L499; `is_number` (scrub.rs:472) L499; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L508; `Dom::descendants` (xmllinq/mod.rs:668) L509; `Dom::name_is` (xmllinq/mod.rs:535) L512; `Dom::value` (xmllinq/mod.rs:1169) L513
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `Some`, `W::del_text`, `W::name`, `W::p`, `W::t`
- **Called by:** `leaks` (scrub.rs:396); `tests::a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is` (scrub.rs:576); `tests::element_names_are_no_leak` (scrub.rs:585)

## Test functions (11)

- `tests::finding` (L527): _no doc comment_
- `tests::introduced_counts_findings_by_code_and_part_not_path` (L542): _no doc comment_
- `tests::initials_take_the_first_letter_of_each_word` (L564): _no doc comment_
- `tests::numbers_and_hex_ids_are_numbers` (L570): _no doc comment_
- `tests::a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is` (L576): _no doc comment_
- `tests::element_names_are_no_leak` (L585): _no doc comment_
- `public_memory_leak_boundary_tests::package` (L598): _no doc comment_
- `public_memory_leak_boundary_tests::snapshot` (L604): _no doc comment_
- `public_memory_leak_boundary_tests::public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources` (L617): _no doc comment_
- `public_memory_leak_boundary_tests::decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not` (L669): _no doc comment_
- `public_memory_leak_boundary_tests::invalid_aliases_are_rejected_before_any_source_property_is_rewritten` (L688): _no doc comment_
