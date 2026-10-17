# Functions and methods in `src/markdown/from_docx/drawing.rs`

58 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `ScopeMap::of`  (L161-183, pub(crate))
`pub(crate) fn of(root: &Element) -> Self`

- **Does:** The scope map of a part's tree.
- **Calls (crate):** `scope_of` (markdown/from_docx/drawing.rs:283) L162; `Element::elements` (markdown/from_docx/ooxml.rs:301) L176; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:164) L177
- **Calls (std/external/unresolved):** `HashMap::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `ScopeMap::of::{fn walk}`  (L164-175)
`fn walk(element: &Element, scope: &Scope, map: &mut HashMap<*const Element, Scope>)`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L166; `merged` (markdown/from_docx/drawing.rs:308) L171; `scope_of` (markdown/from_docx/drawing.rs:283) L171; `Element::elements` (markdown/from_docx/ooxml.rs:301) L172; `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:164) L173
- **Calls (std/external/unresolved):** `.clone`, `.insert`
- **Macros:** `matches!`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:161); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:164)

#### `ScopeMap::scope`  (L187-194, pub(crate))
`pub(crate) fn scope(&self, element: &Element) -> Scope`

- **Does:** The scope frozen at `element`: its ancestors' declarations, then its own.
- **Calls (crate):** `merged` (markdown/from_docx/drawing.rs:308) L188; `scope_of` (markdown/from_docx/drawing.rs:283) L192
- **Calls (std/external/unresolved):** `.get`, `.unwrap_or`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::drawing` (markdown/from_docx/mod.rs:3366)

#### `Pending::push`  (L211-230, pub(crate))
`pub(crate) fn push( list: &mut Vec<Pending>, element: &Element, part: &str, scope: Scope, wrap: bool, ) -> String`

- **Does:** Queue `element`, printing its `![label][dN]` reference.
- **Calls (crate):** `label` (markdown/from_docx/drawing.rs:245) L218; `image` (markdown/from_docx/drawing.rs:236) L220,229
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.push`, `.to_string`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::drawing` (markdown/from_docx/mod.rs:3366)

#### `image`  (L236-242)
`fn image(label: &str, name: &str) -> String`

- **Does:** `![label][name]` with the label made a Markdown alt text: one line, brackets escaped.
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `format!`
- **Called by:** `tests::drawings_compare_their_placement_and_target` (debug/diff.rs:2039) (?); `coverage_boundary_tests::a_reused_header_keeps_its_own_relationships` (document_comparer.rs:10555) (?); `Pending::push` (markdown/from_docx/drawing.rs:211)

#### `label`  (L245-255)
`fn label(element: &Element) -> String`

- **Does:** The `wp:docPr` description, title or name as the token's label.
- **Calls (crate):** `alt` (markdown/from_docx/drawing.rs:260) L246; `Element::find` (markdown/from_docx/ooxml.rs:325) L250; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L252
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.map`, `.unwrap_or_default`
- **Called by:** `Pending::push` (markdown/from_docx/drawing.rs:211); `fill` (markdown/from_docx/drawing.rs:703); `context_view` (text_diff.rs:410) (?)

#### `alt`  (L260-271)
`fn alt(element: &Element) -> String`

- **Does:** A raster's alt text: the `wp:docPr` description or title.
- **Calls (crate):** `Element::find` (markdown/from_docx/ooxml.rs:325) L261; `alt_text` (markdown/from_docx/drawing.rs:274) L264; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L265,268
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.or_else`, `.trim`, `.unwrap_or`, `String::new`
- **Called by:** `label` (markdown/from_docx/drawing.rs:245); `fill` (markdown/from_docx/drawing.rs:703)

#### `alt_text`  (L274-280)
`fn alt_text(text: &str) -> String`

- **Does:** `text` as a Markdown alt text: one line, brackets escaped.
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.replace`, `.split_whitespace`
- **Called by:** `alt` (markdown/from_docx/drawing.rs:260)

#### `scope_of`  (L283-296, pub(crate))
`pub(crate) fn scope_of(element: &Element) -> Scope`

