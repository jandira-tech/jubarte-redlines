# Functions and methods in `src/markdown/from_docx/drawing.rs`

83 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `ScopeMap::of`  (L174-196, pub(crate))
`pub(crate) fn of(root: &Element) -> Self`

- **Does:** The scope map of a part's tree.
- **Calls (crate):** `scope_of` (markdown/from_docx/drawing.rs:437) L175; `Element::elements` (markdown/from_docx/ooxml.rs:372) L189; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L190
- **Calls (std/external/unresolved):** `HashMap::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `ScopeMap::of::{fn walk}`  (L177-188)
`fn walk(element: &Element, scope: &Scope, map: &mut HashMap<*const Element, Scope>)`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L179; `merged` (markdown/from_docx/drawing.rs:462) L184; `scope_of` (markdown/from_docx/drawing.rs:437) L184; `Element::elements` (markdown/from_docx/ooxml.rs:372) L185; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L186
- **Calls (std/external/unresolved):** `.clone`, `.insert`
- **Macros:** `matches!`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:174); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `owned_drawings` (markdown/from_docx/drawing.rs:311); `owned_drawings::{fn walk}` (markdown/from_docx/drawing.rs:312); `strip_box_words` (markdown/from_docx/drawing.rs:833); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `box_ordinals` (markdown/from_docx/drawing.rs:1526); `box_ordinals::{fn walk}` (markdown/from_docx/drawing.rs:1527); `fill_boxes` (markdown/from_docx/drawing.rs:1619); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `ScopeMap::scope`  (L200-207, pub(crate))
`pub(crate) fn scope(&self, element: &Element) -> Scope`

- **Does:** The scope frozen at `element`: its ancestors' declarations, then its own.
- **Calls (crate):** `merged` (markdown/from_docx/drawing.rs:462) L201; `scope_of` (markdown/from_docx/drawing.rs:437) L205
- **Calls (std/external/unresolved):** `.get`, `.unwrap_or`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::drawing` (markdown/from_docx/mod.rs:3572)

#### `Pending::push`  (L226-239, pub(crate))
`pub(crate) fn push( list: &mut Vec<Pending>, element: &Element, part: &str, scope: Scope, wrap: bool, stories: Vec<String>, ) -> String`

- **Does:** Header/footer captures remain opaque and share the same queue.
- **Calls (crate):** `Pending::reserve` (markdown/from_docx/drawing.rs:242) L234; `image` (markdown/from_docx/drawing.rs:390) L235; `label` (markdown/from_docx/drawing.rs:399) L235
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `Pending::reserve`  (L242-262, pub(crate))
`pub(crate) fn reserve( list: &mut Vec<Pending>, element: &Element, part: &str, scope: Scope, wrap: bool, ) -> Option<(usize, String)>`

- **Does:** Reserve the enclosing number before visiting its nested drawings.
- **Calls (crate):** `image` (markdown/from_docx/drawing.rs:390) L261; `label` (markdown/from_docx/drawing.rs:399) L261
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.push`, `.to_string`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Pending::push` (markdown/from_docx/drawing.rs:226); `Writer::drawing` (markdown/from_docx/mod.rs:3572)

#### `Pending::attach`  (L265-306, pub(crate))
`pub(crate) fn attach( list: &mut [Pending], index: usize, element: &Element, stories: Vec<String>, )`

- **Does:** Transfer accepted nested copies to their tokens after recursion.
- **Calls (crate):** `nested_pairs` (markdown/from_docx/drawing.rs:347) L272; `Pending::attach::{fn transfer}` (markdown/from_docx/drawing.rs:281) L304
- **Calls (std/external/unresolved):** `.clear`, `.insert`, `.iter`, `.rposition`, `HashMap::new`, `std::ptr::eq`
- **Macros:** `format!`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3572)

#### `Pending::attach::{fn transfer}`  (L281-303)
`fn transfer( element: &Element, replacements: &HashMap<*const Element, (String, bool)>, ) -> Element`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L286; `Pending::attach::{fn transfer}` (markdown/from_docx/drawing.rs:281) L298
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.expect`, `.get`, `.into`, `.iter`, `.map`, `.to_string`, `Node::Element`, `Node::Text`
- **Macros:** `vec!`
- **Called by:** `Pending::attach` (markdown/from_docx/drawing.rs:265); `Pending::attach::{fn transfer}` (markdown/from_docx/drawing.rs:281)

#### `owned_drawings`  (L311-329, pub(crate))
`pub(crate) fn owned_drawings(story: &Element) -> Vec<&Element>`

- **Does:** Drawing roots directly owned by this story.
- **Calls (crate):** `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L327
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `nested_pairs` (markdown/from_docx/drawing.rs:347); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `owned_drawings::{fn walk}`  (L312-325)
`fn walk<'a>(node: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L313; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L314,318; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L317; `capturable_alternate` (markdown/from_docx/drawing.rs:1516) L318; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L322
- **Calls (std/external/unresolved):** `.push`
- **Macros:** `matches!`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:174); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `owned_drawings` (markdown/from_docx/drawing.rs:311); `owned_drawings::{fn walk}` (markdown/from_docx/drawing.rs:312); `strip_box_words` (markdown/from_docx/drawing.rs:833); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `box_ordinals` (markdown/from_docx/drawing.rs:1526); `box_ordinals::{fn walk}` (markdown/from_docx/drawing.rs:1527); `fill_boxes` (markdown/from_docx/drawing.rs:1619); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `shape_shell`  (L331-343)
`fn shape_shell(element: &Element) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `declare` (markdown/from_docx/drawing.rs:495) L338; `Dom::new` (xmllinq/mod.rs:387) L339; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L340; `serialize` (markdown/from_docx/drawing.rs:538) L340; `Dom::elements` (xmllinq/mod.rs:640) L341; `shell` (comparer/textbox.rs:168) L342
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`
- **Called by:** `nested_pairs` (markdown/from_docx/drawing.rs:347)

#### `nested_pairs`  (L347-381)
`fn nested_pairs(element: &Element) -> Option<Vec<(&Element, &Element, bool)>>`

- **Does:** Pair nearest-owned copies by box ordinal and drawing order.
- **Calls (crate):** `Element::find_all` (markdown/from_docx/ooxml.rs:404) L349; `box_ordinals` (markdown/from_docx/drawing.rs:1526) L352; `owned_drawings` (markdown/from_docx/drawing.rs:311) L353; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L359; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L360; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L361; `shape_shell` (markdown/from_docx/drawing.rs:331) L370
- **Calls (std/external/unresolved):** `.extend`, `.get`, `.insert`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.map`, `.next`, `.push`, `.zip`, `HashMap::<usize, Vec>::new`, `Some`, `Vec::new`
- **Called by:** `Pending::attach` (markdown/from_docx/drawing.rs:265); `nested_editable` (markdown/from_docx/drawing.rs:383)

