# Functions and methods in `src/xmllinq/parse.rs`

41 non-test, 35 test. Part of the [symbol manifest](../README.md).

#### `parse_xdocument`  (L100-111, pub)
`pub fn parse_xdocument(dom: &mut Dom, xml: &str) -> NodeId`

- **Does:** Parse an XML string into a Document node allocated into `dom`.
- **Calls (crate):** `Parser::parse_document` (xmllinq/parse.rs:654) L110
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `Dom::parse_xdocument` (xmllinq/mod.rs:1299); `try_parse_xdocument_with_limits` (xmllinq/parse.rs:131); `depth_scan_tests::built_depth` (xmllinq/parse.rs:1116); `checked_xml_tests::truncated_attribute_input_terminates_in_legacy_parser` (xmllinq/parse.rs:1189); `checked_xml_tests::checked_xml_keeps_refusing_attributes_that_are_not_xml` (xmllinq/parse.rs:1209); `xml_lexical_boundary_contract_tests::literal_attribute_whitespace_reads_as_spaces_references_survive` (xmllinq/parse.rs:1279); `xml_lexical_boundary_contract_tests::literal_text_end_of_line_reads_as_lf_references_survive` (xmllinq/parse.rs:1303); `xml_lexical_boundary_contract_tests::literal_whitespace_sources_round_trip_to_the_same_dom_values` (xmllinq/parse.rs:1324); `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:1351); `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (xmllinq/parse.rs:1575); `tag_policy_tests::parsed_attributes_insert_in_bulk_not_quadratic_scans` (xmllinq/parse.rs:1589); `tag_policy_tests::duplicate_expanded_names_refuse_only_in_the_checked_parser` (xmllinq/parse.rs:1623); +5 more

#### `try_parse_xdocument`  (L121-123, pub)
`pub fn try_parse_xdocument(dom: &mut Dom, xml: &str) -> Result<NodeId, String>`

- **Does:** Checked parse: enforce [`validate_xml`] (well-formedness plus the tag and namespace budgets) before the permissive parser builds the DOM.
- **Calls (crate):** `try_parse_xdocument_with_limits` (xmllinq/parse.rs:131) L122; `<InputLimits as Default>::default` (admission.rs:64) L122
- **Called by:** `Dom::try_parse_xdocument` (xmllinq/mod.rs:1310); `tag_policy_tests::element_attribute_count_is_capped_at_the_boundary` (xmllinq/parse.rs:1409); `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (xmllinq/parse.rs:1451); `tag_policy_tests::a_value_at_the_attribute_byte_cap_is_admitted_one_more_refused` (xmllinq/parse.rs:1488); `tag_policy_tests::duplicate_expanded_names_refuse_only_in_the_checked_parser` (xmllinq/parse.rs:1623); `tag_policy_tests::issue650_wide_tag_preserves_names_values_and_order` (xmllinq/parse.rs:1687); `tag_policy_tests::issue650_attribute_name_byte_boundary` (xmllinq/parse.rs:1704); `tag_policy_tests::issue650_checked_dom_parser_enforces_width_limits` (xmllinq/parse.rs:1713); `tag_policy_tests::issue650_unprefixed_attribute_has_no_default_namespace` (xmllinq/parse.rs:1726); `tag_policy_tests::issue650_attribute_probes_scale_linearly` (xmllinq/parse.rs:1736); `tag_policy_tests::normal_documents_parse_unchanged_through_try_parse` (xmllinq/parse.rs:1812)

#### `try_parse_xdocument_with_limits`  (L131-138, pub)
`pub fn try_parse_xdocument_with_limits( dom: &mut Dom, xml: &str, limits: InputLimits, ) -> Result<NodeId, String>`

- **Does:** Checked parse under explicit input budgets.
- **Calls (crate):** `validate_xml_with_limits` (xmllinq/parse.rs:1052) L136; `parse_xdocument` (xmllinq/parse.rs:100) L137
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `try_parse_xdocument` (xmllinq/parse.rs:121)

#### `nests_deeper`  (L151-160, pub(crate))
`pub(crate) fn nests_deeper(xml: &str, limit: usize) -> bool`

- **Does:** Whether `xml` nests deeper than `limit` elements, read the way [`parse_xdocument`] reads it.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L153; `Parser::nests_deeper` (xmllinq/parse.rs:579) L159
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `tolerant` (changes.rs:620) (?); `depth_scan_tests::scanned_depth` (xmllinq/parse.rs:1130); `depth_scan_tests::the_scan_counts_markup_the_permissive_parser_nests` (xmllinq/parse.rs:1171)

#### `NsScope::get`  (L189-191)
`fn get(&self, prefix: &str) -> Option<&String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `TagPolicy::expand` (xmllinq/parse.rs:430); `resolve` (xmllinq/parse.rs:837)