- **Does:** The `xmlns`/`xmlns:p` attributes `element` declares, `(prefix, uri)`.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.iter`, `.map`, `.strip_prefix`, `.to_string`, `.unwrap_or`
- **Called by:** `ScopeMap::of` (markdown/from_docx/drawing.rs:161); `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:164); `ScopeMap::scope` (markdown/from_docx/drawing.rs:187); `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:318); `symbolize` (markdown/from_docx/drawing.rs:504)

#### `merge`  (L299-306)
`fn merge(scope: &mut Scope, later: &Scope)`

- **Does:** `later` wins over `scope` on the same prefix.
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.iter_mut`, `.push`
- **Called by:** `merged` (markdown/from_docx/drawing.rs:308); `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:318)

#### `merged`  (L308-312)
`fn merged(scope: &Scope, later: &Scope) -> Scope`

- **Does:** _no doc comment_
- **Calls (crate):** `merge` (markdown/from_docx/drawing.rs:299) L310
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `ScopeMap::of::{fn walk}` (markdown/from_docx/drawing.rs:164); `ScopeMap::scope` (markdown/from_docx/drawing.rs:187); `symbolize` (markdown/from_docx/drawing.rs:504)

#### `tree_scope`  (L316-326)
`fn tree_scope(root: &Element) -> Scope`

- **Does:** The union of every `xmlns` scope in the tree: a dependent part is read whole, so its own declarations are enough.
- **Calls (crate):** `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:318) L324
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:438)

#### `tree_scope::{fn collect}`  (L318-323)
`fn collect(element: &Element, scope: &mut Scope)`

- **Does:** _no doc comment_
- **Calls (crate):** `merge` (markdown/from_docx/drawing.rs:299) L319; `scope_of` (markdown/from_docx/drawing.rs:283) L319; `Element::elements` (markdown/from_docx/ooxml.rs:301) L320; `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:318) L321
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217) (?); `tree_scope` (markdown/from_docx/drawing.rs:316); `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:318)

#### `is_rel_name`  (L329-336)
`fn is_rel_name(name: &str, scope: &Scope) -> bool`

- **Does:** `name` is `prefix:local` whose prefix binds to `REL_NS`.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.iter`, `.split_once`
- **Called by:** `symbolize` (markdown/from_docx/drawing.rs:504); `tests::rel_names_match_only_the_relationship_namespace` (markdown/from_docx/drawing.rs:1554)

#### `declare`  (L341-379)
`fn declare(root: &mut Element, scope: &Scope)`

- **Does:** Declare on `root` every prefix the tree uses that no element inside it declares (and every prefix `mc:Requires` names), so the spliced element is self-contained.
- **Calls (crate):** `declare::{fn visit}` (markdown/from_docx/drawing.rs:344) L367
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.find`, `.into_iter`, `.iter`, `.push`, `.sort`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:438); `resolve` (markdown/from_docx/drawing.rs:548); `tests::declares_prefixes_the_tree_uses` (markdown/from_docx/drawing.rs:1537)

#### `declare::{fn visit}`  (L344-366)
`fn visit(element: &Element, declared: &mut HashSet<String>, used: &mut HashSet<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L363; `declare::{fn visit}` (markdown/from_docx/drawing.rs:344) L364
- **Calls (std/external/unresolved):** `.extend`, `.insert`, `.is_some_and`, `.map`, `.split_once`, `.split_whitespace`, `.strip_prefix`, `.to_string`, `String::new`
- **Called by:** `visit_bookmarked_paras` (convert/mod.rs:11833) (?); `declare` (markdown/from_docx/drawing.rs:341); `declare::{fn visit}` (markdown/from_docx/drawing.rs:344); `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) (?); `Dom::for_each_descendant_and_self` (xmllinq/mod.rs:753) (?)

#### `serialize`  (L384-406, pub(crate))
`pub(crate) fn serialize(element: &Element) -> String`

- **Does:** `element` as XML: self-closing when childless, attributes and text escaped.
- **Calls (crate):** `serialize::{fn put}` (markdown/from_docx/drawing.rs:386) L404
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:438); `resolve` (markdown/from_docx/drawing.rs:548); `canonical` (markdown/from_docx/drawing.rs:867); `tests::xml_round_trips_verbatim` (markdown/from_docx/drawing.rs:1530); `tests::declares_prefixes_the_tree_uses` (markdown/from_docx/drawing.rs:1537); `tests::doc_pr_ids_normalize_and_renumber` (markdown/from_docx/drawing.rs:1587); `patch` (markdown/from_docx/mermaid.rs:374)

#### `serialize::{fn put}`  (L386-403)
`fn put(element: &Element, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/xml.rs:426) L389,399; `serialize::{fn put}` (markdown/from_docx/drawing.rs:386) L398
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`
- **Macros:** `write!`
- **Called by:** `tests::tiny_docx` (fields.rs:1466) (?); `serialize` (markdown/from_docx/drawing.rs:384); `serialize::{fn put}` (markdown/from_docx/drawing.rs:386)