#### `nested_editable`  (L383-385, pub(crate))
`pub(crate) fn nested_editable(element: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `nested_pairs` (markdown/from_docx/drawing.rs:347) L384
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Writer::box_blocks` (markdown/from_docx/mod.rs:3636)

#### `image`  (L390-396)
`fn image(label: &str, name: &str) -> String`

- **Does:** `![label][name]` with the label made a Markdown alt text: one line, brackets escaped.
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `format!`
- **Called by:** `tests::drawings_compare_their_placement_and_target` (debug/diff.rs:2039) (?); `coverage_boundary_tests::a_reused_header_keeps_its_own_relationships` (document_comparer.rs:10669) (?); `Pending::push` (markdown/from_docx/drawing.rs:226); `Pending::reserve` (markdown/from_docx/drawing.rs:242)

#### `label`  (L399-409)
`fn label(element: &Element) -> String`

- **Does:** The `wp:docPr` description, title or name as the token's label.
- **Calls (crate):** `alt` (markdown/from_docx/drawing.rs:414) L400; `Element::find` (markdown/from_docx/ooxml.rs:396) L404; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L406
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.map`, `.unwrap_or_default`
- **Called by:** `Pending::push` (markdown/from_docx/drawing.rs:226); `Pending::reserve` (markdown/from_docx/drawing.rs:242); `fill` (markdown/from_docx/drawing.rs:929); `context_view` (text_diff.rs:410) (?)

#### `alt`  (L414-425)
`fn alt(element: &Element) -> String`

- **Does:** A raster's alt text: the `wp:docPr` description or title.
- **Calls (crate):** `Element::find` (markdown/from_docx/ooxml.rs:396) L415; `alt_text` (markdown/from_docx/drawing.rs:428) L418; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L419,422
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.or_else`, `.trim`, `.unwrap_or`, `String::new`
- **Called by:** `label` (markdown/from_docx/drawing.rs:399); `fill` (markdown/from_docx/drawing.rs:929)

#### `alt_text`  (L428-434)
`fn alt_text(text: &str) -> String`

- **Does:** `text` as a Markdown alt text: one line, brackets escaped.
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.replace`, `.split_whitespace`
- **Called by:** `alt` (markdown/from_docx/drawing.rs:414)

#### `scope_of`  (L437-450, pub(crate))
`pub(crate) fn scope_of(element: &Element) -> Scope`

- **Does:** The `xmlns`/`xmlns:p` attributes `element` declares, `(prefix, uri)`.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.iter`, `.map`, `.strip_prefix`, `.to_string`, `.unwrap_or`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:174); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `ScopeMap::scope` (markdown/from_docx/drawing.rs:200); `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:472); `symbolize` (markdown/from_docx/drawing.rs:660)

#### `merge`  (L453-460)
`fn merge(scope: &mut Scope, later: &Scope)`

- **Does:** `later` wins over `scope` on the same prefix.
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.iter_mut`, `.push`
- **Called by:** `merged` (markdown/from_docx/drawing.rs:462); `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:472)

#### `merged`  (L462-466)
`fn merged(scope: &Scope, later: &Scope) -> Scope`

- **Does:** _no doc comment_
- **Calls (crate):** `merge` (markdown/from_docx/drawing.rs:453) L464
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `ScopeMap::scope` (markdown/from_docx/drawing.rs:200); `symbolize` (markdown/from_docx/drawing.rs:660)

#### `tree_scope`  (L470-480)
`fn tree_scope(root: &Element) -> Scope`

- **Does:** The union of every `xmlns` scope in the tree: a dependent part is read whole, so its own declarations are enough.
- **Calls (crate):** `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:472) L478
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:592)

#### `tree_scope::{fn collect}`  (L472-477)
`fn collect(element: &Element, scope: &mut Scope)`

- **Does:** _no doc comment_
- **Calls (crate):** `merge` (markdown/from_docx/drawing.rs:453) L473; `scope_of` (markdown/from_docx/drawing.rs:437) L473; `Element::elements` (markdown/from_docx/ooxml.rs:372) L474; `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:472) L475
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217) (?); `tree_scope` (markdown/from_docx/drawing.rs:470); `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:472)

#### `is_rel_name`  (L483-490)
`fn is_rel_name(name: &str, scope: &Scope) -> bool`

- **Does:** `name` is `prefix:local` whose prefix binds to `REL_NS`.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.iter`, `.split_once`
- **Called by:** `symbolize` (markdown/from_docx/drawing.rs:660); `tests::rel_names_match_only_the_relationship_namespace` (markdown/from_docx/drawing.rs:2068)

#### `declare`  (L495-533)
`fn declare(root: &mut Element, scope: &Scope)`

- **Does:** Declare on `root` every prefix the tree uses that no element inside it declares (and every prefix `mc:Requires` names), so the spliced element is self-contained.
- **Calls (crate):** `declare::{fn visit}` (markdown/from_docx/drawing.rs:498) L521
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.find`, `.into_iter`, `.iter`, `.push`, `.sort`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `shape_shell` (markdown/from_docx/drawing.rs:331); `Deps::dep_for` (markdown/from_docx/drawing.rs:592); `resolve` (markdown/from_docx/drawing.rs:704); `tests::declares_prefixes_the_tree_uses` (markdown/from_docx/drawing.rs:2047)

#### `declare::{fn visit}`  (L498-520)
`fn visit(element: &Element, declared: &mut HashSet<String>, used: &mut HashSet<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L517; `declare::{fn visit}` (markdown/from_docx/drawing.rs:498) L518
- **Calls (std/external/unresolved):** `.extend`, `.insert`, `.is_some_and`, `.map`, `.split_once`, `.split_whitespace`, `.strip_prefix`, `.to_string`, `String::new`
- **Called by:** `visit_bookmarked_paras` (convert/mod.rs:11834) (?); `declare` (markdown/from_docx/drawing.rs:495); `declare::{fn visit}` (markdown/from_docx/drawing.rs:498); `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) (?); `Dom::for_each_descendant_and_self` (xmllinq/mod.rs:753) (?)

#### `serialize`  (L538-560, pub(crate))
`pub(crate) fn serialize(element: &Element) -> String`

