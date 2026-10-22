# Functions and methods in `src/scrub.rs`

19 non-test, 18 test. Part of the [symbol manifest](../README.md).

#### `<ScrubOptions as Default>::default`  (L69-76, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `<ScrubSelection as ScrubOptions>::options` (bin/jubarte.rs:74)

#### `<ScrubError as std::fmt::Display>::fmt`  (L90-95, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `scrub`  (L109-152, pub)
`pub fn scrub(docx: &[u8], options: &ScrubOptions) -> Result<Vec<u8>, ScrubError>`

- **Does:** Remove the identifying data `options` names from a `.docx`.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L110; `refusal_message` (opc/mod.rs:88) L111; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L112; `PartFs::part_bytes` (opc/mod.rs:628) (by name) L114; `validate` (validate.rs:1140) L127,143; `remove_comments` (scrub.rs:221) L129; `alias_authors` (scrub.rs:270) L132; `remove_rsids` (scrub.rs:327) L135; `scrub_properties` (scrub.rs:358) L138; `PartFs::to_zip` (opc/mod.rs:749) (by name) L140; `introduced` (scrub.rs:157) L144
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.join`, `.map_err`, `.to_string`, `.trim`, `Err`, `Ok`, `ScrubError::Invalid`, `ScrubError::Package`
- **Macros:** `format!`
- **Called by:** `run_scrub` (bin/jubarte.rs:114); `Transaction::resolve_one` (edit.rs:3479); `public_memory_leak_boundary_tests::invalid_aliases_are_rejected_before_any_source_property_is_rewritten` (scrub.rs:739)

#### `introduced`  (L157-167)
`fn introduced( before: &[crate::validate::Finding], after: &[crate::validate::Finding], ) -> Vec<String>`

- **Does:** The findings `after` has more of than `before`, as `CODE in part`, counted by code and part: removing elements shifts the paths of findings that were already there.
- **Calls (crate):** `finding_counts` (scrub.rs:169) L161,162
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.get`, `.into_iter`, `.is_none_or`, `.map`
- **Macros:** `format!`
- **Called by:** `scrub` (scrub.rs:109); `tests::introduced_counts_findings_by_code_and_part_not_path` (scrub.rs:593)

#### `finding_counts`  (L169-179)
`fn finding_counts( findings: &[crate::validate::Finding], ) -> std::collections::BTreeMap<(&str, &str), usize>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.entry`, `.or_default`, `std::collections::BTreeMap::new`
- **Called by:** `introduced` (scrub.rs:157)

#### `Part::load`  (L190-201)
`fn load(pkg: &PartFs, name: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L191; `Dom::new` (xmllinq/mod.rs:387) L192; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L193; `Dom::root` (xmllinq/mod.rs:822) L194
- **Calls (std/external/unresolved):** `.to_string`, `Some`
- **Called by:** `remove_comments` (scrub.rs:221); `alias_authors` (scrub.rs:270); `remove_rsids` (scrub.rs:327); `scrub_properties` (scrub.rs:358)

#### `Part::store`  (L203-208)
`fn store(self, pkg: &mut PartFs)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::set_part` (opc/mod.rs:661) L204; `Dom::serialize_document` (xmllinq/mod.rs:1338) L206
- **Calls (std/external/unresolved):** `.into_bytes`
- **Called by:** `append_documents` (append.rs:152) (?); `cli_main` (bin/jubarte.rs:3935) (?); `Transaction::write_comments_part` (edit.rs:6761) (?); `threads` (markdown/package.rs:74) (?); `active::reset` (perf.rs:240) (?); `active::set_atom_count` (perf.rs:277) (?); `active::set_unit_count` (perf.rs:281) (?); `prune_orphan_comments` (revision_processor/comments.rs:274) (?); `prune_by_para` (revision_processor/comments.rs:411) (?); `prune_people` (revision_processor/comments.rs:467) (?); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); +4 more

#### `word_parts`  (L212-217)
`fn word_parts(pkg: &PartFs) -> Vec<String>`

- **Does:** The XML parts under `word/`.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L213
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`, `.into_iter`, `.starts_with`
- **Called by:** `alias_authors` (scrub.rs:270); `remove_rsids` (scrub.rs:327)

#### `remove_comments`  (L221-257)
`fn remove_comments(pkg: &mut PartFs)`