#### `normalize_ids`  (L411-424)
`fn normalize_ids(element: &mut Element)`

- **Does:** `wp:docPr` drawing ids are the writer's to assign: capture normalizes them to `0` so the payload does not change when the written file gives them fresh numbers.
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:272) L412; `normalize_ids` (markdown/from_docx/drawing.rs:411) L421
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `normalize_ids` (markdown/from_docx/drawing.rs:411); `Deps::dep_for` (markdown/from_docx/drawing.rs:438); `resolve` (markdown/from_docx/drawing.rs:548); `tests::doc_pr_ids_normalize_and_renumber` (markdown/from_docx/drawing.rs:1587)

#### `Deps::dep_for`  (L438-498)
`fn dep_for(&mut self, package: &mut Package<'_>, target: &str) -> Option<usize>`

- **Does:** The index of `target`'s part in `parts`, reading and capturing it on first use; `None` when the part is absent.
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:66)) L446; `Package::rels` (markdown/from_docx/ooxml.rs:103) L457; `parse_xml` (markdown/from_docx/ooxml.rs:367) L465; `tree_scope` (markdown/from_docx/drawing.rs:316) L466; `symbolize` (markdown/from_docx/drawing.rs:504) L468; `normalize_ids` (markdown/from_docx/drawing.rs:411) L469; `declare` (markdown/from_docx/drawing.rs:341) L470; `serialize` (markdown/from_docx/drawing.rs:384) L471; `base64_encode` (markdown/from_docx/drawing.rs:1191) L492
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.filter`, `.get`, `.insert`, `.is_empty`, `.len`, `.next`, `.or_else`, `.push`, `.rsplit`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Dep::default`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `symbolize` (markdown/from_docx/drawing.rs:504)

#### `symbolize`  (L504-544)
`fn symbolize( package: &mut Package<'_>, deps: &mut Deps<'_>, element: &mut Element, scope: &Scope, rels: &Rels, sym: &str, out: &mut Vec<PayRel>, )`

