# Functions and methods in `src/settings.rs`

12 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `ProtectionEdit::value`  (L164-172)
`fn value(self) -> &'static str`

- **Does:** The exact `w:edit` value Word writes on the wire (`"none"`, `"readOnly"`, `"comments"`, `"trackedChanges"`, `"forms"`).
- **Calls:** none
- **Called by:** none found in src

#### `default_true`  (L175-177)
`fn default_true() -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `SettingsRequest::describe`  (L216-233, pub)
`pub fn describe(&self) -> String`

- **Does:** A short human-readable description of what this request writes, for a report line: never the password; protection prints its edit value and whether it is enforced.
- **Calls (std/external/unresolved):** `.join`, `.push`, `.value`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_settings` (edit.rs:5791); `tests::the_description_names_each_setting_given` (settings.rs:509) (?)

#### `rank`  (L237-242)
`fn rank(dom: &Dom, node: NodeId) -> Option<usize>`

- **Does:** The rank of `node` in [`SETTINGS_ORDER`], if it is a known child.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L238; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L241; `XName::local_name` (xmllinq/mod.rs:208) (by name) L241
- **Calls (std/external/unresolved):** `.iter`, `.position`
- **Called by:** `set_child` (settings.rs:248)

#### `set_child`  (L248-267, pub)
`pub fn set_child(dom: &mut Dom, root: NodeId, local: &str, attrs: &[(&str, &str)]) -> NodeId`

- **Does:** Insert or replace `<w:local attrs/>` under `root`, ordered against the children already there by the `CT_Settings` sequence ([`SETTINGS_ORDER`]).
- **Calls (crate):** `remove_child` (settings.rs:271) L249; `Dom::new_element` (xmllinq/mod.rs:478) L254; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L256; `Dom::elements` (xmllinq/mod.rs:640) L258; `rank` (settings.rs:237) L261; `Dom::add_before_self` (xmllinq/mod.rs:1130) L263; `Dom::add` (xmllinq/mod.rs:1074) L264
- **Calls (std/external/unresolved):** `.expect`, `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.position`, `Some`, `W::name`
- **Called by:** `apply_to_root` (settings.rs:279); `do_not_expand_shift_return` (settings.rs:383); `tests::a_child_lands_before_the_first_later_one_and_after_unknown_ones` (settings.rs:487); `tests::a_child_with_no_later_one_is_appended` (settings.rs:501)

#### `remove_child`  (L271-276, pub)
`pub fn remove_child(dom: &mut Dom, root: NodeId, local: &str)`

- **Does:** Remove every `<w:local>` child of `root` (none or many).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L273; `Dom::remove` (xmllinq/mod.rs:1118) L274
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `set_child` (settings.rs:248); `apply_to_root` (settings.rs:279)

#### `apply_to_root`  (L279-308)
`fn apply_to_root(dom: &mut Dom, root: NodeId, request: &SettingsRequest)`

- **Does:** Apply `request` to the settings root.
- **Calls (crate):** `set_child` (settings.rs:248) L286,297; `remove_child` (settings.rs:271) L288,294
- **Calls (std/external/unresolved):** `.value`
- **Called by:** `apply_settings` (settings.rs:347)

#### `settings_part`  (L312-337)
`fn settings_part(pkg: &mut PartFs, main: &str) -> String`