- **Does:** `element` as XML: self-closing when childless, attributes and text escaped.
- **Calls (crate):** `serialize::{fn put}` (markdown/from_docx/drawing.rs:540) L558
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `shape_shell` (markdown/from_docx/drawing.rs:331); `Deps::dep_for` (markdown/from_docx/drawing.rs:592); `resolve` (markdown/from_docx/drawing.rs:704); `canonical` (markdown/from_docx/drawing.rs:1093); `fill_boxes` (markdown/from_docx/drawing.rs:1619); `fill_boxes::{fn fallback}` (markdown/from_docx/drawing.rs:1625); `tests::xml_round_trips_verbatim` (markdown/from_docx/drawing.rs:2040); `tests::declares_prefixes_the_tree_uses` (markdown/from_docx/drawing.rs:2047); `tests::unchanged_blank_box_keeps_controls_and_remains_editable` (markdown/from_docx/drawing.rs:2101); `tests::doc_pr_ids_normalize_and_renumber` (markdown/from_docx/drawing.rs:2135); `patch` (markdown/from_docx/mermaid.rs:376)

#### `serialize::{fn put}`  (L540-557)
`fn put(element: &Element, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/xml.rs:601) L543,553; `serialize::{fn put}` (markdown/from_docx/drawing.rs:540) L552
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`
- **Macros:** `write!`
- **Called by:** `tests::tiny_docx` (fields.rs:1466) (?); `serialize` (markdown/from_docx/drawing.rs:538); `serialize::{fn put}` (markdown/from_docx/drawing.rs:540)

#### `normalize_ids`  (L565-578)
`fn normalize_ids(element: &mut Element)`

- **Does:** `wp:docPr` drawing ids are the writer's to assign: capture normalizes them to `0` so the payload does not change when the written file gives them fresh numbers.
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:313) L566; `normalize_ids` (markdown/from_docx/drawing.rs:565) L575
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `normalize_ids` (markdown/from_docx/drawing.rs:565); `Deps::dep_for` (markdown/from_docx/drawing.rs:592); `resolve` (markdown/from_docx/drawing.rs:704); `tests::doc_pr_ids_normalize_and_renumber` (markdown/from_docx/drawing.rs:2135)

#### `Deps::dep_for`  (L592-654)
`fn dep_for(&mut self, package: &mut Package<'_>, target: &str) -> Option<usize>`

- **Does:** The index of `target`'s part in `parts`, reading and capturing it on first use; `None` when the part is absent.
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:105)) L600; `Package::rels` (markdown/from_docx/ooxml.rs:142) L611; `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444) L620; `tree_scope` (markdown/from_docx/drawing.rs:470) L622; `symbolize` (markdown/from_docx/drawing.rs:660) L624; `normalize_ids` (markdown/from_docx/drawing.rs:565) L625; `declare` (markdown/from_docx/drawing.rs:495) L626; `serialize` (markdown/from_docx/drawing.rs:538) L627; `base64_encode` (markdown/from_docx/drawing.rs:1417) L648
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.filter`, `.get`, `.insert`, `.is_empty`, `.len`, `.next`, `.or_else`, `.push`, `.rsplit`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Dep::default`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `symbolize` (markdown/from_docx/drawing.rs:660)

#### `symbolize`  (L660-700)
`fn symbolize( package: &mut Package<'_>, deps: &mut Deps<'_>, element: &mut Element, scope: &Scope, rels: &Rels, sym: &str, out: &mut Vec<PayRel>, )`

- **Does:** Symbolize every relationship id `element` and its descendants name: attributes whose prefix binds to `REL_NS` in `scope`.
- **Calls (crate):** `is_rel_name` (markdown/from_docx/drawing.rs:483) L670; `Rels::get` (markdown/from_docx/ooxml.rs:245) L673; `Deps::dep_for` (markdown/from_docx/drawing.rs:592) L686; `merged` (markdown/from_docx/drawing.rs:462) L694; `scope_of` (markdown/from_docx/drawing.rs:437) L694; `symbolize` (markdown/from_docx/drawing.rs:660) L697
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.get_or_insert_with`, `.len`, `.push`, `PayRel::default`, `Some`
- **Macros:** `format!`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:592); `symbolize` (markdown/from_docx/drawing.rs:660); `resolve` (markdown/from_docx/drawing.rs:704)

#### `resolve`  (L704-746, pub(crate))
`pub(crate) fn resolve( package: &mut Package<'_>, types: &HashMap<String, String>, pending: &Pending, ) -> Option<Drawing>`

- **Does:** Capture `pending`: its element verbatim but for the relationship ids it symbolizes, with every part they reach.
- **Calls (crate):** `Package::rels` (markdown/from_docx/ooxml.rs:142) L709; `symbolize` (markdown/from_docx/drawing.rs:660) L718; `media_slot` (markdown/from_docx/drawing.rs:790) L730; `chart_mermaid` (markdown/from_docx/drawing.rs:753) L731; `normalize_ids` (markdown/from_docx/drawing.rs:565) L733; `declare` (markdown/from_docx/drawing.rs:495) L734; `strip_box_words` (markdown/from_docx/drawing.rs:833) L735; `serialize` (markdown/from_docx/drawing.rs:538) L737
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.is_some`, `.map`, `.ok`, `HashMap::new`, `Some`, `Vec::new`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929)

#### `chart_mermaid`  (L753-764)
`fn chart_mermaid(deps: &Deps<'_>) -> Option<String>`

- **Does:** The chart part's Mermaid text when the payload holds a `chartSpace` `pie`/`xychart-beta` can say: a standalone drawing prints it as a ` ```mermaid ` fence and `mhash` says whether the text came back unchanged.
- **Calls (crate):** `base64_decode` (markdown/from_docx/drawing.rs:1422) L759; `chart` (markdown/from_docx/mermaid.rs:89) L763
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.find`, `.is_empty`, `.iter`, `.ok`, `String::from_utf8`
- **Called by:** `resolve` (markdown/from_docx/drawing.rs:704)

#### `mermaid_hash`  (L767-773)
`fn mermaid_hash(text: &str) -> String`

- **Does:** SHA-256 of `text`, hex: the hash `mhash` carries for a fence's text.
- **Calls (std/external/unresolved):** `.as_bytes`, `.collect`, `.iter`, `.map`, `.trim`, `Sha256::digest`
- **Macros:** `format!`
- **Called by:** `box_hash` (markdown/from_docx/drawing.rs:776); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728); `Drawing::mermaid_unchanged` (markdown/from_docx/drawing.rs:1851); `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1858)

#### `box_hash`  (L776-783)
`fn box_hash(text: &str) -> String`

- **Does:** Ignore line-ending and per-line trailing whitespace in editable box stories.
- **Calls (crate):** `mermaid_hash` (markdown/from_docx/drawing.rs:767) L782
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.lines`, `.map`
- **Called by:** `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `Drawing::check_box_regions` (markdown/from_docx/drawing.rs:1569); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `media_slot`  (L790-829)
`fn media_slot(root: &Element, rels: &[PayRel], deps: &Deps<'_>) -> Option<usize>`