- **Does:** Symbolize every relationship id `element` and its descendants name: attributes whose prefix binds to `REL_NS` in `scope`.
- **Calls (crate):** `is_rel_name` (markdown/from_docx/drawing.rs:329) L514; `Rels::get` (markdown/from_docx/ooxml.rs:206) L517; `Deps::dep_for` (markdown/from_docx/drawing.rs:438) L530; `merged` (markdown/from_docx/drawing.rs:308) L538; `scope_of` (markdown/from_docx/drawing.rs:283) L538; `symbolize` (markdown/from_docx/drawing.rs:504) L541
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.get_or_insert_with`, `.len`, `.push`, `PayRel::default`, `Some`
- **Macros:** `format!`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:438); `symbolize` (markdown/from_docx/drawing.rs:504); `resolve` (markdown/from_docx/drawing.rs:548)

#### `resolve`  (L548-588, pub(crate))
`pub(crate) fn resolve( package: &mut Package<'_>, types: &HashMap<String, String>, pending: &Pending, ) -> Option<Drawing>`

- **Does:** Capture `pending`: its element verbatim but for the relationship ids it symbolizes, with every part they reach.
- **Calls (crate):** `Package::rels` (markdown/from_docx/ooxml.rs:103) L553; `symbolize` (markdown/from_docx/drawing.rs:504) L562; `media_slot` (markdown/from_docx/drawing.rs:622) L574; `chart_mermaid` (markdown/from_docx/drawing.rs:595) L575; `normalize_ids` (markdown/from_docx/drawing.rs:411) L577; `declare` (markdown/from_docx/drawing.rs:341) L578; `serialize` (markdown/from_docx/drawing.rs:384) L580
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.is_some`, `.map`, `.ok`, `HashMap::new`, `Some`, `Vec::new`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703)

#### `chart_mermaid`  (L595-606)
`fn chart_mermaid(deps: &Deps<'_>) -> Option<String>`

- **Does:** The chart part's Mermaid text when the payload holds a `chartSpace` `pie`/`xychart-beta` can say: a standalone drawing prints it as a ` ```mermaid ` fence and `mhash` says whether the text came back unchanged.
- **Calls (crate):** `base64_decode` (markdown/from_docx/drawing.rs:1196) L601; `chart` (markdown/from_docx/mermaid.rs:89) L605
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.find`, `.is_empty`, `.iter`, `.ok`, `String::from_utf8`
- **Called by:** `resolve` (markdown/from_docx/drawing.rs:548)

#### `mermaid_hash`  (L609-615)
`fn mermaid_hash(text: &str) -> String`

- **Does:** SHA-256 of `text`, hex: the hash `mhash` carries for a fence's text.
- **Calls (std/external/unresolved):** `.as_bytes`, `.collect`, `.iter`, `.map`, `.trim`, `Sha256::digest`
- **Macros:** `format!`
- **Called by:** `Drawing::mermaid_unchanged` (markdown/from_docx/drawing.rs:1341); `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1348)

#### `media_slot`  (L622-661)
`fn media_slot(root: &Element, rels: &[PayRel], deps: &Deps<'_>) -> Option<usize>`

- **Does:** The image part the readable layer carries for `root`, as a `deps` index: an `asvg:svgBlip`'s SVG source, else the `a:blip`'s PNG, JPEG, GIF, BMP or TIFF raster.
- **Calls (crate):** `Element::find` (markdown/from_docx/ooxml.rs:325) L623,641,654
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.contains`, `.find_map`, `.find`, `.get`, `.is_some`, `.iter`, `.len`, `.starts_with`, `.then_some`, `Some`, `dep_of`
- **Called by:** `resolve` (markdown/from_docx/drawing.rs:548)

#### `content_types`  (L665-689, pub(crate))
`pub(crate) fn content_types(package: &mut Package<'_>) -> HashMap<String, String>`

- **Does:** The `[Content_Types]` entries of `package`: `/part` for each override, `.ext` for each default.
- **Calls (crate):** `Package::xml` (markdown/from_docx/ooxml.rs:93) L667; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L668; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L669; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L672,679
- **Calls (std/external/unresolved):** `.insert`, `.to_string`, `HashMap::new`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `fill`  (L703-842, pub(crate))
`pub(crate) fn fill( markdown: &str, package: &mut Package<'_>, types: &HashMap<String, String>, pendings: &[Pending], ) -> String`

- **Does:** `markdown` with its `![label][dN]` references given their readable form and their payloads appended at the view's end.
- **Calls (crate):** `resolve` (markdown/from_docx/drawing.rs:548) L738; `standalone` (markdown/from_docx/drawing.rs:1097) L754,775; `label` (markdown/from_docx/drawing.rs:245) L756,779; `fenced` (markdown/from_docx/drawing.rs:1115) L762,785; `encode` (markdown/from_docx/drawing.rs:1185) L765,769,791,799,815,826; `base64_decode` (markdown/from_docx/drawing.rs:1196) L776; `Drawing::without_slot_bytes` (markdown/from_docx/drawing.rs:1314) (by name) L791,815; `canonical` (markdown/from_docx/drawing.rs:867) L804,809; `regenerable` (markdown/from_docx/drawing.rs:917) L811; `relabel` (markdown/from_docx/drawing.rs:847) L829; `alt` (markdown/from_docx/drawing.rs:260) L829
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.ends_with`, `.enumerate`, `.find`, `.get`, `.insert`, `.is_empty`, `.iter`, `.len`, `.push`, `.replace_range`, `.to_string`, `.trim`, `.wrapping_add_signed`, `HashMap::new`, `Some`, `String::new`, `Vec::new`, `find_token`, `str::from_utf8`
- **Macros:** `format!`, `write!`, `writeln!`
- **Called by:** `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (comparer/finalize.rs:28648) (?); `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (comparer/finalize.rs:31772) (?); `convert` (markdown/from_docx/mod.rs:96)

#### `relabel`  (L847-860)
`fn relabel(out: &mut String, pos: usize, label: &str) -> isize`

- **Does:** Rewrite the label of the `![label][dN]` token ending at `pos`: the printable alt text a `docPr` name would have invented.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) L851
- **Calls (std/external/unresolved):** `.len`, `.replace_range`, `.rfind`, `.starts_with`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703)

#### `canonical`  (L867-903)
`fn canonical(drawing: &Drawing) -> String`

- **Does:** The payload minus what use of it may differ in — `docPr` and `cNvPr` alt text and generated names, which the readable layer carries or the writer renumbers — so two drawings of one media payload share its `imageK` definition only while they splice the same XML.
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:367) L869; `canonical::{fn names}` (markdown/from_docx/drawing.rs:872) L887; `serialize` (markdown/from_docx/drawing.rs:384) L888; `encode` (markdown/from_docx/drawing.rs:1185) L902
- **Calls (std/external/unresolved):** `.as_bytes`, `.clear`, `.clone`, `.enumerate`, `.is_some`, `.iter_mut`, `Dep::default`, `Some`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703)

#### `canonical::{fn names}`  (L872-886)
`fn names(element: &mut Element)`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:272) L873; `generated_name` (markdown/from_docx/drawing.rs:907) L875; `canonical::{fn names}` (markdown/from_docx/drawing.rs:872) L883
- **Calls (std/external/unresolved):** `.as_str`, `.clear`
- **Macros:** `matches!`
- **Called by:** `select` (audit.rs:239) (?); `family_faces_in_with` (convert/font.rs:2473) (?); `canonical` (markdown/from_docx/drawing.rs:867); `canonical::{fn names}` (markdown/from_docx/drawing.rs:872)

#### `generated_name`  (L907-910)
`fn generated_name(name: &str) -> bool`

- **Does:** A `docPr`/`cNvPr` name the writer gives the picture it numbers (`Picture 3`): no content to keep.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.strip_prefix`
- **Called by:** `canonical::{fn names}` (markdown/from_docx/drawing.rs:872); `regenerable` (markdown/from_docx/drawing.rs:917)

#### `regenerable`  (L917-1084)
`fn regenerable(drawing: &Drawing) -> bool`

- **Does:** Whether the picture writer rebuilds `drawing` from its `data:` bytes, its alt text and the raster's size alone: a `wp:inline` of exactly the shape `drawing_xml` writes, at the size `read_picture` computes.
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:367) L924; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L927,934,945,952,959,969...; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L930,941,965,977,978,993...; `attrs_ok` (markdown/from_docx/drawing.rs:1087) L935,948,953,960,975,986...; `local` (markdown/from_docx/ooxml.rs:272) L936,948,954,986,1026,1051...; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L946,947,985,1049,1050,1058...; `generated_name` (markdown/from_docx/drawing.rs:907) L962,1005; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L972,982,990,1000,1063; `Element::rel_attr` (markdown/from_docx/ooxml.rs:294) (by name) L1025; `base64_decode` (markdown/from_docx/drawing.rs:1196) L1074; `read_picture` (markdown/write.rs:5616) L1077
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_slice`, `.as_str`, `.collect`, `.contains`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.len`, `.next`, `.nth`, `.to_string`, `Some`
- **Macros:** `matches!`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703)