#### `NsScope::mark`  (L193-195)
`fn mark(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `story_part` (markdown/view_stories.rs:1710) (?); `TagPolicy::enter` (xmllinq/parse.rs:277); `TagPolicy::empty` (xmllinq/parse.rs:295); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `NsScope::push`  (L198-206)
`fn push(&mut self, prefix: &str, uri: &str)`

- **Does:** Record `prefix → uri` as a declaration owned by the current element.
- **Calls (crate):** `bump_ns_scope_declared_bytes` (xmllinq/parse.rs:80) L205
- **Calls (std/external/unresolved):** `.insert`, `.len`, `.push`, `.to_string`
- **Called by:** `TagPolicy::declare` (xmllinq/parse.rs:400); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `NsScope::pop`  (L209-225)
`fn pop(&mut self, mark: usize)`

- **Does:** Restore the scope to `mark`, undoing every push since then.
- **Calls (std/external/unresolved):** `.insert`, `.len`, `.pop`, `.remove`, `.saturating_sub`
- **Called by:** `TagPolicy::enter` (xmllinq/parse.rs:277); `TagPolicy::empty` (xmllinq/parse.rs:295); `TagPolicy::close` (xmllinq/parse.rs:306); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `TagPolicy::new`  (L247-255, pub(crate))
`pub(crate) fn new(limits: InputLimits) -> Self`

- **Does:** The strict gate's policy: every attribute must be well-formed XML, quoted and with a value.
- **Calls (std/external/unresolved):** `NsScope::default`, `Vec::new`
- **Called by:** `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444); `TagPolicy::tolerant` (xmllinq/parse.rs:261); `validate_xml_with_limits` (xmllinq/parse.rs:1052)

#### `TagPolicy::tolerant`  (L261-266, pub(crate))
`pub(crate) fn tolerant(limits: InputLimits) -> Self`

- **Does:** The package admission scan's policy.
- **Calls (crate):** `TagPolicy::new` (xmllinq/parse.rs:247) L264
- **Called by:** `check_xml_depth` (admission.rs:858); `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61)

#### `TagPolicy::open`  (L273-275, pub(crate))
`pub(crate) fn open(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String>`

- **Does:** Check one open tag and push its declarations until [`Self::close`].
- **Calls (crate):** `TagPolicy::enter` (xmllinq/parse.rs:277) L274
- **Called by:** `check_xml_depth` (admission.rs:858); `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61); `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444); `validate_xml_with_limits` (xmllinq/parse.rs:1052)

#### `TagPolicy::enter`  (L277-288)
`fn enter( &mut self, tag: &quick_xml::events::BytesStart<'_>, delimiters: usize, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `NsScope::mark` (xmllinq/parse.rs:193) L282; `TagPolicy::check_tag` (xmllinq/parse.rs:312) L283; `NsScope::pop` (xmllinq/parse.rs:209) L285
- **Calls (std/external/unresolved):** `.as_ref`, `.inspect_err`, `.map_err`, `.name`, `.push`, `Ok`
- **Macros:** `format!`
- **Called by:** `TagPolicy::open` (xmllinq/parse.rs:273)

#### `TagPolicy::empty`  (L295-302, pub(crate))
`pub(crate) fn empty(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String>`

- **Does:** Check a self-closing tag; its declarations never outlive the tag.
- **Calls (crate):** `NsScope::mark` (xmllinq/parse.rs:193) L296; `TagPolicy::check_tag` (xmllinq/parse.rs:312) L297; `NsScope::pop` (xmllinq/parse.rs:209) L300
- **Calls (std/external/unresolved):** `.as_ref`, `.map_err`, `.name`
- **Macros:** `format!`
- **Called by:** `check_xml_depth` (admission.rs:858); `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61); `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444); `validate_xml_with_limits` (xmllinq/parse.rs:1052)

