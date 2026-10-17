# Functions and methods in `src/xmllinq/parse.rs`

30 non-test, 21 test. Part of the [symbol manifest](../README.md).

#### `parse_xdocument`  (L101-112, pub)
`pub fn parse_xdocument(dom: &mut Dom, xml: &str) -> NodeId`

- **Does:** Parse an XML string into a Document node allocated into `dom`.
- **Calls (crate):** `Parser::parse_document` (xmllinq/parse.rs:450) L111
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `Dom::parse_xdocument` (xmllinq/mod.rs:1283); `try_parse_xdocument` (xmllinq/parse.rs:122); `checked_xml_tests::truncated_attribute_input_terminates_in_legacy_parser` (xmllinq/parse.rs:856); `checked_xml_tests::checked_xml_keeps_refusing_attributes_that_are_not_xml` (xmllinq/parse.rs:876); `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:946); `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (xmllinq/parse.rs:1189); `tag_policy_tests::parsed_attributes_insert_in_bulk_not_quadratic_scans` (xmllinq/parse.rs:1203); `tag_policy_tests::duplicate_expanded_names_keep_declaration_order_and_last_value_wins` (xmllinq/parse.rs:1237); `tag_policy_tests::nested_rebinding_shadows_and_restores` (xmllinq/parse.rs:1261); `tag_policy_tests::deep_rebinding_scope_grows_only_with_declared_bytes` (xmllinq/parse.rs:1285); `tag_policy_tests::mechanism_counters_are_isolated_to_the_calling_thread` (xmllinq/parse.rs:1346)

#### `try_parse_xdocument`  (L122-125, pub)
`pub fn try_parse_xdocument(dom: &mut Dom, xml: &str) -> Result<NodeId, String>`

- **Does:** Checked parse: enforce [`validate_xml`] (well-formedness plus the tag and namespace budgets) before the permissive parser builds the DOM.
- **Calls (crate):** `validate_xml` (xmllinq/parse.rs:792) L123; `parse_xdocument` (xmllinq/parse.rs:101) L124
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `Dom::try_parse_xdocument` (xmllinq/mod.rs:1294); `tag_policy_tests::element_attribute_count_is_capped_at_the_boundary` (xmllinq/parse.rs:1004); `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (xmllinq/parse.rs:1046); `tag_policy_tests::a_value_at_the_attribute_byte_cap_is_admitted_one_more_refused` (xmllinq/parse.rs:1083); `tag_policy_tests::normal_documents_parse_unchanged_through_try_parse` (xmllinq/parse.rs:1320)

#### `NsScope::get`  (L154-156)
`fn get(&self, prefix: &str) -> Option<&String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `TagPolicy::expand` (xmllinq/parse.rs:356); `resolve` (xmllinq/parse.rs:644)

