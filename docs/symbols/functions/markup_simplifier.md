# Functions and methods in `src/markup_simplifier.rs`

7 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `xml_space`  (L21-23)
`fn xml_space() -> XName`

- **Does:** xml:space attribute name.
- **Calls (crate):** `XNamespace::xml` (xmllinq/mod.rs:128) L22
- **Calls (std/external/unresolved):** `.name`
- **Called by:** `single_character_run_transform` (markup_simplifier.rs:49)

#### `build_run`  (L26-33)
`fn build_run(dom: &mut Dom, rpr_elems: &[NodeId], inner: NodeId) -> NodeId`

- **Does:** Build `<w:r>` with the given rPr element clones followed by `inner`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L27; `Dom::add` (xmllinq/mod.rs:1058) L29,31
- **Calls (std/external/unresolved):** `W::r`
- **Called by:** `single_character_run_transform` (markup_simplifier.rs:49)

#### `copy_attrs`  (L36-40)
`fn copy_attrs(dom: &mut Dom, src: NodeId, dst: NodeId)`

- **Does:** Copy `src`'s attributes onto `dst`.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L37; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L38
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `single_character_run_transform` (markup_simplifier.rs:49)

#### `single_character_run_transform`  (L49-113, pub)
`pub fn single_character_run_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `SingleCharacterRunTransform(node)` — returns the transformed node(s).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L50; `Dom::name` (xmllinq/mod.rs:526) L53,62,66,86; `Dom::elements` (xmllinq/mod.rs:640) L57,59; `group_adjacent` (util/group_adjacent.rs:20) L66; `Dom::value` (xmllinq/mod.rs:1169) L72; `Dom::new_element` (xmllinq/mod.rs:478) L75,87,104; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L77; `xml_space` (markup_simplifier.rs:21) L77; `Dom::add_text` (xmllinq/mod.rs:1071) L79; `build_run` (markup_simplifier.rs:26) L80,95; `copy_attrs` (markup_simplifier.rs:36) L88,105; `Dom::nodes` (xmllinq/mod.rs:604) L89,106; `single_character_run_transform` (markup_simplifier.rs:49) L90,107; `Dom::add` (xmllinq/mod.rs:1058) L92,109
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.into_iter`, `.push_str`, `.push`, `.to_string`, `.unwrap`, `Some`, `String::new`, `Vec::new`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`, `vec!`
- **Called by:** `single_character_run_transform` (markup_simplifier.rs:49); `transform_element_to_single_character_runs` (markup_simplifier.rs:125)

#### `transform_element_to_single_character_runs`  (L125-132, pub)
`pub fn transform_element_to_single_character_runs(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** `TransformElementToSingleCharacterRuns(element)` — returns the single rebuilt root of a non-`w:r` container such as the body or a paragraph.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L126; `Dom::name` (xmllinq/mod.rs:526) L126; `single_character_run_transform` (markup_simplifier.rs:49) L129
- **Calls (std/external/unresolved):** `.len`, `Some`, `W::r`
- **Macros:** `debug_assert_eq!`
- **Called by:** none found in src

#### `is_rsid_attr`  (L136-144)
`fn is_rsid_attr(name: &XName) -> bool`

- **Does:** True if `name` is a volatile `w:rsid*` attribute (the set stripped by `RemoveRsidTransform`).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L137; `XName::local_name` (xmllinq/mod.rs:208) (by name) L141
- **Macros:** `matches!`
- **Called by:** `remove_rsid_transform` (markup_simplifier.rs:154)

#### `remove_rsid_transform`  (L154-182, pub)
`pub fn remove_rsid_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** Port of `RemoveRsidTransform(node)` — drops every `w:rsid*` element and every `w:rsid*` attribute across the subtree.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L155; `Dom::name` (xmllinq/mod.rs:526) L158; `Dom::attributes` (xmllinq/mod.rs:885) L163; `is_rsid_attr` (markup_simplifier.rs:136) L166; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L170; `Dom::child_count` (xmllinq/mod.rs:622) L173; `Dom::child_at` (xmllinq/mod.rs:632) L174; `remove_rsid_transform` (markup_simplifier.rs:154) L177; `Dom::remove` (xmllinq/mod.rs:1102) L178
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_none`, `.map`, `.unwrap`, `Some`, `W::name`
- **Called by:** `remove_rsid_transform` (markup_simplifier.rs:154); `accept_revisions_for_element` (revision_processor.rs:459); `reject_revisions_document` (revision_processor.rs:729); `accept_revisions_for_part_content` (revision_processor.rs:3252); `remove_rsids` (scrub.rs:326)