#### `TagPolicy::close`  (L306-310, pub(crate))
`pub(crate) fn close(&mut self)`

- **Does:** Pop the declarations of the innermost open element.
- **Calls (crate):** `NsScope::pop` (xmllinq/parse.rs:209) L308
- **Calls (std/external/unresolved):** `.pop`
- **Called by:** `check_xml_depth` (admission.rs:858); `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61); `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444); `validate_xml_with_limits` (xmllinq/parse.rs:1052)

#### `TagPolicy::check_tag`  (L312-369)
`fn check_tag( &mut self, tag: &quick_xml::events::BytesStart<'_>, delimiters: usize, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `TagPolicy::check_attribute` (xmllinq/parse.rs:372) L345,351; `TagPolicy::declare` (xmllinq/parse.rs:400) L353,355; `TagPolicy::expand` (xmllinq/parse.rs:430) L364
- **Calls (std/external/unresolved):** `.as_ref`, `.attributes`, `.html_attributes`, `.insert`, `.len`, `.map_err`, `.name`, `.normalized_value`, `.push`, `.saturating_add`, `.strip_prefix`, `.to_string`, `Err`, `Ok`, `Vec::new`, `std::collections::HashSet::with_capacity`
- **Macros:** `format!`
- **Called by:** `TagPolicy::enter` (xmllinq/parse.rs:277); `TagPolicy::empty` (xmllinq/parse.rs:295)

#### `TagPolicy::check_attribute`  (L372-397)
`fn check_attribute( &self, count: usize, name_bytes: usize, value_bytes: usize, ) -> Result<(), String>`

- **Does:** Enforce the active width budgets at the checked boundary.
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `TagPolicy::check_tag` (xmllinq/parse.rs:312)

#### `TagPolicy::declare`  (L400-424)
`fn declare(&mut self, prefix: &str, uri: &str) -> Result<(), String>`

- **Does:** Push one `xmlns` declaration under the namespace budgets.
- **Calls (crate):** `NsScope::push` (xmllinq/parse.rs:198) L422
- **Calls (std/external/unresolved):** `.len`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `TagPolicy::check_tag` (xmllinq/parse.rs:312)

#### `TagPolicy::expand`  (L430-441)
`fn expand<'a>(&self, key: &'a str) -> (String, &'a str)`

- **Does:** Expand an attribute name the way the DOM parser's `resolve` does: `xmlns:*` and `xml:*` map to their reserved URIs, other prefixes go through the in-scope bindings (unbound → empty), and unprefixed attributes never take the default namespace.
- **Calls (crate):** `NsScope::get` (xmllinq/parse.rs:189) L436
- **Calls (std/external/unresolved):** `.find`, `.map_or`, `.to_string`, `String::new`
- **Called by:** `TagPolicy::check_tag` (xmllinq/parse.rs:312)

#### `Parser::len`  (L445-447)
`fn len(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `Parser::cur` (xmllinq/parse.rs:449); `Parser::starts_with` (xmllinq/parse.rs:460); `Parser::index_of` (xmllinq/parse.rs:472); `Parser::slice` (xmllinq/parse.rs:497); `Parser::skip_ws` (xmllinq/parse.rs:505); `Parser::name_span` (xmllinq/parse.rs:518); `Parser::start_tag` (xmllinq/parse.rs:534); `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709); `Parser::pi_spans` (xmllinq/parse.rs:817)