- **Does:** The image part the readable layer carries for `root`, as a `deps` index: an `asvg:svgBlip`'s SVG source, else the `a:blip`'s PNG, JPEG, GIF, BMP or TIFF raster.
- **Calls (crate):** `Element::find` (markdown/from_docx/ooxml.rs:396) L791,809,822
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.contains`, `.find_map`, `.find`, `.get`, `.is_some`, `.iter`, `.len`, `.starts_with`, `.then_some`, `Some`, `dep_of`
- **Called by:** `resolve` (markdown/from_docx/drawing.rs:704)

#### `strip_box_words`  (L833-887)
`fn strip_box_words(element: &mut Element, stories: &[String]) -> Vec<BoxFrame>`

- **Does:** Keep the box's structure, with text slots instead of words.
- **Calls (crate):** `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L885
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resolve` (markdown/from_docx/drawing.rs:704); `tests::unchanged_blank_box_keeps_controls_and_remains_editable` (markdown/from_docx/drawing.rs:2101)

#### `strip_box_words::{fn strip}`  (L834-855)
`fn strip(element: &mut Element, lengths: &mut Vec<usize>, text: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L838,843; `Element::text` (markdown/from_docx/ooxml.rs:415) L844; `strip_box_words::{fn strip}` (markdown/from_docx/drawing.rs:834) L851
- **Calls (std/external/unresolved):** `.clear`, `.len`, `.push_str`, `.push`
- **Macros:** `matches!`
- **Called by:** `step_h` (comparer/lcs.rs:3621) (?); `align_content_control_ancestry` (comparer/produce.rs:331) (?); `strip_box_words::{fn strip}` (markdown/from_docx/drawing.rs:834); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856)

#### `strip_box_words::{fn walk}`  (L856-883)
`fn walk(element: &mut Element, stories: &[String], boxes: &mut Vec<BoxFrame>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L857; `box_hash` (markdown/from_docx/drawing.rs:776) L862; `owned_drawings` (markdown/from_docx/drawing.rs:311) L867; `Element::text` (markdown/from_docx/ooxml.rs:415) L867; `strip_box_words::{fn strip}` (markdown/from_docx/drawing.rs:834) L873; `mermaid_hash` (markdown/from_docx/drawing.rs:767) L874; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L880
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `.len`, `.max`, `.push`, `.trim`, `BoxFrame::default`, `String::new`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:174); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `owned_drawings` (markdown/from_docx/drawing.rs:311); `owned_drawings::{fn walk}` (markdown/from_docx/drawing.rs:312); `strip_box_words` (markdown/from_docx/drawing.rs:833); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `box_ordinals` (markdown/from_docx/drawing.rs:1526); `box_ordinals::{fn walk}` (markdown/from_docx/drawing.rs:1527); `fill_boxes` (markdown/from_docx/drawing.rs:1619); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `content_types`  (L891-915, pub(crate))
`pub(crate) fn content_types(package: &mut Package<'_>) -> HashMap<String, String>`

- **Does:** The `[Content_Types]` entries of `package`: `/part` for each override, `.ext` for each default.
- **Calls (crate):** `Package::xml` (markdown/from_docx/ooxml.rs:132) L893; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L894; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L895; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L898,905
- **Calls (std/external/unresolved):** `.insert`, `.to_string`, `HashMap::new`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `fill`  (L929-1068, pub(crate))
`pub(crate) fn fill( markdown: &str, package: &mut Package<'_>, types: &HashMap<String, String>, pendings: &[Pending], ) -> String`

- **Does:** `markdown` with its `![label][dN]` references given their readable form and their payloads appended at the view's end.
- **Calls (crate):** `resolve` (markdown/from_docx/drawing.rs:704) L964; `standalone` (markdown/from_docx/drawing.rs:1323) L980,1001; `label` (markdown/from_docx/drawing.rs:399) L982,1005; `fenced` (markdown/from_docx/drawing.rs:1341) L988,1011; `encode` (markdown/from_docx/drawing.rs:1411) L991,995,1017,1025,1041,1052; `base64_decode` (markdown/from_docx/drawing.rs:1422) L1002; `Drawing::without_slot_bytes` (markdown/from_docx/drawing.rs:1824) (by name) L1017,1041; `canonical` (markdown/from_docx/drawing.rs:1093) L1030,1035; `regenerable` (markdown/from_docx/drawing.rs:1143) L1037; `relabel` (markdown/from_docx/drawing.rs:1073) L1055; `alt` (markdown/from_docx/drawing.rs:414) L1055
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.ends_with`, `.enumerate`, `.find`, `.get`, `.insert`, `.is_empty`, `.iter`, `.len`, `.push`, `.replace_range`, `.to_string`, `.trim`, `.wrapping_add_signed`, `HashMap::new`, `Some`, `String::new`, `Vec::new`, `find_token`, `str::from_utf8`
- **Macros:** `format!`, `write!`, `writeln!`
- **Called by:** `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (comparer/finalize.rs:28667) (?); `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (comparer/finalize.rs:31786) (?); `with_operation_targets` (edit/v2_apply.rs:97) (?); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `relabel`  (L1073-1086)
`fn relabel(out: &mut String, pos: usize, label: &str) -> isize`

- **Does:** Rewrite the label of the `![label][dN]` token ending at `pos`: the printable alt text a `docPr` name would have invented.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) L1077
- **Calls (std/external/unresolved):** `.len`, `.replace_range`, `.rfind`, `.starts_with`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929)

#### `canonical`  (L1093-1129)
`fn canonical(drawing: &Drawing) -> String`

- **Does:** The payload minus what use of it may differ in — `docPr` and `cNvPr` alt text and generated names, which the readable layer carries or the writer renumbers — so two drawings of one media payload share its `imageK` definition only while they splice the same XML.
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L1095; `canonical::{fn names}` (markdown/from_docx/drawing.rs:1098) L1113; `serialize` (markdown/from_docx/drawing.rs:538) L1114; `encode` (markdown/from_docx/drawing.rs:1411) L1128
- **Calls (std/external/unresolved):** `.as_bytes`, `.clear`, `.clone`, `.enumerate`, `.is_some`, `.iter_mut`, `Dep::default`, `Some`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929)

#### `canonical::{fn names}`  (L1098-1112)
`fn names(element: &mut Element)`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:313) L1099; `generated_name` (markdown/from_docx/drawing.rs:1133) L1101; `canonical::{fn names}` (markdown/from_docx/drawing.rs:1098) L1109
- **Calls (std/external/unresolved):** `.as_str`, `.clear`
- **Macros:** `matches!`
- **Called by:** `select` (audit.rs:239) (?); `family_faces_in_with` (convert/font.rs:2473) (?); `canonical` (markdown/from_docx/drawing.rs:1093); `canonical::{fn names}` (markdown/from_docx/drawing.rs:1098)

#### `generated_name`  (L1133-1136)
`fn generated_name(name: &str) -> bool`

- **Does:** A `docPr`/`cNvPr` name the writer gives the picture it numbers (`Picture 3`): no content to keep.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.strip_prefix`
- **Called by:** `canonical::{fn names}` (markdown/from_docx/drawing.rs:1098); `regenerable` (markdown/from_docx/drawing.rs:1143)

#### `regenerable`  (L1143-1310)
`fn regenerable(drawing: &Drawing) -> bool`

- **Does:** Whether the picture writer rebuilds `drawing` from its `data:` bytes, its alt text and the raster's size alone: a `wp:inline` of exactly the shape `drawing_xml` writes, at the size `read_picture` computes.
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L1150; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1153,1160,1171,1178,1185,1195...; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1156,1167,1191,1203,1204,1219...; `attrs_ok` (markdown/from_docx/drawing.rs:1313) L1161,1174,1179,1186,1201,1212...; `local` (markdown/from_docx/ooxml.rs:313) L1162,1174,1180,1212,1252,1277...; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1172,1173,1211,1275,1276,1284...; `generated_name` (markdown/from_docx/drawing.rs:1133) L1188,1231; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1198,1208,1216,1226,1289; `Element::rel_attr` (markdown/from_docx/ooxml.rs:365) (by name) L1251; `base64_decode` (markdown/from_docx/drawing.rs:1422) L1300; `read_picture` (markdown/write.rs:6479) L1303
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_slice`, `.as_str`, `.collect`, `.contains`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.len`, `.next`, `.nth`, `.to_string`, `Some`
- **Macros:** `matches!`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929)