#### `attrs_ok`  (L1087-1092)
`fn attrs_ok(element: &Element, ok: impl Fn(&str, &str) -> bool) -> bool`

- **Does:** `element`'s non-`xmlns` attributes all pass `ok`.
- **Calls (std/external/unresolved):** `.all`, `.iter`, `.starts_with`, `ok`
- **Called by:** `regenerable` (markdown/from_docx/drawing.rs:917)

#### `standalone`  (L1097-1111)
`fn standalone(out: &str, pos: usize, end: usize) -> Option<(usize, usize)>`

- **Does:** The span of a `![label][dN]` token that is a whole line: its `![` opening and its line's end.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) L1099
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.find`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map_or`, `.rfind`, `.skip`, `.starts_with`, `.then_some`, `.trim`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703)

#### `fenced`  (L1115-1128)
`fn fenced(info: &str, text: &str) -> String`

- **Does:** `text` as a fenced code block carrying `info`, the fence one backtick wider than any run inside.
- **Calls (std/external/unresolved):** `.bytes`, `.max`, `.repeat`
- **Macros:** `format!`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703)

#### `at`  (L1134-1159, pub(crate))
`pub(crate) fn at(text: &str) -> Option<(&str, usize)>`

- **Does:** The Markdown drawing token at `text`'s start, if any — `![label][dN]`, or a `![label](drawing:PAYLOAD)` of a view an older build wrote.
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.find`, `.is_ascii_alphanumeric`, `.is_ascii_digit`, `.is_empty`, `.len`, `.map`, `.or_else`, `.split_once`, `.strip_prefix`, `.then`
- **Macros:** `matches!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477) (?); `unit_lcs` (comparer/cross_para.rs:1384) (?); `char_weighted_lcs` (comparer/cross_para.rs:1996) (?); `wmf_object_tests::a_polypolygon_fills_with_the_brush_selected_by_zero_based_index` (convert/metafile.rs:1576) (?); `wmf_object_tests::a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen` (convert/metafile.rs:1641) (?); `wmf_object_tests::a_null_brush_rectangle_only_outlines` (convert/metafile.rs:1667) (?); `chrome_part_xml` (convert/mod.rs:20750) (?); `coverage_round_three_tests::coverage_deep_layout_tests::chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags` (convert/mod.rs:53458) (?); `inner_comment_references` (edit.rs:5674) (?); `escape_markdown` (markdown/from_docx/agent.rs:1290) (?); `relabel` (markdown/from_docx/drawing.rs:847); `standalone` (markdown/from_docx/drawing.rs:1097); +8 more

