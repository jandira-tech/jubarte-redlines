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
- **Calls (crate):** `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L93; `status_key` (comparer/revisions.rs:40) L98; `Dom::new_element` (xmllinq/mod.rs:478) L103; `Dom::name` (xmllinq/mod.rs:526) L103; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L104,110; `XNamespace::xmlns` (xmllinq/mod.rs:122) L104; `Dom::attributes` (xmllinq/mod.rs:885) L105; `Dom::serialize_element` (xmllinq/mod.rs:1299) L112; `group_adjacent` (util/group_adjacent.rs:20) L116
- **Calls (std/external/unresolved):** `.clone`, `.expect`, `.len`, `.name`, `.push_str`, `.push`, `.to_string`, `.unwrap`, `PT::unid`, `Some`, `Vec::with_capacity`, `W::id`
- **Called by:** `get_revisions_from_body` (comparer/revisions.rs:151); `get_revisions_from_note_definitions` (comparer/revisions.rs:224)

#### `group_text`  (L121-138)
`fn group_text(dom: &Dom, group: &[(String, ComparisonUnitAtom)]) -> Option<String>`

- **Does:** Shared with D.3: the group's revision text — concatenated content values, `\n` for pPr atoms, None for the no-text content kinds.
- **Calls (crate):** `is_rev_element_with_no_text` (comparer/revisions.rs:32) L123; `Dom::name` (xmllinq/mod.rs:526) L130; `Dom::value_str` (xmllinq/mod.rs:1179) L133
- **Calls (std/external/unresolved):** `.collect`, `.into_owned`, `.iter`, `.map`, `.to_string`, `Some`, `W::p_pr`
- **Called by:** `get_revisions_from_body` (comparer/revisions.rs:151); `get_revisions_from_note_definitions` (comparer/revisions.rs:224)

#### `get_revisions_from_body`  (L151-214, pub)
`pub fn get_revisions_from_body( dom: &mut Dom, content_parent: NodeId, part_name: &str, settings: &WmlComparerSettings, ) -> Vec<WmlComparerRevision>`

- **Does:** D.2 — the main-part portion of `GetRevisions` (WmlComparer.cs:3952–4048): group the redline's atoms by their correlation status and tracking element, drop `Equal` groups, and map each remaining group to a [`WmlComparerRevision`].
- **Calls (crate):** `keyed_groups` (comparer/revisions.rs:88) L157; `group_text` (comparer/revisions.rs:121) L184; `Dom::attribute` (xmllinq/mod.rs:874) L185,186,205; `elements_before_self` (comparer/revisions.rs:69) L200; `Dom::name` (xmllinq/mod.rs:526) L203; `move_group_id_from_name` (comparer/revisions.rs:58) L208
- **Calls (std/external/unresolved):** `.expect`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.map`, `.push`, `.rev`, `.starts_with`, `.to_string`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::name`
- **Called by:** `get_revisions` (document_comparer.rs:6285)

#### `get_revisions_from_note_definitions`  (L224-266, pub)
`pub fn get_revisions_from_note_definitions( dom: &mut Dom, notes_root: NodeId, def_name: &crate::xmllinq::XName, part_name: &str, settings: &WmlComparerSettings, ) -> Vec<WmlComparerRevision>`

- **Does:** D.3 — `GetFootnoteEndnoteRevisionList` (WmlComparer.cs:4072): the same grouping run per note definition under the notes-part root.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L231; `keyed_groups` (comparer/revisions.rs:88) L234; `group_text` (comparer/revisions.rs:121) L253; `Dom::attribute` (xmllinq/mod.rs:874) L254,255
- **Calls (std/external/unresolved):** `.expect`, `.map`, `.push`, `.starts_with`, `.to_string`, `Some`, `Vec::new`, `W::author`, `W::date`
- **Called by:** `get_revisions` (document_comparer.rs:6285)

#### `get_text_from_ancestor_run`  (L270-283)
`fn get_text_from_ancestor_run(dom: &Dom, rpr_change: NodeId) -> String`

- **Does:** D.4 — `GetTextFromAncestorRun` (:4226): the direct `w:t` children of the nearest ancestor run ("" when there is none).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L271,280; `Dom::name` (xmllinq/mod.rs:526) L273; `Dom::elements` (xmllinq/mod.rs:640) L274; `Dom::value` (xmllinq/mod.rs:1169) L277
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Some`, `String::new`, `W::r`, `W::t`
- **Called by:** `get_format_change_revisions` (comparer/revisions.rs:362)

