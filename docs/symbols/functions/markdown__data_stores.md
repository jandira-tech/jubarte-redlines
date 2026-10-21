# Functions and methods in `src/markdown/data_stores.rs`

15 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `write_missing`  (L63-150, pub(crate))
`pub(crate) fn write_missing(package: &mut PartFs, main: &str, warnings: &mut Vec<String>)`

- **Does:** Write the stores the written parts' bindings name and the main part does not reach; drop the bindings no store can be built for.
- **Calls (crate):** `stores` (markdown/data_stores.rs:155) L64; `PartFs::parts` (opc/mod.rs:733) L66; `PartFs::part_bytes` (opc/mod.rs:628) L70,141; `parse_xml` (markdown/from_docx/ooxml.rs:440) (by name) L78; `bindings` (markdown/data_stores.rs:191) L86; `built_in` (markdown/data_stores.rs:186) L87; `insert` (markdown/data_stores.rs:366) L118; `write_store` (markdown/data_stores.rs:461) L138; `unbound` (markdown/data_stores.rs:510) L147; `PartFs::set_part` (opc/mod.rs:661) L148
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_bytes`, `.clone`, `.contains`, `.ends_with`, `.eq_ignore_ascii_case`, `.into_bytes`, `.is_empty`, `.iter`, `.len`, `.ok`, `.position`, `.push`, `.retain`, `.starts_with`, `.to_ascii_uppercase`, `.windows`, `Vec::new`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `stores`  (L155-182, pub(crate))
`pub(crate) fn stores(package: &PartFs, main: &str) -> Vec<String>`

- **Does:** The stores the main part reaches, by item ID (upper case): each `customXml` relationship's item, through its `customXmlProps` relationship to the properties naming it.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L157; `PartFs::resolve_rel_target` (opc/mod.rs:829) L167; `related` (ambiguous: `related` (append.rs:766); `related` (markdown/package.rs:512)) L174,177; `PartFs::part_bytes` (opc/mod.rs:628) L176,178; `parse_xml` (markdown/from_docx/ooxml.rs:440) (by name) L178; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L179; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L180
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_some`, `.map`, `.next`, `.ok`, `.rsplit`, `.to_string`, `.trim_start_matches`, `Some`
- **Called by:** `write_missing` (markdown/data_stores.rs:63); `check_data_bindings_have_stores` (validate.rs:1189) (?)

#### `built_in`  (L186-188, pub(crate))
`pub(crate) fn built_in(store: &str) -> bool`

- **Does:** Whether a binding names one of Word's own stores, which `docProps` holds.
- **Calls (std/external/unresolved):** `.any`, `.eq_ignore_ascii_case`, `.iter`
- **Called by:** `write_missing` (markdown/data_stores.rs:63); `check_data_bindings_have_stores` (validate.rs:1189) (?)

#### `bindings`  (L191-208)
`fn bindings(e: &Element, out: &mut Vec<Binding>)`

- **Does:** Every bound control under `e`, nested ones too.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L192; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L193; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L194; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L195,197,198; `prefix_mappings` (markdown/data_stores.rs:211) L198; `steps` (markdown/data_stores.rs:234) L201; `shown` (markdown/data_stores.rs:266) L203; `bindings` (markdown/data_stores.rs:191) L206
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `.unwrap_or_default`
- **Called by:** `write_missing` (markdown/data_stores.rs:63); `bindings` (markdown/data_stores.rs:191)

#### `prefix_mappings`  (L211-229)
`fn prefix_mappings(text: &str) -> Vec<(String, String)>`

- **Does:** `xmlns:ns0='uri' xmlns:ns1="uri"` as (prefix, URI) pairs.
- **Calls (std/external/unresolved):** `.chars`, `.filter`, `.find`, `.len`, `.next`, `.push`, `.split_once`, `.to_string`, `.trim`, `Vec::new`
- **Called by:** `bindings` (markdown/data_stores.rs:191); `tests::a_simple_path_reads_as_its_steps` (markdown/data_stores.rs:560); `tests::a_name_that_is_no_ncname_is_no_step` (markdown/data_stores.rs:654)