#### `cuts_token`  (L1162-1167, pub(crate))
`pub(crate) fn cuts_token(text: &str, cut: usize) -> bool`

- **Does:** Whether cutting `text` at `cut` splits a `![…](drawing:…)` token.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) L1166
- **Calls (std/external/unresolved):** `.is_some_and`, `.rfind`
- **Called by:** `tracked` (markdown/view_stories.rs:814)

#### `decode`  (L1171-1183, pub(crate))
`pub(crate) fn decode(dest: &str) -> Result<Drawing, String>`

- **Does:** Decode the payload of `![label](drawing:PAYLOAD)`; `dest` is what [`at`] returns (`drawing:…`).
- **Calls (std/external/unresolved):** `.decode`, `.map_err`, `.ok_or_else`, `.or_else`, `.strip_prefix`, `.to_string`, `.trim_end_matches`, `serde_json::from_slice`
- **Macros:** `format!`
- **Called by:** `lookup` (markdown/from_docx/drawing.rs:1234); `Writer::end` (markdown/write.rs:2137); `Writer::picture` (markdown/write.rs:3933)

#### `encode`  (L1185-1189, pub(crate))
`pub(crate) fn encode(drawing: &Drawing) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.encode`, `.unwrap_or_default`, `serde_json::to_vec`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703); `canonical` (markdown/from_docx/drawing.rs:867)

#### `base64_encode`  (L1191-1194)
`fn base64_encode(bytes: &[u8]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.encode`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:438)

#### `base64_decode`  (L1196-1201, pub(crate))
`pub(crate) fn base64_decode(encoded: &str) -> Option<Vec<u8>>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.decode`, `.ok`
- **Called by:** `chart_mermaid` (markdown/from_docx/drawing.rs:595); `fill` (markdown/from_docx/drawing.rs:703); `regenerable` (markdown/from_docx/drawing.rs:917); `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1348); `dep_part` (markdown/from_docx/drawing.rs:1446); `Writer::picture` (markdown/write.rs:3933)

#### `data_uri`  (L1205-1210, pub(crate))
`pub(crate) fn data_uri(dest: &str) -> Option<(&str, &str)>`

- **Does:** A `data:` destination's `(mime, base64)` parts: `data:mime;base64,…`.
- **Calls (std/external/unresolved):** `.is_empty`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `.then_some`
- **Called by:** `lookup` (markdown/from_docx/drawing.rs:1234); `Writer::picture` (markdown/write.rs:3933)

#### `backing_comments`  (L1214-1228, pub(crate))
`pub(crate) fn backing_comments(source: &str) -> HashMap<String, String>`

- **Does:** The `<!-- NAME drawing:PAYLOAD -->` backing comments of a view's drawings appendix, `NAME` → the `drawing:` destination.
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.is_empty`, `.lines`, `.split_once`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.then`, `.to_string`, `.trim`
- **Called by:** `read` (markdown/write.rs:125)

#### `lookup`  (L1234-1253, pub(crate))
`pub(crate) fn lookup( key_or_dest: &str, defs: &HashMap<String, String>, backings: &HashMap<String, String>, ) -> Option<Drawing>`

- **Does:** The `Drawing` a reference name or destination in the view resolves to: a `drawing:` destination or `dN` definition as it is; an `imageK` name through its backing comment, the `data:` definition's bytes filling the slot the readable layer carries.
- **Calls (crate):** `decode` (markdown/from_docx/drawing.rs:1171) L1245,1248; `data_uri` (markdown/from_docx/drawing.rs:1205) L1247; `Drawing::fill_slot_data` (markdown/from_docx/drawing.rs:1327) (by name) L1249
- **Calls (std/external/unresolved):** `.get`, `.map`, `.ok`, `.or_else`, `.starts_with`, `.then_some`
- **Called by:** `story_part` (markdown/view_stories.rs:1118); `Writer::picture` (markdown/write.rs:3933)

#### `holds_drawing`  (L1257-1260, pub(crate))
`pub(crate) fn holds_drawing(element: &Element) -> bool`

- **Does:** Whether `element` or a descendant holds a drawing: `w:drawing`, `w:pict` or `w:object` anywhere below `AlternateContent`'s `Choice`/`Fallback`.
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1258; `Element::elements` (markdown/from_docx/ooxml.rs:301) L1259
- **Calls (std/external/unresolved):** `.any`
- **Macros:** `matches!`
- **Called by:** `capturable_alternate` (markdown/from_docx/drawing.rs:1290); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736)

#### `is_plain_inline`  (L1265-1271, pub(crate))
`pub(crate) fn is_plain_inline(element: &Element) -> bool`

- **Does:** Whether `element` is a plain inline picture the `![alt](name)` syntax keeps when its raster is extracted: `wp:inline` holding an `a:blip`, no floating anchor, no text box the payload would carry with it.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) L1266; `Element::find` (markdown/from_docx/ooxml.rs:325) L1267,1268,1269,1270
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3366)

#### `holds_block_content`  (L1278-1286)
`fn holds_block_content(element: &Element) -> bool`

- **Does:** `element` holds paragraph-level content (`w:p`, `w:tbl`, `w:sdt`) outside a drawing, picture or text box — where it is encapsulated and stays inside the run whatever it holds.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1279; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1280,1283; `holds_block_content` (markdown/from_docx/drawing.rs:1278) L1283
- **Calls (std/external/unresolved):** `.any`
- **Macros:** `matches!`
- **Called by:** `holds_block_content` (markdown/from_docx/drawing.rs:1278); `capturable_alternate` (markdown/from_docx/drawing.rs:1290)

#### `capturable_alternate`  (L1290-1297, pub(crate))
`pub(crate) fn capturable_alternate(element: &Element) -> bool`

- **Does:** Whether a paragraph-level `mc:AlternateContent` may be captured whole: one with a drawing and only run-level content below its `Choice`.
- **Calls (crate):** `holds_drawing` (markdown/from_docx/drawing.rs:1257) L1291; `Element::elements` (markdown/from_docx/ooxml.rs:301) L1294; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1295; `holds_block_content` (markdown/from_docx/drawing.rs:1278) L1295
- **Calls (std/external/unresolved):** `.all`
- **Macros:** `matches!`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Drawing::wraps_run`  (L1308-1310, pub(crate))
`pub(crate) fn wraps_run(&self) -> bool`