#### `Parser::cur`  (L449-455)
`fn cur(&self) -> char`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L450
- **Called by:** `Parser::skip_ws` (xmllinq/parse.rs:505); `Parser::name_span` (xmllinq/parse.rs:518); `Parser::start_tag` (xmllinq/parse.rs:534); `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709); `Parser::pi_spans` (xmllinq/parse.rs:817)

#### `Parser::starts_with`  (L460-468)
`fn starts_with(&self, s: &str) -> bool`

- **Does:** True if the buffer at `pos` matches `s` (char-for-char).
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L463
- **Calls (std/external/unresolved):** `.chars`, `.enumerate`
- **Called by:** `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Parser::index_of`  (L472-495)
`fn index_of(&self, s: &str, from: usize) -> Option<usize>`

- **Does:** Index of substring `s` at or after `from`, in char units.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L473,478,480
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.enumerate`, `.is_empty`, `.saturating_sub`, `Some`
- **Called by:** `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709); `Parser::pi_spans` (xmllinq/parse.rs:817)

#### `Parser::slice`  (L497-503)
`fn slice(&self, start: usize, end: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L498
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.min`, `String::new`
- **Called by:** `Parser::read_name` (xmllinq/parse.rs:512); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709) (?); `Parser::parse_pi_node` (xmllinq/parse.rs:808)

#### `Parser::skip_ws`  (L505-509)
`fn skip_ws(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L506; `Parser::cur` (xmllinq/parse.rs:449) L506
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `Parser::start_tag` (xmllinq/parse.rs:534); `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Parser::read_name`  (L512-515)
`fn read_name(&mut self) -> String`

- **Does:** readName — consume until whitespace, '/', '>', or '='.
- **Calls (crate):** `Parser::name_span` (xmllinq/parse.rs:518) L513; `Parser::slice` (xmllinq/parse.rs:497) L514
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Parser::name_span`  (L518-528)
`fn name_span(&mut self) -> (usize, usize)`

- **Does:** The span [`Self::read_name`] consumes.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L520; `Parser::cur` (xmllinq/parse.rs:449) L521
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `Parser::read_name` (xmllinq/parse.rs:512); `Parser::start_tag` (xmllinq/parse.rs:534); `Parser::nests_deeper` (xmllinq/parse.rs:579)

#### `Parser::start_tag`  (L534-574)
`fn start_tag( &mut self, mut attribute: impl FnMut(&Self, (usize, usize), (usize, usize)), ) -> ((usize, usize), bool)`

- **Does:** A start tag from its `<` through `>` or `/>`: the raw name's span, each attribute's raw name and value spans handed to `attribute` in order, and whether the tag closed itself.
- **Calls (crate):** `Parser::name_span` (xmllinq/parse.rs:518) L539,545; `Parser::skip_ws` (xmllinq/parse.rs:505) L541,551,555,567; `Parser::len` (xmllinq/parse.rs:445) L542,559; `Parser::cur` (xmllinq/parse.rs:449) L542,553,556,559,568; `attribute` (markdown/unified.rs:756) (by name) L565
- **Called by:** `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Parser::nests_deeper`  (L579-652)
`fn nests_deeper(&mut self, limit: usize) -> bool`

- **Does:** [`nests_deeper`]'s walk: [`Self::parse_document`]'s loop at the top level, [`Self::parse_element`]'s inside an element, with a depth count where they recurse.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L580; `Parser::skip_ws` (xmllinq/parse.rs:505) L584,617; `Parser::starts_with` (xmllinq/parse.rs:460) L585,598,602,606,614,624...; `Parser::index_of` (xmllinq/parse.rs:472) L593,603,607,625,629; `Parser::pi_spans` (xmllinq/parse.rs:817) L599,633; `Parser::cur` (xmllinq/parse.rs:449) L610,618,636,637; `Parser::name_span` (xmllinq/parse.rs:518) L616; `Parser::start_tag` (xmllinq/parse.rs:534) L643
- **Calls (std/external/unresolved):** `.get`, `.is_none_or`, `.is_whitespace`, `.map_or`, `.map`, `.unwrap_or`
- **Called by:** `nests_deeper` (xmllinq/parse.rs:151)

#### `Parser::parse_document`  (L654-707)
`fn parse_document(&mut self) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_document` (xmllinq/mod.rs:472) L655; `Parser::len` (xmllinq/parse.rs:445) L659; `Parser::skip_ws` (xmllinq/parse.rs:505) L661; `Parser::starts_with` (xmllinq/parse.rs:460) L662,680,685,693; `Parser::index_of` (xmllinq/parse.rs:472) L672,686,695; `Parser::slice` (xmllinq/parse.rs:497) L673,687; `parse_declaration` (xmllinq/parse.rs:864) L674; `Dom::set_declaration` (xmllinq/mod.rs:589) L675; `Parser::parse_pi_node` (xmllinq/parse.rs:808) L681; `Dom::add` (xmllinq/mod.rs:1074) L682,689,701; `Dom::new_comment` (xmllinq/mod.rs:487) L688; `Parser::cur` (xmllinq/parse.rs:449) L699; `Parser::parse_element` (xmllinq/parse.rs:709) L700
- **Calls (std/external/unresolved):** `.get`, `.is_none_or`, `.is_whitespace`, `.map`, `.unwrap_or`, `NsScope::default`, `Some`
- **Called by:** `parse_xdocument` (xmllinq/parse.rs:100)

#### `Parser::parse_element`  (L709-804)
`fn parse_element(&mut self, scope: &mut NsScope) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::start_tag` (xmllinq/parse.rs:534) L712; `Parser::slice` (xmllinq/parse.rs:497) (by name) L717,718; `unescape_xml_text` (xmllinq/parse.rs:897) L718,798; `normalize_raw_attribute_whitespace` (xmllinq/parse.rs:961) L718; `Parser::slice` (xmllinq/parse.rs:497) L721,764,772,796; `NsScope::mark` (xmllinq/parse.rs:193) L727; `NsScope::push` (xmllinq/parse.rs:198) L730,732; `resolve` (xmllinq/parse.rs:837) L736,742; `Dom::new_element` (xmllinq/mod.rs:478) L737; `Dom::set_attributes` (xmllinq/mod.rs:935) L744; `NsScope::pop` (xmllinq/parse.rs:209) L747,802; `Parser::len` (xmllinq/parse.rs:445) L751; `Parser::starts_with` (xmllinq/parse.rs:460) L753,762,770,780; `Parser::read_name` (xmllinq/parse.rs:512) L755; `Parser::skip_ws` (xmllinq/parse.rs:505) L756; `Parser::cur` (xmllinq/parse.rs:449) L757,785,791; `Parser::index_of` (xmllinq/parse.rs:472) L763,771; `Dom::new_comment` (xmllinq/mod.rs:487) L765; `Dom::add` (xmllinq/mod.rs:1074) L766,776,782,787,799; `Dom::new_text` (xmllinq/mod.rs:483) L775,798; `normalize_raw_text_end_of_line` (xmllinq/parse.rs:991) L775,796; `Parser::parse_pi_node` (xmllinq/parse.rs:808) L781; `Parser::parse_element` (xmllinq/parse.rs:709) L786
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.is_empty`, `.iter`, `.map`, `.push`, `.strip_prefix`, `.unwrap_or`, `Vec::new`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Parser::parse_pi_node`  (L808-813)
`fn parse_pi_node(&mut self) -> NodeId`

- **Does:** Parse a processing instruction (`<?target data?>`) into a `Pi` node.
- **Calls (crate):** `Parser::pi_spans` (xmllinq/parse.rs:817) L809; `Parser::slice` (xmllinq/parse.rs:497) L810,811; `Dom::new_pi` (xmllinq/mod.rs:492) L812
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Parser::pi_spans`  (L817-832)
`fn pi_spans(&mut self) -> (usize, usize, usize)`

- **Does:** Consume a processing instruction (`self.pos` on `<?`): where its target starts and ends, and where its data ends.
- **Calls (crate):** `Parser::len` (xmllinq/parse.rs:445) L818; `Parser::cur` (xmllinq/parse.rs:449) L822; `Parser::index_of` (xmllinq/parse.rs:472) L829
- **Calls (std/external/unresolved):** `.is_whitespace`, `.map`, `.unwrap_or`
- **Called by:** `Parser::nests_deeper` (xmllinq/parse.rs:579); `Parser::parse_pi_node` (xmllinq/parse.rs:808)

#### `resolve`  (L837-861)
`fn resolve(qn: &str, is_attr: bool, local_scope: &NsScope) -> XName`

- **Does:** Resolve a qualified name against the in-scope namespaces.
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L842; `XNamespace::xml` (xmllinq/mod.rs:128) L845; `NsScope::get` (xmllinq/parse.rs:189) L847,856; `XNamespace::get` (xmllinq/mod.rs:108) L848,858; `XName::get` (xmllinq/mod.rs:180) L849,853,859
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.name`
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:709)

#### `parse_declaration`  (L864-870)
`fn parse_declaration(decl: &str) -> XDeclaration`

- **Does:** Parse the `<?xml ...?>` declaration body.
- **Calls (crate):** `extract_pseudo_attr` (xmllinq/parse.rs:873) L866,867,868
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`, `Some`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:654)

#### `extract_pseudo_attr`  (L873-885)
`fn extract_pseudo_attr(s: &str, key: &str) -> Option<String>`

- **Does:** Extract `key="value"` / `key='value'` from a declaration string.
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.len`, `.next`, `.to_string`, `.trim_start`, `Some`
- **Called by:** `parse_declaration` (xmllinq/parse.rs:864)

#### `unescape_xml_text`  (L897-923, pub)
`pub fn unescape_xml_text(s: &str) -> String`

- **Does:** Decode XML entity and character references in attribute and text payloads (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&#nnn;`, `&#xhh;`).
- **Calls (crate):** `decode_entity` (xmllinq/parse.rs:925) L909
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.iter`, `.len`, `.position`, `.push_str`, `.push`, `.to_string`, `String::with_capacity`
- **Called by:** `PartFs::resolve_rel_target` (opc/mod.rs:829); `Parser::parse_element` (xmllinq/parse.rs:709); `xml_lexical_boundary_contract_tests::undecodable_character_references_remain_literal_and_uppercase_hex_decodes` (xmllinq/parse.rs:1260)

#### `decode_entity`  (L925-950)
`fn decode_entity(body: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.collect`, `.is_ascii_alphabetic`, `.is_empty`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.to_string`, `Some`, `char::from_u32`, `u32::from_str_radix`
- **Called by:** `unescape_xml_text` (xmllinq/parse.rs:897); `validate_xml_with_limits` (xmllinq/parse.rs:1052)

#### `normalize_raw_attribute_whitespace`  (L961-981)
`fn normalize_raw_attribute_whitespace(value: String) -> String`

- **Does:** XML 1.0 §3.3.3 attribute-value normalization, applied to the RAW value slice *before* [`unescape_xml_text`]: §2.11 end-of-line handling folds a literal CRLF (and a lone CR) to one LF first, and every literal whitespace character then reads as one space — the value a conforming reader (Word) sees.
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.contains`, `.len`, `.next`, `.push`, `.starts_with`, `String::with_capacity`
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:709)

#### `normalize_raw_text_end_of_line`  (L991-1008)
`fn normalize_raw_text_end_of_line(text: String) -> String`

- **Does:** XML 1.0 §2.11 end-of-line handling, applied to the RAW text slice (CDATA bodies included — the processor normalizes line breaks before parsing) *before* [`unescape_xml_text`]: a literal CRLF and a lone CR both read as one LF.
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.contains`, `.len`, `.next`, `.push`, `.starts_with`, `String::with_capacity`
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:709)

