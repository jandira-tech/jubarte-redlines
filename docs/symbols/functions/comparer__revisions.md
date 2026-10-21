# Functions and methods in `src/comparer/revisions.rs`

14 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `is_rev_element_with_no_text`  (L32-37)
`fn is_rev_element_with_no_text(dom: &Dom, e: NodeId) -> bool`

- **Does:** `RevElementsWithNoText` (:3934) — content kinds whose revisions carry no text: `m:oMath`, `m:oMathPara`, `w:drawing`.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L33
- **Calls (std/external/unresolved):** `M::name`, `W::name`
- **Called by:** `group_text` (comparer/revisions.rs:121)

#### `status_key`  (L40-53)
`fn status_key(s: CorrelationStatus) -> &'static str`

- **Does:** `CorrelationStatus.ToString()` for the grouping key.
- **Calls:** none
- **Called by:** `keyed_groups` (comparer/revisions.rs:88)

#### `move_group_id_from_name`  (L58-65)
`fn move_group_id_from_name(name: &str) -> i32`

- **Does:** Stand-in for .NET `string.GetHashCode() & 0x7FFFFFFF` (:4034): FNV-1a 32-bit masked non-negative.
- **Calls (std/external/unresolved):** `.as_bytes`, `.wrapping_mul`, `u32::from`
- **Called by:** `get_revisions_from_body` (comparer/revisions.rs:151)

#### `elements_before_self`  (L69-83)
`fn elements_before_self(dom: &Dom, e: NodeId) -> Vec<NodeId>`

- **Does:** C# `ElementsBeforeSelf()`: the element siblings preceding `e` in document order.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L70; `Dom::nodes` (xmllinq/mod.rs:604) L74; `Dom::is_element` (xmllinq/mod.rs:502) L78
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `get_revisions_from_body` (comparer/revisions.rs:151)

#### `keyed_groups`  (L88-117)
`fn keyed_groups( dom: &mut Dom, content_parent: NodeId, settings: &WmlComparerSettings, ) -> Vec<(String, Vec<(String, ComparisonUnitAtom)>)>`

- **Does:** Shared with D.3: atomize the content, key each atom by status + the revision tracking element serialized WITHOUT `w:id`/`pt:Unid` (:3962), and group adjacent same-key atoms.
- **Calls (crate):** `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L93; `status_key` (comparer/revisions.rs:40) L98; `Dom::new_element` (xmllinq/mod.rs:478) L103; `Dom::name` (xmllinq/mod.rs:526) L103; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L104,110; `XNamespace::xmlns` (xmllinq/mod.rs:122) L104; `Dom::attributes` (xmllinq/mod.rs:899) L105; `Dom::serialize_element` (xmllinq/mod.rs:1315) L112; `group_adjacent` (util/group_adjacent.rs:20) L116
- **Calls (std/external/unresolved):** `.clone`, `.expect`, `.len`, `.name`, `.push_str`, `.push`, `.to_string`, `.unwrap`, `PT::unid`, `Some`, `Vec::with_capacity`, `W::id`
- **Called by:** `get_revisions_from_body` (comparer/revisions.rs:151); `get_revisions_from_note_definitions` (comparer/revisions.rs:226)

#### `group_text`  (L121-138)
`fn group_text(dom: &Dom, group: &[(String, ComparisonUnitAtom)]) -> Option<String>`

- **Does:** Shared with D.3: the group's revision text — concatenated content values, `\n` for pPr atoms, None for the no-text content kinds.
- **Calls (crate):** `is_rev_element_with_no_text` (comparer/revisions.rs:32) L123; `Dom::name` (xmllinq/mod.rs:526) L130; `Dom::value_str` (xmllinq/mod.rs:1195) L133
- **Calls (std/external/unresolved):** `.collect`, `.into_owned`, `.iter`, `.map`, `.to_string`, `Some`, `W::p_pr`
- **Called by:** `get_revisions_from_body` (comparer/revisions.rs:151); `get_revisions_from_note_definitions` (comparer/revisions.rs:226)

#### `get_revisions_from_body`  (L151-216, pub)
`pub fn get_revisions_from_body( dom: &mut Dom, content_parent: NodeId, part_name: &str, settings: &WmlComparerSettings, ) -> Vec<WmlComparerRevision>`

- **Does:** D.2 — the main-part portion of `GetRevisions` (WmlComparer.cs:3952–4048): group the redline's atoms by their correlation status and tracking element, drop `Equal` groups, and map each remaining group to a [`WmlComparerRevision`].
- **Calls (crate):** `keyed_groups` (comparer/revisions.rs:88) L157; `group_text` (comparer/revisions.rs:121) L184; `Dom::attribute` (xmllinq/mod.rs:874) L185,187,207; `revision_date` (changes.rs:1082) L186; `XName::get` (xmllinq/mod.rs:180) L187; `elements_before_self` (comparer/revisions.rs:69) L202; `Dom::name` (xmllinq/mod.rs:526) L205; `move_group_id_from_name` (comparer/revisions.rs:58) L210
- **Calls (std/external/unresolved):** `.expect`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.map`, `.push`, `.rev`, `.starts_with`, `.to_string`, `Some`, `Vec::new`, `W::author`, `W::name`
- **Called by:** `get_revisions` (document_comparer.rs:6293)