- **Does:** The main part's settings part, created with its relationship and content type when the package has none.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L313; `PartFs::resolve_rel_target` (opc/mod.rs:829) L320; `PartFs::part_bytes` (opc/mod.rs:628) L326; `PartFs::set_part` (opc/mod.rs:661) L332; `PartFs::add_content_type_override` (opc/mod.rs:845) L334; `PartFs::add_document_relationship` (opc/mod.rs:1003) L335
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.find`, `.into_bytes`, `.is_none`, `.iter`, `.map`, `.rsplit_once`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `apply_settings` (settings.rs:347); `no_stretched_line_breaks` (settings.rs:405); `tests::a_part_that_is_not_settings_is_refused` (settings.rs:526)

#### `apply_settings`  (L347-363, pub)
`pub fn apply_settings( pkg: &mut PartFs, main: &str, request: &SettingsRequest, ) -> Result<(), String>`

- **Does:** Apply `request` into the package's settings part, creating it with its relationship and content type when the package has none.
- **Calls (crate):** `settings_part` (settings.rs:312) L352; `PartFs::part_string` (opc/mod.rs:636) L353; `Dom::new` (xmllinq/mod.rs:387) L354; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L355; `Dom::root` (xmllinq/mod.rs:822) L356; `Dom::name_is` (xmllinq/mod.rs:535) L358; `apply_to_root` (settings.rs:279) L360; `PartFs::set_part` (opc/mod.rs:661) L361; `Dom::serialize_document` (xmllinq/mod.rs:1338) L361
- **Calls (std/external/unresolved):** `.filter`, `.into_bytes`, `.ok_or_else`, `.unwrap_or_default`, `Ok`, `W::name`
- **Macros:** `format!`
- **Called by:** `Transaction::finish` (edit.rs:6645); `apply_settings_to_docx` (settings.rs:424); `tests::a_part_that_is_not_settings_is_refused` (settings.rs:526)

#### `do_not_expand_shift_return`  (L383-401)
`fn do_not_expand_shift_return(dom: &mut Dom, root: NodeId)`

- **Does:** Word's "Don't justify lines ending in soft line break" (`w:compat/w:doNotExpandShiftReturn`, ECMA-376 17.15.3.10) turned on under the settings `root`, in schema order.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L384,388,392; `set_child` (settings.rs:248) L386; `Dom::remove` (xmllinq/mod.rs:1118) L389; `Dom::new_element` (xmllinq/mod.rs:478) L391; `Dom::name` (xmllinq/mod.rs:526) L393; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L394; `XName::local_name` (xmllinq/mod.rs:208) (by name) L394; `Dom::add_before_self` (xmllinq/mod.rs:1130) L398; `Dom::add` (xmllinq/mod.rs:1074) L399
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `Some`, `W::name`
- **Called by:** `no_stretched_line_breaks` (settings.rs:405); `tests::a_shift_return_flag_lands_in_compat_in_schema_order` (settings.rs:464)

#### `no_stretched_line_breaks`  (L405-421, pub(crate))
`pub(crate) fn no_stretched_line_breaks(docx: &[u8]) -> Result<Vec<u8>, String>`

- **Does:** [`do_not_expand_shift_return`] on a whole package: a justified line that ends in a line break is not stretched to the margin.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L406; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L407; `settings_part` (settings.rs:312) L410; `PartFs::part_string` (opc/mod.rs:636) (by name) L411; `Dom::new` (xmllinq/mod.rs:387) L412; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L413; `Dom::root` (xmllinq/mod.rs:822) L414; `Dom::name_is` (xmllinq/mod.rs:535) L416; `do_not_expand_shift_return` (settings.rs:383) L418; `PartFs::set_part` (opc/mod.rs:661) (by name) L419; `Dom::serialize_document` (xmllinq/mod.rs:1338) L419; `PartFs::to_zip` (opc/mod.rs:749) (by name) L420
- **Calls (std/external/unresolved):** `.filter`, `.into_bytes`, `.map_err`, `.ok_or_else`, `.ok_or`, `.to_string`, `.unwrap_or_default`, `W::name`
- **Macros:** `format!`
- **Called by:** `markdown_to_docx` (markdown/write.rs:59)

#### `apply_settings_to_docx`  (L424-431, pub)
`pub fn apply_settings_to_docx(docx: &[u8], request: &SettingsRequest) -> Result<Vec<u8>, String>`

- **Does:** [`apply_settings`] on a whole package.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L425; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L426; `apply_settings` (settings.rs:347) L429; `PartFs::to_zip` (opc/mod.rs:749) (by name) L430
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or`, `.to_string`
- **Called by:** `produce` (edit/prepared.rs:928); `markdown_to_docx` (markdown/write.rs:59)

## Test functions (7)

- `tests::settings` (L438): _no doc comment_
- `tests::each_protection_is_spelled_on_the_wire_as_in_the_xml` (L449): _no doc comment_
- `tests::a_shift_return_flag_lands_in_compat_in_schema_order` (L464): _no doc comment_
- `tests::a_child_lands_before_the_first_later_one_and_after_unknown_ones` (L487): _no doc comment_
- `tests::a_child_with_no_later_one_is_appended` (L501): _no doc comment_
- `tests::the_description_names_each_setting_given` (L509): _no doc comment_
- `tests::a_part_that_is_not_settings_is_refused` (L526): _no doc comment_