#### `NsScope::mark`  (L158-160)
`fn mark(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `story_part` (markdown/view_stories.rs:1118) (?); `TagPolicy::open` (xmllinq/parse.rs:236); `TagPolicy::empty` (xmllinq/parse.rs:248); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `NsScope::push`  (L163-171)
`fn push(&mut self, prefix: &str, uri: &str)`

- **Does:** Record `prefix → uri` as a declaration owned by the current element.
- **Calls (crate):** `bump_ns_scope_declared_bytes` (xmllinq/parse.rs:86) L170
- **Calls (std/external/unresolved):** `.insert`, `.len`, `.push`, `.to_string`
- **Called by:** `TagPolicy::declare` (xmllinq/parse.rs:326); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `NsScope::pop`  (L174-190)
`fn pop(&mut self, mark: usize)`

- **Does:** Restore the scope to `mark`, undoing every push since then.
- **Calls (std/external/unresolved):** `.insert`, `.len`, `.pop`, `.remove`, `.saturating_sub`
- **Called by:** `TagPolicy::open` (xmllinq/parse.rs:236); `TagPolicy::empty` (xmllinq/parse.rs:248); `TagPolicy::close` (xmllinq/parse.rs:257); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `TagPolicy::new`  (L211-218, pub(crate))
`pub(crate) fn new() -> Self`

- **Does:** The strict gate's policy: every attribute must be well-formed XML, quoted and with a value.
- **Calls (std/external/unresolved):** `NsScope::default`, `Vec::new`
- **Called by:** `TagPolicy::tolerant` (xmllinq/parse.rs:224); `validate_xml` (xmllinq/parse.rs:792)

#### `TagPolicy::tolerant`  (L224-229, pub(crate))
`pub(crate) fn tolerant() -> Self`

- **Does:** The package admission scan's policy.
- **Calls (crate):** `TagPolicy::new` (xmllinq/parse.rs:211) L227
- **Called by:** `check_xml_depth` (admission.rs:678)

#### `TagPolicy::open`  (L236-241, pub(crate))
`pub(crate) fn open(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String>`

- **Does:** Check one open tag and push its declarations until [`Self::close`].
- **Calls (crate):** `NsScope::mark` (xmllinq/parse.rs:158) L237; `TagPolicy::check_tag` (xmllinq/parse.rs:263) L238; `NsScope::pop` (xmllinq/parse.rs:174) L238
- **Calls (std/external/unresolved):** `.inspect_err`, `.push`, `Ok`
- **Called by:** `check_xml_depth` (admission.rs:678); `validate_xml` (xmllinq/parse.rs:792)

#### `TagPolicy::empty`  (L248-253, pub(crate))
`pub(crate) fn empty(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String>`

- **Does:** Check a self-closing tag; its declarations never outlive the tag.
- **Calls (crate):** `NsScope::mark` (xmllinq/parse.rs:158) L249; `TagPolicy::check_tag` (xmllinq/parse.rs:263) L250; `NsScope::pop` (xmllinq/parse.rs:174) L251
- **Called by:** `check_xml_depth` (admission.rs:678); `validate_xml` (xmllinq/parse.rs:792)

#### `TagPolicy::close`  (L257-261, pub(crate))
`pub(crate) fn close(&mut self)`

- **Does:** Pop the declarations of the innermost open element.
- **Calls (crate):** `NsScope::pop` (xmllinq/parse.rs:174) L259
- **Calls (std/external/unresolved):** `.pop`
- **Called by:** `check_xml_depth` (admission.rs:678); `validate_xml` (xmllinq/parse.rs:792)

#### `TagPolicy::check_tag`  (L263-323)
`fn check_tag(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `TagPolicy::declare` (xmllinq/parse.rs:326) L307,309; `TagPolicy::expand` (xmllinq/parse.rs:356) L318
- **Calls (std/external/unresolved):** `.as_ref`, `.attributes`, `.html_attributes`, `.insert`, `.len`, `.map_err`, `.name`, `.normalized_value`, `.push`, `.strip_prefix`, `.to_string`, `Err`, `Ok`, `Vec::new`, `std::collections::HashSet::with_capacity`
- **Macros:** `format!`
- **Called by:** `TagPolicy::open` (xmllinq/parse.rs:236); `TagPolicy::empty` (xmllinq/parse.rs:248)

#### `TagPolicy::declare`  (L326-350)
`fn declare(&mut self, prefix: &str, uri: &str) -> Result<(), String>`

- **Does:** Push one `xmlns` declaration under the namespace budgets.
- **Calls (crate):** `NsScope::push` (xmllinq/parse.rs:163) L348
- **Calls (std/external/unresolved):** `.len`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `TagPolicy::check_tag` (xmllinq/parse.rs:263)

#### `TagPolicy::expand`  (L356-367)
`fn expand<'a>(&self, key: &'a str) -> (String, &'a str)`

- **Does:** Expand an attribute name the way the DOM parser's `resolve` does: `xmlns:*` and `xml:*` map to their reserved URIs, other prefixes go through the in-scope bindings (unbound → empty), and unprefixed attributes never take the default namespace.
- **Calls (crate):** `NsScope::get` (xmllinq/parse.rs:154) L362
- **Calls (std/external/unresolved):** `.find`, `.map_or`, `.to_string`, `String::new`
- **Called by:** `TagPolicy::check_tag` (xmllinq/parse.rs:263)

#### `Parser::len`  (L371-373)
`fn len(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Parser::cur` (xmllinq/parse.rs:375); `Parser::starts_with` (xmllinq/parse.rs:386); `Parser::index_of` (xmllinq/parse.rs:398); `Parser::slice` (xmllinq/parse.rs:423); `Parser::skip_ws` (xmllinq/parse.rs:431); `Parser::read_name` (xmllinq/parse.rs:438); `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505); `Parser::parse_pi_node` (xmllinq/parse.rs:623)

#### `Parser::cur`  (L375-381)
`fn cur(&self) -> char`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L376
- **Called by:** `Parser::skip_ws` (xmllinq/parse.rs:431); `Parser::read_name` (xmllinq/parse.rs:438); `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505); `Parser::parse_pi_node` (xmllinq/parse.rs:623)

#### `Parser::starts_with`  (L386-394)
`fn starts_with(&self, s: &str) -> bool`

- **Does:** True if the buffer at `pos` matches `s` (char-for-char).
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L389
- **Calls (std/external/unresolved):** `.chars`, `.enumerate`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Parser::index_of`  (L398-421)
`fn index_of(&self, s: &str, from: usize) -> Option<usize>`

- **Does:** Index of substring `s` at or after `from`, in char units.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L399,404,406
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.enumerate`, `.is_empty`, `.saturating_sub`, `Some`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505); `Parser::parse_pi_node` (xmllinq/parse.rs:623)

#### `Parser::slice`  (L423-429)
`fn slice(&self, start: usize, end: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L424
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.min`, `String::new`
- **Called by:** `Parser::read_name` (xmllinq/parse.rs:438); `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505); `Parser::parse_pi_node` (xmllinq/parse.rs:623)

#### `Parser::skip_ws`  (L431-435)
`fn skip_ws(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L432; `Parser::cur` (xmllinq/parse.rs:375) L432
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Parser::read_name`  (L438-448)
`fn read_name(&mut self) -> String`

- **Does:** readName — consume until whitespace, '/', '>', or '='.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L440; `Parser::cur` (xmllinq/parse.rs:375) L441; `Parser::slice` (xmllinq/parse.rs:423) L447
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Parser::parse_document`  (L450-503)
`fn parse_document(&mut self) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_document` (xmllinq/mod.rs:472) L451; `Parser::len` (xmllinq/parse.rs:371) L455; `Parser::skip_ws` (xmllinq/parse.rs:431) L457; `Parser::starts_with` (xmllinq/parse.rs:386) L458,476,481,489; `Parser::index_of` (xmllinq/parse.rs:398) L468,482,491; `Parser::slice` (xmllinq/parse.rs:423) L469,483; `parse_declaration` (xmllinq/parse.rs:671) L470; `Dom::set_declaration` (xmllinq/mod.rs:589) L471; `Parser::parse_pi_node` (xmllinq/parse.rs:623) L477; `Dom::add` (xmllinq/mod.rs:1058) L478,485,497; `Dom::new_comment` (xmllinq/mod.rs:487) L484; `Parser::cur` (xmllinq/parse.rs:375) L495; `Parser::parse_element` (xmllinq/parse.rs:505) L496
- **Calls (std/external/unresolved):** `.get`, `.is_none_or`, `.is_whitespace`, `.map`, `.unwrap_or`, `NsScope::default`, `Some`
- **Called by:** `parse_xdocument` (xmllinq/parse.rs:101)

#### `Parser::parse_element`  (L505-619)
`fn parse_element(&mut self, scope: &mut NsScope) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::read_name` (xmllinq/parse.rs:438) L508,516,574; `Parser::skip_ws` (xmllinq/parse.rs:431) L511,522,526,562,575; `Parser::len` (xmllinq/parse.rs:371) L512,530,570; `Parser::cur` (xmllinq/parse.rs:375) L512,524,527,530,563,576...; `Parser::slice` (xmllinq/parse.rs:423) L533,583,591,611; `unescape_xml_text` (xmllinq/parse.rs:704) L536,613; `NsScope::mark` (xmllinq/parse.rs:158) L543; `NsScope::push` (xmllinq/parse.rs:163) L546,548; `resolve` (xmllinq/parse.rs:644) L552,558; `Dom::new_element` (xmllinq/mod.rs:478) L553; `Dom::set_attributes` (xmllinq/mod.rs:921) L560; `NsScope::pop` (xmllinq/parse.rs:174) L565,617; `Parser::starts_with` (xmllinq/parse.rs:386) L572,581,589,597; `Parser::index_of` (xmllinq/parse.rs:398) L582,590; `Dom::new_comment` (xmllinq/mod.rs:487) L584; `Dom::add` (xmllinq/mod.rs:1058) L585,593,599,604,614; `Dom::new_text` (xmllinq/mod.rs:483) L592,613; `Parser::parse_pi_node` (xmllinq/parse.rs:623) L598; `Parser::parse_element` (xmllinq/parse.rs:505) L603
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.is_empty`, `.iter`, `.map`, `.push`, `.strip_prefix`, `.unwrap_or`, `String::new`, `Vec::new`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Parser::parse_pi_node`  (L623-639)
`fn parse_pi_node(&mut self) -> NodeId`

- **Does:** Parse a processing instruction (`<?target data?>`) into a `Pi` node.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:371) L624; `Parser::cur` (xmllinq/parse.rs:375) L628; `Parser::slice` (xmllinq/parse.rs:423) L634,636; `Parser::index_of` (xmllinq/parse.rs:398) L635; `Dom::new_pi` (xmllinq/mod.rs:492) L638
- **Calls (std/external/unresolved):** `.is_whitespace`, `.map`, `.unwrap_or`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `resolve`  (L644-668)
`fn resolve(qn: &str, is_attr: bool, local_scope: &NsScope) -> XName`

- **Does:** Resolve a qualified name against the in-scope namespaces.
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L649; `XNamespace::xml` (xmllinq/mod.rs:128) L652; `NsScope::get` (xmllinq/parse.rs:154) L654,663; `XNamespace::get` (xmllinq/mod.rs:108) L655,665; `XName::get` (xmllinq/mod.rs:180) L656,660,666
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.name`
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:505)

#### `parse_declaration`  (L671-677)
`fn parse_declaration(decl: &str) -> XDeclaration`

- **Does:** Parse the `<?xml ...?>` declaration body.
- **Calls (crate):** `extract_pseudo_attr` (xmllinq/parse.rs:680) L673,674,675
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`, `Some`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450)