#### `attrs_ok`  (L1313-1318)
`fn attrs_ok(element: &Element, ok: impl Fn(&str, &str) -> bool) -> bool`

- **Does:** `element`'s non-`xmlns` attributes all pass `ok`.
- **Calls (std/external/unresolved):** `.all`, `.iter`, `.starts_with`, `ok`
- **Called by:** `regenerable` (markdown/from_docx/drawing.rs:1143)

#### `standalone`  (L1323-1337)
`fn standalone(out: &str, pos: usize, end: usize) -> Option<(usize, usize)>`

- **Does:** The span of a `![label][dN]` token that is a whole line: its `![` opening and its line's end.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) L1325
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.find`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map_or`, `.rfind`, `.skip`, `.starts_with`, `.then_some`, `.trim`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929)

#### `fenced`  (L1341-1354)
`fn fenced(info: &str, text: &str) -> String`

- **Does:** `text` as a fenced code block carrying `info`, the fence one backtick wider than any run inside.
- **Calls (std/external/unresolved):** `.bytes`, `.max`, `.repeat`
- **Macros:** `format!`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929)

#### `at`  (L1360-1385, pub(crate))
`pub(crate) fn at(text: &str) -> Option<(&str, usize)>`

- **Does:** The Markdown drawing token at `text`'s start, if any — `![label][dN]`, or a `![label](drawing:PAYLOAD)` of a view an older build wrote.
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.find`, `.is_ascii_alphanumeric`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map`, `.or_else`, `.split_once`, `.strip_prefix`, `.then`
- **Macros:** `matches!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458) (?); `unit_lcs` (comparer/cross_para.rs:1384) (?); `char_weighted_lcs` (comparer/cross_para.rs:1996) (?); `wmf_object_tests::a_polypolygon_fills_with_the_brush_selected_by_zero_based_index` (convert/metafile.rs:1576) (?); `wmf_object_tests::a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen` (convert/metafile.rs:1641) (?); `wmf_object_tests::a_null_brush_rectangle_only_outlines` (convert/metafile.rs:1667) (?); `chrome_part_xml` (convert/mod.rs:20786) (?); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (convert/mod.rs:53494) (?); `inner_comment_references` (edit.rs:7556) (?); `closing_comment_references` (edit.rs:7595) (?); `escape_markdown` (markdown/from_docx/agent.rs:1751) (?); `relabel` (markdown/from_docx/drawing.rs:1073); +10 more

#### `cuts_token`  (L1388-1393, pub(crate))
`pub(crate) fn cuts_token(text: &str, cut: usize) -> bool`

- **Does:** Whether cutting `text` at `cut` splits a `![…](drawing:…)` token.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) L1392
- **Calls (std/external/unresolved):** `.is_some_and`, `.rfind`
- **Called by:** `tracked` (markdown/view_stories.rs:1393)

#### `decode`  (L1397-1409, pub(crate))
`pub(crate) fn decode(dest: &str) -> Result<Drawing, String>`

- **Does:** Decode the payload of `![label](drawing:PAYLOAD)`; `dest` is what [`at`] returns (`drawing:…`).
- **Calls (std/external/unresolved):** `.decode`, `.map_err`, `.ok_or_else`, `.or_else`, `.strip_prefix`, `.to_string`, `.trim_end_matches`, `serde_json::from_slice`
- **Macros:** `format!`
- **Called by:** `lookup` (markdown/from_docx/drawing.rs:1460); `tests::image_names_keep_delimiter_characters_out_of_the_markup` (markdown/from_docx/mod.rs:5520); `Writer::end` (markdown/write.rs:2398); `Writer::picture` (markdown/write.rs:4308)

#### `encode`  (L1411-1415, pub(crate))
`pub(crate) fn encode(drawing: &Drawing) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.encode`, `.unwrap_or_default`, `serde_json::to_vec`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929); `canonical` (markdown/from_docx/drawing.rs:1093)

#### `base64_encode`  (L1417-1420)
`fn base64_encode(bytes: &[u8]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.encode`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:592)

#### `base64_decode`  (L1422-1427, pub(crate))
`pub(crate) fn base64_decode(encoded: &str) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.decode`, `.ok`
- **Called by:** `chart_mermaid` (markdown/from_docx/drawing.rs:753); `fill` (markdown/from_docx/drawing.rs:929); `regenerable` (markdown/from_docx/drawing.rs:1143); `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1858); `dep_part` (markdown/from_docx/drawing.rs:1956); `Writer::picture` (markdown/write.rs:4308)

#### `data_uri`  (L1431-1436, pub(crate))
`pub(crate) fn data_uri(dest: &str) -> Option<(&str, &str)>`