#### `get_revisions_from_note_definitions`  (L226-270, pub)
`pub fn get_revisions_from_note_definitions( dom: &mut Dom, notes_root: NodeId, def_name: &crate::xmllinq::XName, part_name: &str, settings: &WmlComparerSettings, ) -> Vec<WmlComparerRevision>`

- **Does:** D.3 — `GetFootnoteEndnoteRevisionList` (WmlComparer.cs:4072): the same grouping run per note definition under the notes-part root.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L233; `keyed_groups` (comparer/revisions.rs:88) L236; `group_text` (comparer/revisions.rs:121) L255; `Dom::attribute` (xmllinq/mod.rs:874) L256,258; `revision_date` (changes.rs:1082) L257; `XName::get` (xmllinq/mod.rs:180) L258
- **Calls (std/external/unresolved):** `.expect`, `.map`, `.push`, `.starts_with`, `.to_string`, `Some`, `Vec::new`, `W::author`
- **Called by:** `get_revisions` (document_comparer.rs:6293)

#### `get_text_from_ancestor_run`  (L274-287)
`fn get_text_from_ancestor_run(dom: &Dom, rpr_change: NodeId) -> String`

- **Does:** D.4 — `GetTextFromAncestorRun` (:4226): the direct `w:t` children of the nearest ancestor run ("" when there is none).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L275,284; `Dom::name` (xmllinq/mod.rs:526) L277; `Dom::elements` (xmllinq/mod.rs:640) L278; `Dom::value` (xmllinq/mod.rs:1185) L281
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Some`, `String::new`, `W::r`, `W::t`
- **Called by:** `get_format_change_revisions` (comparer/revisions.rs:366)

#### `get_property_value`  (L291-299)
`fn get_property_value(dom: &mut Dom, prop: NodeId) -> String`

- **Does:** D.4 — `GetPropertyValue` (:4294): the `w:val` value, "true" for a bare boolean property, or the serialized element for complex properties.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L292; `Dom::has_elements` (xmllinq/mod.rs:862) L295; `Dom::has_attributes` (xmllinq/mod.rs:867) L295; `Dom::serialize_element` (xmllinq/mod.rs:1315) L298
- **Calls (std/external/unresolved):** `.to_string`, `W::val`
- **Called by:** `extract_property_dictionary` (comparer/revisions.rs:303)

#### `extract_property_dictionary`  (L303-322)
`fn extract_property_dictionary( dom: &mut Dom, rpr: Option<NodeId>, ) -> std::collections::BTreeMap<String, String>`

- **Does:** D.4 — `ExtractPropertyDictionary` (:4275): friendly-name → value for every property child of an rPr (skipping the rPrChange itself).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L311; `Dom::name` (xmllinq/mod.rs:526) L313; `friendly_property_name` (comparer/formatchg.rs:162) L317; `XName::local_name` (xmllinq/mod.rs:208) (by name) L317; `get_property_value` (comparer/revisions.rs:291) L318
- **Calls (std/external/unresolved):** `.insert`, `.unwrap`, `W::name`, `std::collections::BTreeMap::new`
- **Called by:** `extract_format_change_details` (comparer/revisions.rs:327)

#### `extract_format_change_details`  (L327-355)
`fn extract_format_change_details( dom: &mut Dom, rpr_change: NodeId, ) -> super::atoms::FormatChangeInfo`

- **Does:** D.4 — `ExtractFormatChangeDetails` (:4240): old props from `rPrChange/w:rPr`, new props from the PARENT `w:rPr`; a property is changed when its presence or value differs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L331; `Dom::parent` (xmllinq/mod.rs:597) L332; `extract_property_dictionary` (comparer/revisions.rs:303) L333,334
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.dedup`, `.get`, `.keys`, `.push`, `.sort`, `Vec::new`, `W::r_pr`
- **Called by:** `get_format_change_revisions` (comparer/revisions.rs:366)

#### `get_format_change_revisions`  (L366-399, pub)
`pub fn get_format_change_revisions( dom: &mut Dom, parts: &[(NodeId, &str)], ) -> Vec<WmlComparerRevision>`

- **Does:** D.4 — `GetFormatChangeRevisions` (WmlComparer.cs:4152): every `w:rPrChange` under each given `(root, part_name)` becomes a `FormatChanged` revision.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L372; `Dom::attribute` (xmllinq/mod.rs:874) L374,379; `revision_date` (changes.rs:1082) L378; `XName::get` (xmllinq/mod.rs:180) L379; `get_text_from_ancestor_run` (comparer/revisions.rs:274) L382; `extract_format_change_details` (comparer/revisions.rs:327) L383
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::new`, `W::author`, `W::name`
- **Called by:** `get_revisions` (document_comparer.rs:6293)

#### `detect_moves`  (L414-471, pub)
`pub fn detect_moves(revisions: &mut [WmlComparerRevision], settings: &WmlComparerSettings)`

- **Does:** D.5 — `DetectMoves` (WmlComparer.cs:4313): post-process the revision list, pairing each qualifying deletion with its best-matching unmatched insertion (word-level Jaccard ≥ [`super::WmlComparerSettings::move_similarity_threshold`], both ≥ [`super::WmlComparerSettings::move_minimum_word_count`]) into a `Moved` pair sha...
- **Calls (crate):** `count_words` (comparer/moves.rs:75) L422; `jaccard` (comparer/moves.rs:125) L450
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.trim`, `.unwrap_or`, `Some`, `qualifies`, `std::collections::HashSet::new`
- **Called by:** `get_revisions` (document_comparer.rs:6293)