- **Does:** The element goes inside a `w:r` (`w:drawing`, `w:pict`, run-level AlternateContent), not where a `w:r` may sit (paragraph-level AlternateContent).
- **Calls:** none
- **Called by:** `story_part` (markdown/view_stories.rs:1118) (?); `write_run` (markdown/xml.rs:1854) (?)

#### `Drawing::without_slot_bytes`  (L1314-1323)
`fn without_slot_bytes(&self) -> Drawing`

- **Does:** `self` with the readable layer's bytes out of the payload: the backing comment of a `data:` definition or an `svg` fence.
- **Calls (std/external/unresolved):** `.clear`, `.clone`, `.get_mut`
- **Called by:** `fill` (markdown/from_docx/drawing.rs:703) (?)

#### `Drawing::fill_slot_data`  (L1327-1337, pub(crate))
`pub(crate) fn fill_slot_data(&mut self, b64: &str, kind: &str) -> bool`

- **Does:** Fill the slot with `b64` bytes and, when `kind` differs, its content type; `false` when the payload carries no slot.
- **Calls (std/external/unresolved):** `.and_then`, `.clear`, `.get_mut`, `.to_string`
- **Called by:** `lookup` (markdown/from_docx/drawing.rs:1234) (?); `Writer::end` (markdown/write.rs:2137) (?)

#### `Drawing::mermaid_unchanged`  (L1341-1343, pub(crate))
`pub(crate) fn mermaid_unchanged(&self, text: &str) -> bool`