#### `steps`  (L234-257)
`fn steps(xpath: &str, prefixes: &[(String, String)]) -> Option<Vec<(String, String)>>`

- **Does:** A simple path's steps: `/p:a[1]/p:b[1]`, each a mapped prefix's name, the first of its name (`[1]` or no index).
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.clone`, `.collect`, `.find`, `.is_alphabetic`, `.is_alphanumeric`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.next`, `.split_once`, `.split`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.to_string`, `.unwrap_or`, `Some`, `simple`
- **Macros:** `matches!`
- **Called by:** `bindings` (markdown/data_stores.rs:191); `tests::a_simple_path_reads_as_its_steps` (markdown/data_stores.rs:560); `tests::a_name_that_is_no_ncname_is_no_step` (markdown/data_stores.rs:654)

#### `shown`  (L266-305)
`fn shown(sdt: &Element) -> Option<String>`

- **Does:** What a control shows, as its store holds it: nothing while it shows its placeholder; a date control's date as its `w:storeMappedDataAs` asks; a list's chosen item's value; a plain text control's text.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L267,276; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L270,271,279,283; `plain` (markdown/data_stores.rs:309) L277; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L282,285,295,298; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L294
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.find`, `.into_iter`, `.is_some`, `.map`, `.next`, `.or_else`, `.split`, `.unwrap_or`, `Some`, `String::new`
- **Called by:** `refused` (edit/anchor_error.rs:462) (?); `quoted_anchor` (edit/anchor_error.rs:591) (?); `tests::a_pasted_clause_anchor_is_quoted_bounded` (edit/anchor_error.rs:715) (?); `bindings` (markdown/data_stores.rs:191); `tests::a_control_s_store_value_is_its_text_only_where_word_keeps_it_so` (markdown/data_stores.rs:663)

#### `plain`  (L309-332)
`fn plain(e: &Element, out: &mut String, paragraphs: &mut usize) -> Option<()>`

- **Does:** The text a plain text control's runs show: `w:t`, a tab, a break.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L310; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L311,327; `.text` (ambiguous, 10 candidates) L312; `plain` (markdown/data_stores.rs:309) L320,323
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.push_str`, `.push`, `Some`
- **Called by:** `shown` (markdown/data_stores.rs:266); `plain` (markdown/data_stores.rs:309)

#### `insert`  (L366-399)
`fn insert(root: &mut Node, steps: &[(String, String)], value: &str) -> Placed`

- **Does:** Put `value` at `steps` under `root` (whose own step is `steps[0]`).
- **Calls (std/external/unresolved):** `.clone`, `.iter`, `.len`, `.position`, `.push`, `.to_string`, `Placed::Conflict`, `Some`, `Vec::new`
- **Called by:** `write_missing` (markdown/data_stores.rs:63); `tests::a_store_holds_each_path_once_under_one_root` (markdown/data_stores.rs:590); `tests::a_node_bound_whole_and_in_parts_keeps_its_parts` (markdown/data_stores.rs:788); `operations` (markdown/patch.rs:502) (?)

#### `namespaces`  (L402-414)
`fn namespaces(root: &Node) -> Vec<&str>`

- **Does:** The namespaces a store's nodes use, in the order they use them.
- **Calls (crate):** `namespaces::{fn walk}` (markdown/data_stores.rs:403) L412
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `store_xml` (markdown/data_stores.rs:419); `write_store` (markdown/data_stores.rs:461); `tests::a_store_holds_each_path_once_under_one_root` (markdown/data_stores.rs:590)

#### `namespaces::{fn walk}`  (L403-410)
`fn walk<'a>(node: &'a Node, out: &mut Vec<&'a str>)`

