# Functions and methods in `src/revision_processor/style_records.rs`

26 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `attribute_groups`  (L38-55)
`fn attribute_groups(local: &str) -> Option<&'static [&'static [&'static str]]>`

- **Does:** Attribute groups of the properties Word merges attribute by attribute; the attributes of one group are alternatives for one value.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `prop_slots` (revision_processor/style_records.rs:186); `drop_slot` (revision_processor/style_records.rs:758)

#### `Slot::name`  (L110-112)
`fn name(&self) -> XName`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L111
- **Called by:** `drop_slot` (revision_processor/style_records.rs:758); `write_slot` (revision_processor/style_records.rs:783)

#### `Slot::is_toggle`  (L114-116)
`fn is_toggle(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.contains`
- **Called by:** `Slot::built_in` (revision_processor/style_records.rs:121); `Slot::value` (revision_processor/style_records.rs:156)

#### `Slot::built_in`  (L121-144)
`fn built_in(&self) -> Option<&'static [(&'static str, &'static str)]>`

- **Does:** Word's built-in value, as the attributes it writes (empty for a bare on/off element); `None` where Word's value is unknown, which leaves the slot inheriting.
- **Calls (crate):** `Slot::is_toggle` (revision_processor/style_records.rs:114) L141
- **Calls (std/external/unresolved):** `.as_str`, `.then_some`, `Some`
- **Called by:** `Slot::built_in_value` (revision_processor/style_records.rs:146); `resolve` (revision_processor/style_records.rs:687) (?)

#### `Slot::built_in_value`  (L146-153)
`fn built_in_value(&self) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Slot::built_in` (revision_processor/style_records.rs:121) L147; `Slot::value` (revision_processor/style_records.rs:156) L152
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.to_string`, `Some`
- **Called by:** `complete_from_original_chain` (revision_processor/style_records.rs:448) (?); `resolve` (revision_processor/style_records.rs:687) (?)

#### `Slot::value`  (L156-173)
`fn value(&self, mut attrs: Vec<(String, String)>, children: &str) -> String`

- **Does:** A comparable value from the slot's attributes and serialized children.
- **Calls (crate):** `Slot::is_toggle` (revision_processor/style_records.rs:114) L157; `toggle` (revision_processor/style_records.rs:176) L162,164
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.ends_with`, `.find`, `.into`, `.iter`, `.map`, `.push`, `.retain`, `.sort`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `Slot::built_in_value` (revision_processor/style_records.rs:146); `slot_value` (revision_processor/style_records.rs:236)