- **Does:** `text` is the ` ```mermaid ` fence this drawing printed: its hash is `mhash`'s, so the drawing writes back byte for byte.
- **Calls (crate):** `mermaid_hash` (markdown/from_docx/drawing.rs:609) L1342
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `Some`
- **Called by:** `Writer::end` (markdown/write.rs:2137) (?)

#### `Drawing::rebuild_mermaid`  (L1348-1368, pub(crate))
`pub(crate) fn rebuild_mermaid(&mut self, text: &str) -> bool`

- **Does:** The fence's text edited: patch the chart part it names in place, keeping everything the Mermaid cannot say.
- **Calls (crate):** `base64_decode` (markdown/from_docx/drawing.rs:1196) L1357; `patch` (markdown/from_docx/mermaid.rs:374) L1361; `mermaid_hash` (markdown/from_docx/drawing.rs:609) L1366
- **Calls (std/external/unresolved):** `.and_then`, `.clear`, `.clone`, `.ends_with`, `.find`, `.is_empty`, `.iter_mut`, `.ok`, `Some`, `String::from_utf8`
- **Called by:** `Writer::end` (markdown/write.rs:2137) (?)

#### `materialize`  (L1371-1395, pub(crate))
`pub(crate) fn materialize( drawing: &Drawing, package: &mut PartFs, part: &str, existing: &HashSet<String>, used: &mut HashSet<String>, next_doc_pr: &mut u32, ) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `dep_part` (markdown/from_docx/drawing.rs:1446) L1384; `relative_rel_target` (opc/mod.rs:129) L1385,1387; `relate` (markdown/from_docx/drawing.rs:1400) L1390; `renumber_doc_pr` (markdown/from_docx/drawing.rs:1418) L1393
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.replace`, `Some`
- **Macros:** `format!`
- **Called by:** `<PartRelate as Relate>::drawing` (markdown/package.rs:1541); `story_part` (markdown/view_stories.rs:1118)

#### `relate`  (L1400-1414)
`fn relate(package: &mut PartFs, part: &str, kind: &str, target: &str, external: bool) -> String`

- **Does:** A relationship `part` → `target` of `kind`, reusing one already written when there is one: two drawings that share a media part share its relationship too.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L1401; `PartFs::add_document_relationship_external` (opc/mod.rs:806) L1410; `PartFs::add_document_relationship` (opc/mod.rs:772) L1412
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.iter`
- **Called by:** `materialize` (markdown/from_docx/drawing.rs:1371); `dep_part` (markdown/from_docx/drawing.rs:1446)

#### `renumber_doc_pr`  (L1418-1442)
`fn renumber_doc_pr(xml: &mut String, next: &mut u32)`

- **Does:** Give every `docPr id` in `xml` a fresh number: the stored `0`s collide with each other and with pictures the writer numbers.
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.is_ascii_digit`, `.len`, `.push_str`, `.unwrap_or`, `String::with_capacity`
- **Macros:** `write!`
- **Called by:** `materialize` (markdown/from_docx/drawing.rs:1371); `dep_part` (markdown/from_docx/drawing.rs:1446); `tests::doc_pr_ids_normalize_and_renumber` (markdown/from_docx/drawing.rs:1587)

#### `dep_part`  (L1446-1498)
`fn dep_part( drawing: &Drawing, index: usize, package: &mut PartFs, existing: &HashSet<String>, used: &mut HashSet<String>, next_doc_pr: &mut u32, ) -> Option<String>`

- **Does:** Write dependent part `index` and its own dependency closure; its new part name.
- **Calls (crate):** `name_for` (markdown/from_docx/drawing.rs:1502) L1455; `dep_part` (markdown/from_docx/drawing.rs:1446) L1464; `relative_rel_target` (opc/mod.rs:129) L1467,1471; `relate` (markdown/from_docx/drawing.rs:1400) L1474; `base64_decode` (markdown/from_docx/drawing.rs:1196) L1480; `renumber_doc_pr` (markdown/from_docx/drawing.rs:1418) L1482; `PartFs::part_bytes` (opc/mod.rs:493) L1488; `PartFs::set_part` (opc/mod.rs:510) L1495; `PartFs::add_content_type_override` (opc/mod.rs:616) L1496
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.enumerate`, `.get`, `.insert`, `.into_bytes`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.replace`, `Some`
- **Macros:** `format!`
- **Called by:** `materialize` (markdown/from_docx/drawing.rs:1371); `dep_part` (markdown/from_docx/drawing.rs:1446)

#### `name_for`  (L1502-1518)
`fn name_for(want: &str, existing: &HashSet<String>, used: &HashSet<String>) -> String`

- **Does:** `want` when nothing holds it, else `want` with `-j2`, `-j3`, … before its extension.
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.rsplit_once`, `.to_string`, `String::new`, `free`
- **Macros:** `format!`
- **Called by:** `dep_part` (markdown/from_docx/drawing.rs:1446)

## Test functions (6)

- `tests::element` (L1525): _no doc comment_
- `tests::xml_round_trips_verbatim` (L1530): _no doc comment_
- `tests::declares_prefixes_the_tree_uses` (L1537): _no doc comment_
- `tests::rel_names_match_only_the_relationship_namespace` (L1554): _no doc comment_
- `tests::token_parses_destination_and_length` (L1568): _no doc comment_
- `tests::doc_pr_ids_normalize_and_renumber` (L1587): _no doc comment_