- **Does:** A `data:` destination's `(mime, base64)` parts: `data:mime;base64,…`.
- **Calls (std/external/unresolved):** `.is_empty`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.then_some`
- **Called by:** `lookup` (markdown/from_docx/drawing.rs:1460); `Writer::picture` (markdown/write.rs:4308)

#### `backing_comments`  (L1440-1454, pub(crate))
`pub(crate) fn backing_comments(source: &str) -> HashMap<String, String>`

- **Does:** The `<!-- NAME drawing:PAYLOAD -->` backing comments of a view's drawings appendix, `NAME` → the `drawing:` destination.
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.is_empty`, `.lines`, `.split_once`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.then`, `.to_string`, `.trim`
- **Called by:** `read` (markdown/write.rs:137)

#### `lookup`  (L1460-1479, pub(crate))
`pub(crate) fn lookup( key_or_dest: &str, defs: &HashMap<String, String>, backings: &HashMap<String, String>, ) -> Option<Drawing>`

- **Does:** The `Drawing` a reference name or destination in the view resolves to: a `drawing:` destination or `dN` definition as it is; an `imageK` name through its backing comment, the `data:` definition's bytes filling the slot the readable layer carries.
- **Calls (crate):** `decode` (markdown/from_docx/drawing.rs:1397) L1471,1474; `data_uri` (markdown/from_docx/drawing.rs:1431) L1473; `Drawing::fill_slot_data` (markdown/from_docx/drawing.rs:1837) (by name) L1475
- **Calls (std/external/unresolved):** `.get`, `.map`, `.ok`, `.or_else`, `.starts_with`, `.then_some`
- **Called by:** `story_part` (markdown/view_stories.rs:1710); `Writer::picture` (markdown/write.rs:4308)

#### `holds_drawing`  (L1483-1486, pub(crate))
`pub(crate) fn holds_drawing(element: &Element) -> bool`

- **Does:** Whether `element` or a descendant holds a drawing: `w:drawing`, `w:pict` or `w:object` anywhere below `AlternateContent`'s `Choice`/`Fallback`.
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1484; `Element::elements` (markdown/from_docx/ooxml.rs:372) L1485
- **Calls (std/external/unresolved):** `.any`
- **Macros:** `matches!`
- **Called by:** `capturable_alternate` (markdown/from_docx/drawing.rs:1516); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923)

#### `is_plain_inline`  (L1491-1497, pub(crate))
`pub(crate) fn is_plain_inline(element: &Element) -> bool`

- **Does:** Whether `element` is a plain inline picture the `![alt](name)` syntax keeps when its raster is extracted: `wp:inline` holding an `a:blip`, no floating anchor, no text box the payload would carry with it.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L1492; `Element::find` (markdown/from_docx/ooxml.rs:396) L1493,1494,1495,1496
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3572)

#### `holds_block_content`  (L1504-1512)
`fn holds_block_content(element: &Element) -> bool`

- **Does:** `element` holds paragraph-level content (`w:p`, `w:tbl`, `w:sdt`) outside a drawing, picture or text box — where it is encapsulated and stays inside the run whatever it holds.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1505; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1506,1509; `holds_block_content` (markdown/from_docx/drawing.rs:1504) L1509
- **Calls (std/external/unresolved):** `.any`
- **Macros:** `matches!`
- **Called by:** `holds_block_content` (markdown/from_docx/drawing.rs:1504); `capturable_alternate` (markdown/from_docx/drawing.rs:1516)

#### `capturable_alternate`  (L1516-1523, pub(crate))
`pub(crate) fn capturable_alternate(element: &Element) -> bool`

- **Does:** Whether a paragraph-level `mc:AlternateContent` may be captured whole: one with a drawing and only run-level content below its `Choice`.
- **Calls (crate):** `holds_drawing` (markdown/from_docx/drawing.rs:1483) L1517; `Element::elements` (markdown/from_docx/ooxml.rs:372) L1520; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1521; `holds_block_content` (markdown/from_docx/drawing.rs:1504) L1521
- **Calls (std/external/unresolved):** `.all`
- **Macros:** `matches!`
- **Called by:** `owned_drawings::{fn walk}` (markdown/from_docx/drawing.rs:312); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `box_ordinals`  (L1526-1547)
`fn box_ordinals(root: &Element) -> Vec<usize>`

- **Does:** Box ordinals follow the selected branch; its fallback repeats those ordinals.
- **Calls (crate):** `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L1545
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `nested_pairs` (markdown/from_docx/drawing.rs:347); `Drawing::box_count` (markdown/from_docx/drawing.rs:1550); `Drawing::check_box_regions` (markdown/from_docx/drawing.rs:1569); `fill_boxes` (markdown/from_docx/drawing.rs:1619)