#### `extract_pseudo_attr`  (L680-692)
`fn extract_pseudo_attr(s: &str, key: &str) -> Option<String>`

- **Does:** Extract `key="value"` / `key='value'` from a declaration string.
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.len`, `.next`, `.to_string`, `.trim_start`, `Some`
- **Called by:** `parse_declaration` (xmllinq/parse.rs:671)

#### `unescape_xml_text`  (L704-730, pub)
`pub fn unescape_xml_text(s: &str) -> String`

- **Does:** Decode XML entity and character references in attribute and text payloads (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&#nnn;`, `&#xhh;`).
- **Calls (crate):** `decode_entity` (xmllinq/parse.rs:732) L716
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.iter`, `.len`, `.position`, `.push_str`, `.push`, `.to_string`, `String::with_capacity`
- **Called by:** `PartFs::resolve_rel_target` (opc/mod.rs:600); `Parser::parse_element` (xmllinq/parse.rs:505); `xml_lexical_boundary_contract_tests::undecodable_character_references_remain_literal_and_uppercase_hex_decodes` (xmllinq/parse.rs:927)

#### `decode_entity`  (L732-757)
`fn decode_entity(body: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.collect`, `.is_ascii_alphabetic`, `.is_empty`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.to_string`, `Some`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `unescape_xml_text` (xmllinq/parse.rs:704); `validate_xml` (xmllinq/parse.rs:792)

#### `validate_xml`  (L792-848, pub)
`pub fn validate_xml(xml: &str) -> Result<(), String>`

- **Does:** Structural admission for a part before the permissive DOM parser reads it: well-formed nesting, a single root, no DTD, only the five predefined or numeric entities, and nesting no deeper than 256.
- **Calls (crate):** `TagPolicy::new` (xmllinq/parse.rs:211) L799; `TagPolicy::open` (xmllinq/parse.rs:236) L808; `TagPolicy::empty` (xmllinq/parse.rs:248) L818; `TagPolicy::close` (xmllinq/parse.rs:257) L825; `decode_entity` (xmllinq/parse.rs:732) L830
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.checked_sub`, `.config_mut`, `.is_ascii_whitespace`, `.is_none`, `.map_err`, `.ok_or`, `.read_event`, `.to_string`, `Err`, `Ok`, `Reader::from_str`
- **Macros:** `format!`
- **Called by:** `parse` (append.rs:330); `CommentFamily::load` (comments.rs:429); `package_tree` (debug/diff.rs:303); `document_text_view` (debug.rs:2011); `load_stories` (edit.rs:1311); `parse_part` (inspect.rs:693); `try_parse_xdocument` (xmllinq/parse.rs:122); `checked_xml_tests::truncated_attribute_input_terminates_in_legacy_parser` (xmllinq/parse.rs:856); `checked_xml_tests::checked_xml_keeps_refusing_attributes_that_are_not_xml` (xmllinq/parse.rs:876); `checked_xml_tests::checked_xml_rejects_invalid_or_unbounded_structure` (xmllinq/parse.rs:886); `checked_xml_tests::checked_xml_accepts_normal_documents_and_entities` (xmllinq/parse.rs:910); `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:946); +7 more