#### `toggle`  (L176-181)
`fn toggle(val: Option<&str>) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `definition` (markdown/view_styles.rs:939) (?); `Slot::value` (revision_processor/style_records.rs:156); `read_toggles_against_restored_ancestors` (revision_processor/style_records.rs:637)

#### `prop_slots`  (L186-220)
`fn prop_slots(dom: &Dom, prop: NodeId) -> Vec<(Slot, Vec<(String, String)>)>`

- **Does:** The slots one property element of a block holds, each with its attributes (a group's members, or all of a whole element's); none for a change record or rsid.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L187; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L190; `XName::local_name` (xmllinq/mod.rs:208) (by name) L190,198; `Dom::attributes` (xmllinq/mod.rs:899) L194; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L197; `attribute_groups` (revision_processor/style_records.rs:38) L205; `slot` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::slot` (convert/mod.rs:50332); `cell_picture_owned_room_contract_tests::slot` (convert/mod.rs:63552)) L206,216
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.contains`, `.filter`, `.find`, `.into_iter`, `.iter_mut`, `.iter`, `.map_or`, `.map`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `slot_values` (revision_processor/style_records.rs:223); `complete_from_original_chain` (revision_processor/style_records.rs:448); `effective_rpr` (revision_processor/style_records.rs:585)

#### `slot_values`  (L223-232)
`fn slot_values(dom: &Dom, block: NodeId) -> HashMap<Slot, String>`

- **Does:** Every slot of `block` with its value.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L225; `prop_slots` (revision_processor/style_records.rs:186) L226; `slot_value` (revision_processor/style_records.rs:236) L227
- **Calls (std/external/unresolved):** `.insert`, `HashMap::new`
- **Called by:** `Levels::parse` (revision_processor/style_records.rs:280); `complete_from_original_chain` (revision_processor/style_records.rs:448); `effective_rpr` (revision_processor/style_records.rs:585); `resolve` (revision_processor/style_records.rs:687)

#### `slot_value`  (L236-249)
`fn slot_value(dom: &Dom, prop: NodeId, slot: &Slot, attrs: Vec<(String, String)>) -> String`

- **Does:** The value `prop` gives `slot`: its attributes, and for a whole element its sorted child signatures (tabs).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L238; `signature` (revision_processor/style_records.rs:253) L241; `Slot::value` (revision_processor/style_records.rs:156) L248
- **Calls (std/external/unresolved):** `.collect`, `.concat`, `.into_iter`, `.is_empty`, `.map`, `.sort`, `String::new`
- **Called by:** `slot_values` (revision_processor/style_records.rs:223); `complete_from_original_chain` (revision_processor/style_records.rs:448)

#### `signature`  (L253-270)
`fn signature(dom: &Dom, e: NodeId) -> String`

- **Does:** An element's name, sorted attributes and sorted child signatures: equal for the same property whatever the attribute or child order (tabs).
- **Calls (crate):** `Dom::attributes_iter` (xmllinq/mod.rs:886) L254; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L256; `XName::local_name` (xmllinq/mod.rs:208) (by name) L257,268; `Dom::elements` (xmllinq/mod.rs:640) L260; `signature` (revision_processor/style_records.rs:253) L263; `Dom::name` (xmllinq/mod.rs:526) L266
- **Calls (std/external/unresolved):** `.collect`, `.concat`, `.filter`, `.into_iter`, `.join`, `.map_or`, `.map`, `.sort`, `.to_string`, `String::new`
- **Macros:** `format!`
- **Called by:** `slot_value` (revision_processor/style_records.rs:236); `signature` (revision_processor/style_records.rs:253)

#### `Levels::parse`  (L280-350, pub(super))
`pub(super) fn parse(numbering_xml: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L281; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L282; `Dom::root` (xmllinq/mod.rs:822) L283; `Dom::element` (xmllinq/mod.rs:655) L287,299; `Dom::attribute` (xmllinq/mod.rs:874) L288,295,309,322; `Dom::elements` (xmllinq/mod.rs:640) L292,308,321,340; `slot_values` (revision_processor/style_records.rs:223) L345
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.collect`, `.copied`, `.extend`, `.get`, `.insert`, `.into_iter`, `.iter`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `HashMap::new`, `Self::default`, `Self`, `Some`, `W::name`, `W::val`, `level_pprs`, `val`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `Levels::of`  (L355-363)
`fn of(&self, dom: &Dom, num_pr: NodeId, num_id: &str) -> Option<&HashMap<Slot, String>>`

- **Does:** The level a `numPr` naming `num_id` numbers with: its ilvl (0 when absent), or the nearest lower level its definition has, as Word continues the parent level (`resolve_ilvl`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L357; `Dom::attribute` (xmllinq/mod.rs:874) L359; `ShortId::range` (ids/short.rs:344) (by name) L362
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map`, `.next_back`, `.ok`, `.parse`, `.unwrap_or`, `W::name`, `W::val`
- **Called by:** `resolve` (revision_processor/style_records.rs:687)

#### `recorded_blocks`  (L368-390, pub(super))
`pub(super) fn recorded_blocks(dom: &Dom, styles_root: NodeId) -> Vec<(String, &'static str)>`

- **Does:** The `(styleId, block)` pairs of paragraph and character styles whose `pPr`/`rPr` holds a change record.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L370; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L372; `Dom::attribute` (xmllinq/mod.rs:874) L378; `Dom::element` (xmllinq/mod.rs:655) L380,382
- **Calls (std/external/unresolved):** `.and_then`, `.is_some`, `.push`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`
- **Macros:** `matches!`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `restore_against_built_ins`  (L398-434, pub(super))
`pub(super) fn restore_against_built_ins( dom: &mut Dom, styles_root: NodeId, restored: &[(String, &'static str)], levels: &Levels, next_id: &mut super::RevisionIds, )`

- **Does:** Read the restored toggles against their restored ancestors (R30), resolve each restored paragraph style block of `restored` (the records already rejected) against Word's built-ins, parents before children, then give each linked character style its paragraph style's old rPr.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L405; `Dom::attribute` (xmllinq/mod.rs:874) L408,424; `ancestors` (revision_processor/style_records.rs:666) L414; `read_toggles_against_restored_ancestors` (revision_processor/style_records.rs:637) L423; `resolve` (revision_processor/style_records.rs:687) L431; `resync_linked_character_styles` (revision_processor/style_records.rs:543) L433
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.get`, `.into_iter`, `.iter`, `.len`, `.map`, `.retain`, `.sort_by_key`, `.to_string`, `Some`, `W::name`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `complete_from_original_chain`  (L448-527, pub(crate))
`pub(crate) fn complete_from_original_chain( dom: &mut Dom, old: NodeId, a_root: NodeId, a_style: NodeId, block_local: &str, recorded: &dyn Fn(NodeId) -> bool, ) -> bool`

- **Does:** Give the old `block_local` record `old` every slot the original's style `a_style` (in the original's stylesheet `a_root`) resolved to through its docDefaults and `basedOn` chain and `old` leaves unsaid, except built-in values, which Reject All supplies itself, and the flipping toggles a restored ancestor gives back (R...
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L456,483; `Dom::attribute` (xmllinq/mod.rs:874) L459; `Dom::element` (xmllinq/mod.rs:655) L463,465,466,476; `ancestors` (revision_processor/style_records.rs:666) L470; `prop_slots` (revision_processor/style_records.rs:186) L484; `slot_values` (revision_processor/style_records.rs:223) L489; `tests::recorded` (edit/flags.rs:899) (by name) L495; `Slot::built_in_value` (revision_processor/style_records.rs:146) (by name) L506; `slot_value` (revision_processor/style_records.rs:236) L506; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L510; `insert_in_order` (revision_processor/style_records.rs:810) L516; `write_slot` (revision_processor/style_records.rs:783) L522
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.extend`, `.filter_map`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.push`, `.reverse`, `.sort`, `.to_string`, `HashMap::new`, `Some`, `Vec::new`, `W::name`, `toggled_elsewhere`
- **Macros:** `format!`
- **Called by:** `complete` (document_comparer/based_style_record_tests.rs:69); `toggles_depend_on_the_declaring_ancestor_being_restored` (document_comparer/based_style_record_tests.rs:230); `unrecorded_nearer_toggle_is_kept_when_a_distant_ancestor_is_recorded` (document_comparer/based_style_record_tests.rs:266); `extension_properties_keep_their_namespace_and_do_not_become_word_toggles` (document_comparer/based_style_record_tests.rs:332); `complete_based_style_change_records` (document_comparer.rs:1951)

#### `linked`  (L530-535)
`fn linked(dom: &Dom, by_id: &HashMap<String, NodeId>, style: NodeId, kind: &str) -> Option<NodeId>`

- **Does:** `style`'s `w:link` target, when it is of type `kind`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L531; `Dom::attribute` (xmllinq/mod.rs:874) L532,534
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.filter`, `.get`, `.unwrap_or`, `W::name`, `W::val`
- **Called by:** `resync_linked_character_styles` (revision_processor/style_records.rs:543)

#### `resync_linked_character_styles`  (L543-581)
`fn resync_linked_character_styles( dom: &mut Dom, styles_root: NodeId, by_id: &HashMap<String, NodeId>, recorded: &[NodeId], next_id: &mut super::RevisionIds, )`

- **Does:** R28: the linked character style of each paragraph style whose rPr record was rejected (`recorded`) takes the paragraph style's effective rPr, and so, in turn, does each linked character style based on a resynced one.
- **Calls (crate):** `linked` (revision_processor/style_records.rs:530) L552,575; `effective_rpr` (revision_processor/style_records.rs:585) L560; `Dom::element` (xmllinq/mod.rs:655) L561,569; `Dom::remove` (xmllinq/mod.rs:1118) L562; `Dom::elements` (xmllinq/mod.rs:640) L564; `ensure_block` (revision_processor/style_records.rs:829) L565; `Dom::replace_with` (xmllinq/mod.rs:1165) L566; `Dom::attribute` (xmllinq/mod.rs:874) L571,574
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.filter_map`, `.get`, `.is_empty`, `.iter`, `.pop`, `.push`, `.values`, `Some`, `Vec::new`, `W::name`, `W::val`
- **Called by:** `restore_against_built_ins` (revision_processor/style_records.rs:398)

#### `effective_rpr`  (L585-629)
`fn effective_rpr( dom: &mut Dom, styles_root: NodeId, by_id: &HashMap<String, NodeId>, style: NodeId, next_id: &mut super::RevisionIds, ) -> NodeId`

- **Does:** A new, detached rPr: `style`'s own run properties over its basedOn chain, slot by slot, less the slots the docDefaults say alike.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L592; `ancestors` (revision_processor/style_records.rs:666) L593; `Dom::element` (xmllinq/mod.rs:655) L597,616,618,619; `Dom::elements` (xmllinq/mod.rs:640) L600; `prop_slots` (revision_processor/style_records.rs:186) L601; `drop_slot` (revision_processor/style_records.rs:758) L602,624; `clone_with_fresh_property_ids` (revision_processor.rs:3855) L604; `insert_in_order` (revision_processor/style_records.rs:810) L605; `write_slot` (revision_processor/style_records.rs:783) L611; `slot_values` (revision_processor/style_records.rs:223) L621,622
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.collect`, `.get`, `.is_empty`, `.iter`, `.map`, `.push`, `.reverse`, `Some`, `W::name`
- **Called by:** `resync_linked_character_styles` (revision_processor/style_records.rs:543)

#### `read_toggles_against_restored_ancestors`  (L637-663)
`fn read_toggles_against_restored_ancestors( dom: &mut Dom, by_id: &HashMap<String, NodeId>, styles: &[NodeId], )`

- **Does:** R30: an old record's toggle flips what the nearest ancestor restored by the same reject says (`styles`, parents first): over a restored `b` on, `b` reads off and is written `val=0`.
- **Calls (crate):** `ancestors` (revision_processor/style_records.rs:666) L644; `Dom::element` (xmllinq/mod.rs:655) L649,652; `toggle` (revision_processor/style_records.rs:176) L653; `Dom::attribute` (xmllinq/mod.rs:874) L653; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L657
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.copied`, `.filter`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.is_some`, `.then_some`, `.unwrap_or_default`, `.unwrap_or`, `HashMap::new`, `W::name`, `W::val`
- **Called by:** `restore_against_built_ins` (revision_processor/style_records.rs:398)

#### `ancestors`  (L666-682)
`fn ancestors(dom: &Dom, by_id: &HashMap<String, NodeId>, style: NodeId) -> Vec<NodeId>`

- **Does:** `style`'s `basedOn` ancestors, nearest first, stopping at a cycle.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L670; `Dom::attribute` (xmllinq/mod.rs:874) L672
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.copied`, `.filter`, `.get`, `.len`, `.push`, `Vec::new`, `W::name`, `W::val`
- **Called by:** `restore_against_built_ins` (revision_processor/style_records.rs:398); `complete_from_original_chain` (revision_processor/style_records.rs:448); `effective_rpr` (revision_processor/style_records.rs:585); `read_toggles_against_restored_ancestors` (revision_processor/style_records.rs:637); `resolve` (revision_processor/style_records.rs:687)

#### `resolve`  (L687-754)
`fn resolve( dom: &mut Dom, styles_root: NodeId, by_id: &HashMap<String, NodeId>, levels: &Levels, style: NodeId, block_local: &str, )`

- **Does:** Rewrite `style`'s `block_local` so each slot says what its old value was only where the chain, or for a pPr the style's numbering level, would say otherwise.
- **Calls (crate):** `ancestors` (revision_processor/style_records.rs:666) L695; `Dom::element` (xmllinq/mod.rs:655) L697,699,705,706,714,716...; `Dom::attribute` (xmllinq/mod.rs:874) L708; `Levels::of` (revision_processor/style_records.rs:355) L709; `slot_values` (revision_processor/style_records.rs:223) L721,723; `Slot::built_in_value` (revision_processor/style_records.rs:146) (by name) L733,746; `drop_slot` (revision_processor/style_records.rs:758) L740; `Slot::built_in` (revision_processor/style_records.rs:121) (by name) L745; `ensure_block` (revision_processor/style_records.rs:829) L748; `write_slot` (revision_processor/style_records.rs:783) L749
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.chain`, `.cloned`, `.collect`, `.filter_map`, `.find_map`, `.flat_map`, `.flatten`, `.get`, `.into_iter`, `.iter`, `.keys`, `.map`, `.or_else`, `.push`, `.then`, `.unwrap_or_default`, `Some`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `restore_against_built_ins` (revision_processor/style_records.rs:398)

#### `drop_slot`  (L758-780)
`fn drop_slot(dom: &mut Dom, block: NodeId, slot: &Slot)`

- **Does:** Remove one slot from `block`: the whole element, or its group's attributes (and the element once none is left).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L759; `Slot::name` (revision_processor/style_records.rs:110) L759; `attribute_groups` (revision_processor/style_records.rs:38) L762; `Dom::remove` (xmllinq/mod.rs:1118) L763,778; `Dom::attributes` (xmllinq/mod.rs:899) L772; `XName::local_name` (xmllinq/mod.rs:208) (by name) L773; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L774; `Dom::has_attributes` (xmllinq/mod.rs:867) L777
- **Calls (std/external/unresolved):** `.contains`, `.copied`, `.filter`, `.find`, `.iter`, `.unwrap_or`
- **Called by:** `effective_rpr` (revision_processor/style_records.rs:585); `resolve` (revision_processor/style_records.rs:687)

#### `write_slot`  (L783-806)
`fn write_slot( dom: &mut Dom, block: NodeId, block_local: &str, slot: &Slot, attrs: &[(&str, &str)], )`

- **Does:** Write a built-in slot into `block`, in schema order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L790; `Slot::name` (revision_processor/style_records.rs:110) L790,793; `Dom::new_element` (xmllinq/mod.rs:478) L793; `insert_in_order` (revision_processor/style_records.rs:810) L799; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L804; `XName::get` (xmllinq/mod.rs:180) L804
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `complete_from_original_chain` (revision_processor/style_records.rs:448); `effective_rpr` (revision_processor/style_records.rs:585); `resolve` (revision_processor/style_records.rs:687)

#### `insert_in_order`  (L810-826)
`fn insert_in_order(dom: &mut Dom, parent: NodeId, child: NodeId, order: &[(&str, i32)])`

- **Does:** Insert `child` after the last child `order` ranks before it; elements outside `order` (w14 extensions) rank last.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L812; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L813; `XName::local_name` (xmllinq/mod.rs:208) (by name) L814; `rank` (ambiguous, 4 candidates) L817,821; `Dom::elements` (xmllinq/mod.rs:640) L818; `Dom::add_after_self` (xmllinq/mod.rs:1147) L823; `Dom::add_first` (xmllinq/mod.rs:1101) L824
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.find`, `.into_iter`, `.iter`, `.map_or`, `.rfind`
- **Called by:** `complete_from_original_chain` (revision_processor/style_records.rs:448); `effective_rpr` (revision_processor/style_records.rs:585); `write_slot` (revision_processor/style_records.rs:783)

#### `ensure_block`  (L829-848)
`fn ensure_block(dom: &mut Dom, style: NodeId, block_local: &str) -> NodeId`

- **Does:** The style's `pPr`/`rPr`, created where CT_Style puts it when missing.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L830; `Dom::new_element` (xmllinq/mod.rs:478) L833; `Dom::elements` (xmllinq/mod.rs:640) L839; `Dom::name` (xmllinq/mod.rs:526) L842; `XName::local_name` (xmllinq/mod.rs:208) (by name) L842; `Dom::add_before_self` (xmllinq/mod.rs:1130) L844; `Dom::add` (xmllinq/mod.rs:1074) L845
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`, `W::name`
- **Called by:** `resync_linked_character_styles` (revision_processor/style_records.rs:543); `resolve` (revision_processor/style_records.rs:687)