#### `get_property_value`  (L287-295)
`fn get_property_value(dom: &mut Dom, prop: NodeId) -> String`

- **Does:** D.4 — `GetPropertyValue` (:4294): the `w:val` value, "true" for a bare boolean property, or the serialized element for complex properties.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L288; `Dom::has_elements` (xmllinq/mod.rs:862) L291; `Dom::attributes` (xmllinq/mod.rs:885) L291; `Dom::serialize_element` (xmllinq/mod.rs:1299) L294
- **Calls (std/external/unresolved):** `.is_empty`, `.to_string`, `W::val`
- **Called by:** `extract_property_dictionary` (comparer/revisions.rs:299)

#### `extract_property_dictionary`  (L299-318)
`fn extract_property_dictionary( dom: &mut Dom, rpr: Option<NodeId>, ) -> std::collections::BTreeMap<String, String>`

- **Does:** D.4 — `ExtractPropertyDictionary` (:4275): friendly-name → value for every property child of an rPr (skipping the rPrChange itself).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L307; `Dom::name` (xmllinq/mod.rs:526) L309; `friendly_property_name` (comparer/formatchg.rs:162) L313; `XName::local_name` (xmllinq/mod.rs:208) (by name) L313; `get_property_value` (comparer/revisions.rs:287) L314
- **Calls (std/external/unresolved):** `.insert`, `.unwrap`, `W::name`, `std::collections::BTreeMap::new`
- **Called by:** `extract_format_change_details` (comparer/revisions.rs:323)

#### `extract_format_change_details`  (L323-351)
`fn extract_format_change_details( dom: &mut Dom, rpr_change: NodeId, ) -> super::atoms::FormatChangeInfo`

- **Does:** D.4 — `ExtractFormatChangeDetails` (:4240): old props from `rPrChange/w:rPr`, new props from the PARENT `w:rPr`; a property is changed when its presence or value differs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L327; `Dom::parent` (xmllinq/mod.rs:597) L328; `extract_property_dictionary` (comparer/revisions.rs:299) L329,330
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.dedup`, `.get`, `.keys`, `.push`, `.sort`, `Vec::new`, `W::r_pr`
- **Called by:** `get_format_change_revisions` (comparer/revisions.rs:362)

#### `get_format_change_revisions`  (L362-395, pub)
`pub fn get_format_change_revisions( dom: &mut Dom, parts: &[(NodeId, &str)], ) -> Vec<WmlComparerRevision>`

- **Does:** D.4 — `GetFormatChangeRevisions` (WmlComparer.cs:4152): every `w:rPrChange` under each given `(root, part_name)` becomes a `FormatChanged` revision.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L368; `Dom::attribute` (xmllinq/mod.rs:874) L370,374; `get_text_from_ancestor_run` (comparer/revisions.rs:270) L378; `extract_format_change_details` (comparer/revisions.rs:323) L379
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::name`
- **Called by:** `get_revisions` (document_comparer.rs:6285)

#### `detect_moves`  (L410-467, pub)
`pub fn detect_moves(revisions: &mut [WmlComparerRevision], settings: &WmlComparerSettings)`

- **Does:** D.5 — `DetectMoves` (WmlComparer.cs:4313): post-process the revision list, pairing each qualifying deletion with its best-matching unmatched insertion (word-level Jaccard ≥ [`super::WmlComparerSettings::move_similarity_threshold`], both ≥ [`super::WmlComparerSettings::move_minimum_word_count`]) into a `Moved` pair sha...
- **Calls (crate):** `count_words` (comparer/moves.rs:75) L418; `jaccard` (comparer/moves.rs:125) L446
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.trim`, `.unwrap_or`, `Some`, `qualifies`, `std::collections::HashSet::new`
- **Called by:** `get_revisions` (document_comparer.rs:6285)