- **Does:** Drop every comment reference and range, then the comment parts and `people.xml` that no comment needs any more.
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3716) L222; `Part::load` (scrub.rs:190) L228; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L231; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L238; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L241,249; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L245; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L254
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.remove`, `Some`, `W::name`, `W::r_pr`, `W::r`, `crate::revision_processor::prune_orphan_comments`
- **Called by:** `scrub` (scrub.rs:109)

#### `initials`  (L260-266)
`fn initials(alias: &str) -> String`

- **Does:** The alias's initials: the first letter of each word, upper case.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter_map`, `.flat_map`, `.next`, `.split_whitespace`
- **Called by:** `alias_authors` (scrub.rs:270); `tests::initials_take_the_first_letter_of_each_word` (scrub.rs:615)

#### `alias_authors`  (L270-323)
`fn alias_authors(pkg: &mut PartFs, alias: &str)`

- **Does:** Every author in the `word/` parts becomes `alias`; `people.xml` keeps one person, the alias, without presence information.
- **Calls (crate):** `initials` (scrub.rs:260) L273; `word_parts` (scrub.rs:212) L274; `Part::load` (scrub.rs:190) L278,305; `Dom::descendants_and_self` (xmllinq/mod.rs:746) (by name) L282; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L283,291; `Dom::set_attribute_value` (xmllinq/mod.rs:977) (by name) L288,296,317; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L302,322; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L311; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L318
- **Calls (std/external/unresolved):** `.is_some_and`, `.remove`, `Some`, `W15::name`, `W::author`, `W::name`
- **Called by:** `scrub` (scrub.rs:109)

#### `remove_rsids`  (L327-342)
`fn remove_rsids(pkg: &mut PartFs)`

- **Does:** Strip `w:rsid*` attributes and `w:rsid` elements everywhere under `word/`, and the settings' `w:rsids` list.
- **Calls (crate):** `word_parts` (scrub.rs:212) L329; `PartFs::part_string` (opc/mod.rs:636) L330; `Part::load` (scrub.rs:190) L333; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L336; `remove_rsid_transform` (markup_simplifier.rs:154) L339; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L340
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.remove`, `Some`, `W::name`
- **Called by:** `scrub` (scrub.rs:109)

#### `remove_children`  (L345-354)
`fn remove_children(part: &mut Part, names: &[XName]) -> bool`

- **Does:** Remove the elements named `names` that are children of `part`'s root.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L348; `Dom::remove` (xmllinq/mod.rs:1118) L349
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `scrub_properties` (scrub.rs:358)

#### `scrub_properties`  (L358-386)
`fn scrub_properties(pkg: &mut PartFs)`

- **Does:** Drop the people and dates of the core and extended properties, and the custom properties part.
- **Calls (crate):** `Part::load` (scrub.rs:190) L359,372; `XName::get` (xmllinq/mod.rs:180) L361,362,363,364,365,366...; `remove_children` (scrub.rs:345) L368,377; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L369,378; `PartFs::part_bytes` (opc/mod.rs:628) L381; `PartFs::remove_part` (opc/mod.rs:710) L382; `PartFs::remove_package_relationships_by_type` (opc/mod.rs:914) L383; `PartFs::remove_content_type_override` (opc/mod.rs:897) L384
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `format!`
- **Called by:** `scrub` (scrub.rs:109)

#### `leaks`  (L404-406, pub)
`pub fn leaks(docx: &[u8], needle: &str) -> Vec<String>`

- **Does:** The parts of `docx` where `needle` still occurs, sorted: in a text node, an attribute value that can hold text, an XML comment, the joined run text of a paragraph (a text split across runs), or the raw bytes of a part that is not XML (UTF-8 or UTF-16LE).
- **Calls (crate):** `leaks_within` (scrub.rs:410) L405; `InputLimits::compare` (admission.rs:87) L405
- **Called by:** `Transaction::check_redactions` (edit.rs:5846); `public_memory_leak_boundary_tests::public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources` (scrub.rs:668); `public_memory_leak_boundary_tests::decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not` (scrub.rs:720)

#### `leaks_within`  (L410-479, pub(crate))
`pub(crate) fn leaks_within( docx: &[u8], needle: &str, limits: crate::admission::InputLimits, ) -> Vec<String>`

- **Does:** [`leaks`] under an explicit budget, so the fail-closed paths are provable with budgets small enough for a test.
- **Calls (crate):** `preindex_budget` (admission.rs:399) L419; `part_declared_over` (admission.rs:369) L445; `tally_inflated` (admission.rs:411) L463; `xml_holds` (scrub.rs:528) L470; `contains` (scrub.rs:481) L472
- **Calls (std/external/unresolved):** `.as_bytes`, `.by_index`, `.collect`, `.encode_utf16`, `.ends_with`, `.flat_map`, `.insert`, `.into_iter`, `.into_owned`, `.is_empty`, `.len`, `.min`, `.name`, `.read_to_end`, `.saturating_add`, `.saturating_sub`, `.size`, `.take`, `BTreeSet::new`, `String::from_utf8_lossy`, `Vec::new`, `crate::opc::open_archive`, `std::io::Cursor::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `leaks` (scrub.rs:404); `bounded_input_tests::refuses` (scrub.rs:764); `bounded_input_tests::unreadable_entry_is_a_finding` (scrub.rs:787); `bounded_input_tests::exact_budget_control` (scrub.rs:799)