## Test functions (21)

- `bump_ns_scope_declared_bytes` (L86): Add to this thread's [`NS_SCOPE_DECLARED_BYTES`].
- `checked_xml_tests::truncated_attribute_input_terminates_in_legacy_parser` (L856): _no doc comment_
- `checked_xml_tests::checked_xml_keeps_refusing_attributes_that_are_not_xml` (L876): Package admission tolerates unquoted and valueless attributes so the permissive parser can read them; the strict gate never did, and the edit, comments, append and inspect paths still refuse such a part.
- `checked_xml_tests::checked_xml_rejects_invalid_or_unbounded_structure` (L886): _no doc comment_
- `checked_xml_tests::checked_xml_accepts_normal_documents_and_entities` (L910): _no doc comment_
- `xml_lexical_boundary_contract_tests::undecodable_character_references_remain_literal_and_uppercase_hex_decodes` (L927): _no doc comment_
- `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (L946): _no doc comment_
- `tag_policy_tests::xml_with_attrs` (L996): _no doc comment_
- `tag_policy_tests::element_attribute_count_is_capped_at_the_boundary` (L1004): _no doc comment_
- `tag_policy_tests::tag_whole_width_is_bounded` (L1025): _no doc comment_
- `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (L1046): Word writes a VML `o:gfxdata` attribute of up to ~366 KiB inside `word/header*.xml` parts of its own documents; the checked gate (N01) must admit the real attribute size Word uses, not a value the engine happens to like.
- `tag_policy_tests::a_value_at_the_attribute_byte_cap_is_admitted_one_more_refused` (L1083): The exact-boundary contract for the new value cap: a value byte at `MAX_ATTRIBUTE_VALUE_BYTES` is admitted (it is the inclusive cap); one byte more is refused with the gate's structured error.
- `tag_policy_tests::a_tag_at_the_attribute_byte_cap_is_admitted_one_more_refused` (L1111): The exact-boundary contract for the new tag-byte cap: attribute bytes totalling exactly `MAX_TAG_ATTRIBUTE_BYTES` are admitted; one byte more is refused while every individual value stays under the value cap.
- `tag_policy_tests::namespace_declaration_budgets_reject_before_dom_insertion` (L1151): _no doc comment_
- `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (L1189): _no doc comment_
- `tag_policy_tests::parsed_attributes_insert_in_bulk_not_quadratic_scans` (L1203): _no doc comment_
- `tag_policy_tests::duplicate_expanded_names_keep_declaration_order_and_last_value_wins` (L1237): _no doc comment_
- `tag_policy_tests::nested_rebinding_shadows_and_restores` (L1261): _no doc comment_
- `tag_policy_tests::deep_rebinding_scope_grows_only_with_declared_bytes` (L1285): _no doc comment_
- `tag_policy_tests::normal_documents_parse_unchanged_through_try_parse` (L1320): _no doc comment_
- `tag_policy_tests::mechanism_counters_are_isolated_to_the_calling_thread` (L1346): `cargo test` runs tests as threads of one process, so a shared counter would see every other test's parsing.