- **Does:** _no doc comment_
- **Calls (crate):** `namespaces::{fn walk}` (markdown/data_stores.rs:403) L408
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.push`
- **Called by:** `namespaces` (markdown/data_stores.rs:402); `namespaces::{fn walk}` (markdown/data_stores.rs:403)

#### `store_xml`  (L419-457)
`fn store_xml(root: &Node) -> String`

- **Does:** The store as XML: one namespace as the root's default, as Word writes a cover page's; several declared once on the root, `ns0`, `ns1`, … in the order the nodes use them.
- **Calls (crate):** `namespaces` (markdown/data_stores.rs:402) L445; `escape` (ambiguous, 5 candidates) L447,451; `store_xml::{fn element}` (markdown/data_stores.rs:420) L455
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.enumerate`, `.iter`, `.map`, `String::from`
- **Macros:** `format!`
- **Called by:** `write_store` (markdown/data_stores.rs:461); `tests::a_store_holds_each_path_once_under_one_root` (markdown/data_stores.rs:590); `tests::a_node_bound_whole_and_in_parts_keeps_its_parts` (markdown/data_stores.rs:788)

#### `store_xml::{fn element}`  (L420-444)
`fn element(node: &Node, uris: &[&str], declarations: &str, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `store_xml::{fn element}` (markdown/data_stores.rs:420) L435; `escape` (ambiguous, 5 candidates) L440
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.iter`, `.position`, `.push_str`, `.push`, `.unwrap_or`
- **Macros:** `format!`, `write!`
- **Called by:** `store_xml` (markdown/data_stores.rs:419); `store_xml::{fn element}` (markdown/data_stores.rs:420)

#### `write_store`  (L461-505)
`fn write_store(package: &mut PartFs, main: &str, store: &str, root: &Node)`

- **Does:** Write `customXml/itemN.xml` with its `itemPropsN.xml`, related from the main document part.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L462; `PartFs::read_rels_for` (opc/mod.rs:991) L469; `PartFs::set_part` (opc/mod.rs:661) L476,481; `store_xml` (markdown/data_stores.rs:419) L476; `namespaces` (markdown/data_stores.rs:402) L477; `escape` (ambiguous, 5 candidates) L479,487; `PartFs::content_type_for` (opc/mod.rs:837) L491; `PartFs::add_content_type_override` (opc/mod.rs:845) L492,494; `PartFs::add_document_relationship` (opc/mod.rs:1003) L495,500; `relative_rel_target` (opc/mod.rs:180) (by name) L503
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.eq_ignore_ascii_case`, `.find`, `.into_bytes`, `.is_none`, `.iter`, `.map`, `.unwrap_or`, `taken`
- **Macros:** `format!`
- **Called by:** `write_missing` (markdown/data_stores.rs:63)

#### `unbound`  (L510-550)
`fn unbound(xml: &str, store: &str, xpath: &str) -> String`

- **Does:** `xml` without the `dataBinding` elements that bind `store` at `xpath`, read as the parser reads them (either quote, entities, an element closed by its own end tag).
- **Calls (crate):** `element_from` (markdown/from_docx/ooxml.rs:552) (by name) L512; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L513; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L514,516
- **Calls (std/external/unresolved):** `.as_ref`, `.buffer_position`, `.config_mut`, `.eq_ignore_ascii_case`, `.is_ok_and`, `.is_some_and`, `.len`, `.name`, `.push_str`, `.push`, `.read_event`, `.to_string`, `.trim_text`, `Arc::default`, `Reader::from_str`, `Some`, `String::with_capacity`, `Vec::new`, `binds`
- **Called by:** `write_missing` (markdown/data_stores.rs:63); `tests::an_unbuilt_binding_is_dropped_and_no_other` (markdown/data_stores.rs:753); `tests::a_binding_is_dropped_however_its_tag_is_written` (markdown/data_stores.rs:764)

## Test functions (7)

- `tests::a_simple_path_reads_as_its_steps` (L560): _no doc comment_
- `tests::a_store_holds_each_path_once_under_one_root` (L590): _no doc comment_
- `tests::a_name_that_is_no_ncname_is_no_step` (L654): _no doc comment_
- `tests::a_control_s_store_value_is_its_text_only_where_word_keeps_it_so` (L663): _no doc comment_
- `tests::an_unbuilt_binding_is_dropped_and_no_other` (L753): _no doc comment_
- `tests::a_binding_is_dropped_however_its_tag_is_written` (L764): _no doc comment_
- `tests::a_node_bound_whole_and_in_parts_keeps_its_parts` (L788): _no doc comment_