#### `validate_xml`  (L1043-1045, pub)
`pub fn validate_xml(xml: &str) -> Result<(), String>`

- **Does:** Structural admission for a part before the permissive DOM parser reads it: well-formed nesting, a single root, no DTD, only the five predefined or numeric entities, and nesting no deeper than 256.
- **Calls (crate):** `validate_xml_with_limits` (xmllinq/parse.rs:1052) L1044; `<InputLimits as Default>::default` (admission.rs:64) L1044
- **Called by:** `parse` (append.rs:336); `CommentFamily::load` (comments.rs:457); `package_tree` (debug/diff.rs:303); `document_text_view` (debug.rs:2181); `load_stories` (edit.rs:1680); `checked_xml_tests::truncated_attribute_input_terminates_in_legacy_parser` (xmllinq/parse.rs:1189); `checked_xml_tests::checked_xml_keeps_refusing_attributes_that_are_not_xml` (xmllinq/parse.rs:1209); `checked_xml_tests::checked_xml_rejects_invalid_or_unbounded_structure` (xmllinq/parse.rs:1219); `checked_xml_tests::checked_xml_accepts_normal_documents_and_entities` (xmllinq/parse.rs:1243); `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:1351); `tag_policy_tests::element_attribute_count_is_capped_at_the_boundary` (xmllinq/parse.rs:1409); `tag_policy_tests::tag_whole_width_is_bounded` (xmllinq/parse.rs:1430); +7 more

#### `validate_xml_with_limits`  (L1052-1108, pub)
`pub fn validate_xml_with_limits(xml: &str, limits: InputLimits) -> Result<(), String>`

- **Does:** Validate XML under explicit XML-related [`InputLimits`].
- **Calls (crate):** `TagPolicy::new` (xmllinq/parse.rs:247) L1059; `TagPolicy::open` (xmllinq/parse.rs:273) L1068; `TagPolicy::empty` (xmllinq/parse.rs:295) L1078; `TagPolicy::close` (xmllinq/parse.rs:306) L1085; `decode_entity` (xmllinq/parse.rs:925) L1090
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.checked_sub`, `.config_mut`, `.is_ascii_whitespace`, `.is_none`, `.map_err`, `.ok_or`, `.read_event`, `.to_string`, `Err`, `Ok`, `Reader::from_str`
- **Macros:** `format!`
- **Called by:** `parse_part_within` (inspect.rs:930); `try_parse_xdocument_with_limits` (xmllinq/parse.rs:131); `validate_xml` (xmllinq/parse.rs:1043)