#### `box_ordinals::{fn walk}`  (L1527-1543)
`fn walk(element: &Element, next: &mut usize, out: &mut Vec<usize>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L1528,1531; `Element::elements` (markdown/from_docx/ooxml.rs:372) L1533,1539; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L1535,1540
- **Calls (std/external/unresolved):** `.max`, `.push`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:174); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `owned_drawings` (markdown/from_docx/drawing.rs:311); `owned_drawings::{fn walk}` (markdown/from_docx/drawing.rs:312); `strip_box_words` (markdown/from_docx/drawing.rs:833); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `box_ordinals` (markdown/from_docx/drawing.rs:1526); `box_ordinals::{fn walk}` (markdown/from_docx/drawing.rs:1527); `fill_boxes` (markdown/from_docx/drawing.rs:1619); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `Drawing::box_count`  (L1550-1555, pub(crate))
`pub(crate) fn box_count(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L1551; `box_ordinals` (markdown/from_docx/drawing.rs:1526) L1553
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.into_iter`, `.map_or`, `.max`, `.ok`
- **Called by:** `Writer::finish` (markdown/write.rs:5162) (?)

#### `Drawing::nested_tokens`  (L1557-1567, pub(crate))
`pub(crate) fn nested_tokens(&self) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L1558; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L1562; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1565
- **Calls (std/external/unresolved):** `.as_bytes`, `.collect`, `.filter_map`, `.into_iter`, `.map`, `Vec::new`
- **Called by:** `Writer::finish` (markdown/write.rs:5162) (?)

#### `Drawing::check_box_regions`  (L1569-1614, pub(crate))
`pub(crate) fn check_box_regions( &self, name: &str, regions: &HashMap<String, String>, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L1577; `Drawing::check_box_regions::{fn templates}` (markdown/from_docx/drawing.rs:1580) L1590; `box_ordinals` (markdown/from_docx/drawing.rs:1526) L1591; `box_hash` (markdown/from_docx/drawing.rs:776) L1606
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.contains_key`, `.find`, `.get`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.zip`, `Err`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::finish` (markdown/write.rs:5162) (?)

#### `Drawing::check_box_regions::{fn templates}`  (L1580-1588)
`fn templates<'a>(element: &'a Element, out: &mut Vec<&'a Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L1581; `Element::elements` (markdown/from_docx/ooxml.rs:372) L1584; `Drawing::check_box_regions::{fn templates}` (markdown/from_docx/drawing.rs:1580) L1585
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Drawing::check_box_regions` (markdown/from_docx/drawing.rs:1569); `Drawing::check_box_regions::{fn templates}` (markdown/from_docx/drawing.rs:1580)

#### `fill_boxes`  (L1619-1807, pub(crate))
`pub(crate) fn fill_boxes( xml: &str, drawing: &Drawing, stories: &[(String, String)], next_doc_pr: &mut u32, ) -> String`

- **Does:** Refill the text-free templates.
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L1793; `box_ordinals` (markdown/from_docx/drawing.rs:1526) L1796; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L1797; `serialize` (markdown/from_docx/drawing.rs:538) L1806
- **Calls (std/external/unresolved):** `.as_bytes`, `.to_string`
- **Called by:** `tests::unchanged_blank_box_keeps_controls_and_remains_editable` (markdown/from_docx/drawing.rs:2101); `<PartRelate as Relate>::drawing` (markdown/package.rs:1747)

#### `fill_boxes::{fn fallback}`  (L1625-1646)
`fn fallback(element: &Element, next: &mut u32) -> Vec<Node>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L1626; `Element::is` (markdown/from_docx/ooxml.rs:322) L1628; `serialize` (markdown/from_docx/drawing.rs:538) L1639; `renumber_doc_pr` (markdown/from_docx/drawing.rs:1928) L1640; `parse_xml` (markdown/from_docx/ooxml.rs:440) L1641
- **Calls (std/external/unresolved):** `.as_bytes`, `.clone`, `.collect`, `.expect`, `.filter`, `.into_iter`, `.map_or_else`, `.map`, `Node::Element`
- **Macros:** `vec!`
- **Called by:** `fill_boxes::{fn splice}` (markdown/from_docx/drawing.rs:1647); `fill_boxes::{fn fallback_story}` (markdown/from_docx/drawing.rs:1669)

#### `fill_boxes::{fn splice}`  (L1647-1668)
`fn splice(element: &mut Element, nested: &HashMap<String, Element>, next: &mut u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1651; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1652,1653; `fill_boxes::{fn fallback}` (markdown/from_docx/drawing.rs:1625) L1654; `fill_boxes::{fn splice}` (markdown/from_docx/drawing.rs:1647) L1660
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.extend`, `.get`, `.push`, `Node::Element`, `Some`, `Vec::new`, `std::mem::take`
- **Called by:** `fill_boxes::{fn splice}` (markdown/from_docx/drawing.rs:1647); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `fill_boxes::{fn fallback_story}`  (L1669-1687)
`fn fallback_story(element: &mut Element, next: &mut u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1674; `fill_boxes::{fn fallback}` (markdown/from_docx/drawing.rs:1625) L1677; `fill_boxes::{fn fallback_story}` (markdown/from_docx/drawing.rs:1669) L1679
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Node::Element`, `Vec::new`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `fill_boxes::{fn fallback_story}` (markdown/from_docx/drawing.rs:1669); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `fill_boxes::{fn words}`  (L1688-1702)
`fn words(element: &Element, text: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1690,1695; `Element::text` (markdown/from_docx/ooxml.rs:415) L1696; `Element::elements` (markdown/from_docx/ooxml.rs:372) L1698; `fill_boxes::{fn words}` (markdown/from_docx/drawing.rs:1688) L1699
- **Calls (std/external/unresolved):** `.push_str`
- **Macros:** `matches!`
- **Called by:** `fill_boxes::{fn words}` (markdown/from_docx/drawing.rs:1688); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `fill_boxes::{fn refill}`  (L1703-1727)
`fn refill( element: &mut Element, lengths: &mut impl Iterator<Item = usize>, text: &mut &str, ) -> Option<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1709,1714; `fill_boxes::{fn refill}` (markdown/from_docx/drawing.rs:1703) L1722
- **Calls (std/external/unresolved):** `.get`, `.next`, `.to_string`, `Node::Text`, `Some`
- **Macros:** `matches!`, `vec!`
- **Called by:** `fill_boxes::{fn refill}` (markdown/from_docx/drawing.rs:1703); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `fill_boxes::{fn walk}`  (L1728-1792)
`fn walk( element: &mut Element, drawing: &Drawing, stories: &[(String, String)], at: &mut usize, ordinals: &[usize], in_fallback: bool, next: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L1737,1786; `parse_xml` (markdown/from_docx/ooxml.rs:440) L1743; `fill_boxes::{fn words}` (markdown/from_docx/drawing.rs:1688) L1747; `box_hash` (markdown/from_docx/drawing.rs:776) L1751,1756; `mermaid_hash` (markdown/from_docx/drawing.rs:767) L1757; `fill_boxes::{fn refill}` (markdown/from_docx/drawing.rs:1703) L1760; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L1768; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1771; `owned_drawings` (markdown/from_docx/drawing.rs:311) L1772; `fill_boxes::{fn splice}` (markdown/from_docx/drawing.rs:1647) L1775; `fill_boxes::{fn fallback_story}` (markdown/from_docx/drawing.rs:1669) L1781; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177) L1789
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_str`, `.clone`, `.collect`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.to_string`, `.zip`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:174); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:177); `owned_drawings` (markdown/from_docx/drawing.rs:311); `owned_drawings::{fn walk}` (markdown/from_docx/drawing.rs:312); `strip_box_words` (markdown/from_docx/drawing.rs:833); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `box_ordinals` (markdown/from_docx/drawing.rs:1526); `box_ordinals::{fn walk}` (markdown/from_docx/drawing.rs:1527); `fill_boxes` (markdown/from_docx/drawing.rs:1619); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728)

#### `Drawing::wraps_run`  (L1818-1820, pub(crate))
`pub(crate) fn wraps_run(&self) -> bool`

- **Does:** The element goes inside a `w:r` (`w:drawing`, `w:pict`, run-level AlternateContent), not where a `w:r` may sit (paragraph-level AlternateContent).
- **Calls:** none
- **Called by:** `story_part` (markdown/view_stories.rs:1710) (?); `write_run` (markdown/xml.rs:2227) (?)

#### `Drawing::without_slot_bytes`  (L1824-1833)
`fn without_slot_bytes(&self) -> Drawing`

- **Does:** `self` with the readable layer's bytes out of the payload: the backing comment of a `data:` definition or an `svg` fence.
- **Calls (std/external/unresolved):** `.clear`, `.clone`, `.get_mut`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:929) (?)