#### `contains`  (L481-483)
`fn contains(hay: &[u8], needle: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.len`, `.windows`
- **Called by:** `leaks_within` (scrub.rs:410)

#### `is_number`  (L523-526)
`fn is_number(value: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_hexdigit`, `.is_empty`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `xml_holds` (scrub.rs:528); `tests::numbers_and_hex_ids_are_numbers` (scrub.rs:621)

#### `xml_holds`  (L528-571)
`fn xml_holds(xml: &str, needle: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L529; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L530; `Dom::root` (xmllinq/mod.rs:822) L531; `Dom::descendant_nodes` (xmllinq/mod.rs:730) L535; `Dom::is_text` (xmllinq/mod.rs:506) L536; `Dom::is_comment` (xmllinq/mod.rs:515) L536; `Dom::text_value` (xmllinq/mod.rs:547) L537; `Dom::is_pi` (xmllinq/mod.rs:519) L540; `Dom::pi_data` (xmllinq/mod.rs:572) L541; `Dom::is_element` (xmllinq/mod.rs:502) L544; `Dom::attr_count` (xmllinq/mod.rs:909) L545; `Dom::attr_at` (xmllinq/mod.rs:920) L546; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L547; `XName::local_name` (xmllinq/mod.rs:208) (by name) L550; `is_number` (scrub.rs:523) L550; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L559; `Dom::descendants` (xmllinq/mod.rs:668) L560; `Dom::name_is` (xmllinq/mod.rs:535) L563; `Dom::value` (xmllinq/mod.rs:1185) L564
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `Some`, `W::del_text`, `W::name`, `W::p`, `W::t`
- **Called by:** `leaks_within` (scrub.rs:410); `tests::a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is` (scrub.rs:627); `tests::element_names_are_no_leak` (scrub.rs:636)

## Test functions (18)

- `tests::finding` (L578): _no doc comment_
- `tests::introduced_counts_findings_by_code_and_part_not_path` (L593): _no doc comment_
- `tests::initials_take_the_first_letter_of_each_word` (L615): _no doc comment_
- `tests::numbers_and_hex_ids_are_numbers` (L621): _no doc comment_
- `tests::a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is` (L627): _no doc comment_
- `tests::element_names_are_no_leak` (L636): _no doc comment_
- `public_memory_leak_boundary_tests::package` (L649): _no doc comment_
- `public_memory_leak_boundary_tests::snapshot` (L655): _no doc comment_
- `public_memory_leak_boundary_tests::public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources` (L668): _no doc comment_
- `public_memory_leak_boundary_tests::decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not` (L720): _no doc comment_
- `public_memory_leak_boundary_tests::invalid_aliases_are_rejected_before_any_source_property_is_rewritten` (L739): _no doc comment_
- `bounded_input_tests::refuses` (L764): _no doc comment_
- `bounded_input_tests::compressed_limit` (L771): _no doc comment_
- `bounded_input_tests::count_limit` (L775): _no doc comment_
- `bounded_input_tests::part_limit` (L779): _no doc comment_
- `bounded_input_tests::aggregate_limit` (L783): _no doc comment_
- `bounded_input_tests::unreadable_entry_is_a_finding` (L787): _no doc comment_
- `bounded_input_tests::exact_budget_control` (L799): _no doc comment_