## Test functions (35)

- `bump_ns_scope_declared_bytes` (L80): Add to this thread's [`NS_SCOPE_DECLARED_BYTES`].
- `depth_scan_tests::built_depth` (L1116): The deepest element of the tree `parse_xdocument` builds.
- `depth_scan_tests::scanned_depth` (L1130): The deepest nesting the scan reports: the least limit it keeps within.
- `depth_scan_tests::the_scan_measures_the_tree_the_parser_builds` (L1141): The scan measures the tree the parser builds, malformed input included: the two cannot drift apart unnoticed.
- `depth_scan_tests::the_scan_counts_markup_the_permissive_parser_nests` (L1171): Inside an element the permissive parser reads `<!x>` as a child element, so the scan that guards its recursion must count it too (#1353 review): a strict reader stops at it.
- `checked_xml_tests::truncated_attribute_input_terminates_in_legacy_parser` (L1189): _no doc comment_
- `checked_xml_tests::checked_xml_keeps_refusing_attributes_that_are_not_xml` (L1209): Package admission tolerates unquoted and valueless attributes so the permissive parser can read them; the strict gate never did, and the edit, comments, append and inspect paths still refuse such a part.
- `checked_xml_tests::checked_xml_rejects_invalid_or_unbounded_structure` (L1219): _no doc comment_
- `checked_xml_tests::checked_xml_accepts_normal_documents_and_entities` (L1243): _no doc comment_
- `xml_lexical_boundary_contract_tests::undecodable_character_references_remain_literal_and_uppercase_hex_decodes` (L1260): _no doc comment_
- `xml_lexical_boundary_contract_tests::literal_attribute_whitespace_reads_as_spaces_references_survive` (L1279): _no doc comment_
- `xml_lexical_boundary_contract_tests::literal_text_end_of_line_reads_as_lf_references_survive` (L1303): _no doc comment_
- `xml_lexical_boundary_contract_tests::literal_whitespace_sources_round_trip_to_the_same_dom_values` (L1324): _no doc comment_
- `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (L1351): _no doc comment_
- `tag_policy_tests::xml_with_attrs` (L1401): _no doc comment_
- `tag_policy_tests::element_attribute_count_is_capped_at_the_boundary` (L1409): _no doc comment_
- `tag_policy_tests::tag_whole_width_is_bounded` (L1430): _no doc comment_
- `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (L1451): Word writes a VML `o:gfxdata` attribute of up to ~366 KiB inside `word/header*.xml` parts of its own documents; the checked gate (N01) must admit the real attribute size Word uses, not a value the engine happens to like.
- `tag_policy_tests::a_value_at_the_attribute_byte_cap_is_admitted_one_more_refused` (L1488): The exact-boundary contract for the new value cap: a value byte at `MAX_ATTRIBUTE_VALUE_BYTES` is admitted (it is the inclusive cap); one byte more is refused with the gate's structured error.
- `tag_policy_tests::a_tag_at_the_attribute_byte_cap_is_admitted_one_more_refused` (L1512): The complete start tag, including spaces, quotes and delimiters, has an inclusive byte cap independent of each value's cap.
- `tag_policy_tests::namespace_declaration_budgets_reject_before_dom_insertion` (L1537): _no doc comment_
- `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (L1575): _no doc comment_
- `tag_policy_tests::parsed_attributes_insert_in_bulk_not_quadratic_scans` (L1589): _no doc comment_
- `tag_policy_tests::duplicate_expanded_names_refuse_only_in_the_checked_parser` (L1623): _no doc comment_
- `tag_policy_tests::issue650_infallible_parser_preserves_over_default_count` (L1648): _no doc comment_
- `tag_policy_tests::issue650_infallible_parser_preserves_over_default_name` (L1669): _no doc comment_
- `tag_policy_tests::issue650_wide_tag_preserves_names_values_and_order` (L1687): _no doc comment_
- `tag_policy_tests::issue650_attribute_name_byte_boundary` (L1704): _no doc comment_
- `tag_policy_tests::issue650_checked_dom_parser_enforces_width_limits` (L1713): _no doc comment_
- `tag_policy_tests::issue650_unprefixed_attribute_has_no_default_namespace` (L1726): _no doc comment_
- `tag_policy_tests::issue650_attribute_probes_scale_linearly` (L1736): _no doc comment_
- `tag_policy_tests::nested_rebinding_shadows_and_restores` (L1753): _no doc comment_
- `tag_policy_tests::deep_rebinding_scope_grows_only_with_declared_bytes` (L1777): _no doc comment_
- `tag_policy_tests::normal_documents_parse_unchanged_through_try_parse` (L1812): _no doc comment_
- `tag_policy_tests::mechanism_counters_are_isolated_to_the_calling_thread` (L1838): `cargo test` runs tests as threads of one process, so a shared counter would see every other test's parsing.