#### `Drawing::fill_slot_data`  (L1837-1847, pub(crate))
`pub(crate) fn fill_slot_data(&mut self, b64: &str, kind: &str) -> bool`

- **Does:** Fill the slot with `b64` bytes and, when `kind` differs, its content type; `false` when the payload carries no slot.
- **Calls (std/external/unresolved):** `.and_then`, `.clear`, `.get_mut`, `.to_string`
- **Called by:** `lookup` (markdown/from_docx/drawing.rs:1460) (?); `Writer::end` (markdown/write.rs:2398) (?)

#### `Drawing::mermaid_unchanged`  (L1851-1853, pub(crate))
`pub(crate) fn mermaid_unchanged(&self, text: &str) -> bool`

- **Does:** `text` is the ` ```mermaid ` fence this drawing printed: its hash is `mhash`'s, so the drawing writes back byte for byte.
- **Calls (crate):** `mermaid_hash` (markdown/from_docx/drawing.rs:767) L1852
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `Some`
- **Called by:** `Writer::end` (markdown/write.rs:2398) (?)

#### `Drawing::rebuild_mermaid`  (L1858-1878, pub(crate))
`pub(crate) fn rebuild_mermaid(&mut self, text: &str) -> bool`

- **Does:** The fence's text edited: patch the chart part it names in place, keeping everything the Mermaid cannot say.
- **Calls (crate):** `base64_decode` (markdown/from_docx/drawing.rs:1422) L1867; `patch` (markdown/from_docx/mermaid.rs:376) L1871; `mermaid_hash` (markdown/from_docx/drawing.rs:767) L1876
- **Calls (std/external/unresolved):** `.and_then`, `.clear`, `.clone`, `.ends_with`, `.find`, `.is_empty`, `.iter_mut`, `.ok`, `Some`, `String::from_utf8`
- **Called by:** `Writer::end` (markdown/write.rs:2398) (?)

#### `materialize`  (L1881-1905, pub(crate))
`pub(crate) fn materialize( drawing: &Drawing, package: &mut PartFs, part: &str, existing: &HashSet<String>, used: &mut HashSet<String>, next_doc_pr: &mut u32, ) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `dep_part` (markdown/from_docx/drawing.rs:1956) L1894; `relative_rel_target` (opc/mod.rs:180) L1895,1897; `relate` (markdown/from_docx/drawing.rs:1910) L1900; `renumber_doc_pr` (markdown/from_docx/drawing.rs:1928) L1903
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.replace`, `Some`
- **Macros:** `format!`
- **Called by:** `<PartRelate as Relate>::drawing` (markdown/package.rs:1747); `story_part` (markdown/view_stories.rs:1710)

#### `relate`  (L1910-1924)
`fn relate(package: &mut PartFs, part: &str, kind: &str, target: &str, external: bool) -> String`

- **Does:** A relationship `part` → `target` of `kind`, reusing one already written when there is one: two drawings that share a media part share its relationship too.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L1911; `PartFs::add_document_relationship_external` (opc/mod.rs:1047) L1920; `PartFs::add_document_relationship` (opc/mod.rs:1003) L1922
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.iter`
- **Called by:** `materialize` (markdown/from_docx/drawing.rs:1881); `dep_part` (markdown/from_docx/drawing.rs:1956)

#### `renumber_doc_pr`  (L1928-1952)
`fn renumber_doc_pr(xml: &mut String, next: &mut u32)`

- **Does:** Give every `docPr id` in `xml` a fresh number: the stored `0`s collide with each other and with pictures the writer numbers.
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.is_ascii_digit`, `.len`, `.push_str`, `.unwrap_or`, `String::with_capacity`
- **Macros:** `write!`
- **Called by:** `fill_boxes::{fn fallback}` (markdown/from_docx/drawing.rs:1625); `materialize` (markdown/from_docx/drawing.rs:1881); `dep_part` (markdown/from_docx/drawing.rs:1956); `tests::doc_pr_ids_normalize_and_renumber` (markdown/from_docx/drawing.rs:2135)

#### `dep_part`  (L1956-2008)
`fn dep_part( drawing: &Drawing, index: usize, package: &mut PartFs, existing: &HashSet<String>, used: &mut HashSet<String>, next_doc_pr: &mut u32, ) -> Option<String>`

- **Does:** Write dependent part `index` and its own dependency closure; its new part name.
- **Calls (crate):** `name_for` (markdown/from_docx/drawing.rs:2012) L1965; `dep_part` (markdown/from_docx/drawing.rs:1956) L1974; `relative_rel_target` (opc/mod.rs:180) L1977,1981; `relate` (markdown/from_docx/drawing.rs:1910) L1984; `base64_decode` (markdown/from_docx/drawing.rs:1422) L1990; `renumber_doc_pr` (markdown/from_docx/drawing.rs:1928) L1992; `PartFs::part_bytes` (opc/mod.rs:628) L1998; `PartFs::set_part` (opc/mod.rs:661) L2005; `PartFs::add_content_type_override` (opc/mod.rs:845) L2006
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.enumerate`, `.get`, `.insert`, `.into_bytes`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.replace`, `Some`
- **Macros:** `format!`
- **Called by:** `materialize` (markdown/from_docx/drawing.rs:1881); `dep_part` (markdown/from_docx/drawing.rs:1956)

#### `name_for`  (L2012-2028)
`fn name_for(want: &str, existing: &HashSet<String>, used: &HashSet<String>) -> String`

- **Does:** `want` when nothing holds it, else `want` with `-j2`, `-j3`, … before its extension.
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.rsplit_once`, `.to_string`, `String::new`, `free`
- **Macros:** `format!`
- **Called by:** `dep_part` (markdown/from_docx/drawing.rs:1956)

## Test functions (7)

- `tests::element` (L2035): _no doc comment_
- `tests::xml_round_trips_verbatim` (L2040): _no doc comment_
- `tests::declares_prefixes_the_tree_uses` (L2047): _no doc comment_
- `tests::rel_names_match_only_the_relationship_namespace` (L2068): _no doc comment_
- `tests::token_parses_destination_and_length` (L2082): _no doc comment_
- `tests::unchanged_blank_box_keeps_controls_and_remains_editable` (L2101): _no doc comment_
- `tests::doc_pr_ids_normalize_and_renumber` (L2135): _no doc comment_
