# Functions and methods in `src/markdown/package.rs`

49 non-test, 12 test. Part of the [symbol manifest](../README.md).

#### `default_section`  (L40-45)
`fn default_section(page: PageSize) -> &'static str`

- **Does:** The section a document gets when nothing else gives one.
- **Calls:** none
- **Called by:** `frame` (markdown/package.rs:524); `default_template` (markdown/package.rs:1777); `tests::a_main_part_without_a_usable_root_or_section_falls_back_to_the_page` (markdown/package.rs:1930)

#### `threads`  (L74-96)
`fn threads( package: &mut PartFs, main: &str, comments: &[xml::Comment], ) -> Result<(), MarkdownError>`

- **Does:** Replies (`c6 @AS re c5`) and resolved comments (`c5 @AC resolved`): Word keys both by the `w14:paraId` of a comment's last paragraph, in `commentsExtended.xml`.
- **Calls (crate):** `CommentFamily::load` (comments.rs:457) L82; `CommentFamily::set_thread` (comments.rs:819) (by name) L84; `offset_date_utc` (markdown/xml.rs:2930) L86; `CommentFamily::set_date_utc` (comments.rs:726) (by name) L88; `CommentFamily::set_done` (comments.rs:830) (by name) L92; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L94
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.is_none`, `.is_some`, `.iter`, `.map_err`, `Ok`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `err`  (L98-100)
`fn err(message: impl std::fmt::Display) -> MarkdownError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `MarkdownError::Package`
- **Called by:** `AnchorChanges::records` (edit/comment_settlement.rs:78) (?); `cursor_offset` (edit/for_edit.rs:503) (?); `Selection::build` (edit/for_edit.rs:547) (?); `text_selection` (edit/for_edit.rs:663) (?); `changed_selection` (edit/for_edit.rs:703) (?); `for_edit` (edit/for_edit.rs:1581) (?); `Transaction::footnotes_story` (edit/notes.rs:51) (?); `err` (edit/prepared.rs:1047) (?); `normalize_permission_ids` (edit/ranges.rs:374) (?); `restore` (edit/ranges.rs:432) (?); `typing_order` (edit/replacement_order.rs:82) (?); `refusal` (edit/source_refs.rs:8) (?); +31 more

#### `assemble`  (L103-372, pub(super))
`pub(super) fn assemble( document: &Document, options: &DocxOptions<'_>, markdown: &str, warnings: &mut Vec<String>, ) -> Result<Vec<u8>, MarkdownError>`

- **Does:** The finished package.
- **Calls (crate):** `default_template` (markdown/package.rs:1777) L111; `PartFs::open` (opc/mod.rs:585) L113; `refusal_message` (opc/mod.rs:88) L116; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L119; `PartFs::part_string` (opc/mod.rs:636) (by name) L122,171,177,327; `frame` (markdown/package.rs:524) L131; `prune` (markdown/package.rs:595) L132; `styles_part` (markdown/package.rs:467) L134; `with_styles` (markdown/package.rs:1065) L172; `with_named_styles` (markdown/package.rs:1082) L173; `apply` (markdown/view_styles.rs:586) L174; `with_style_changes` (markdown/view_styles.rs:45) L175; `related` (markdown/package.rs:512) L176; `with_table_styles` (markdown/package.rs:706) L179; `PartFs::set_part` (opc/mod.rs:661) (by name) L180,292,308,367; `numbering` (markdown/package.rs:1149) L183; `label_numbering` (markdown/package.rs:1165) L184; `Document::section_breaks` (markdown/xml.rs:574) L186; `write` (markdown/view_stories.rs:1599) L205; `.apply` (ambiguous, 6 candidates) L221; `Document::section` (markdown/xml.rs:561) L221; `text_width` (markdown/package.rs:569) L222; `max_drawing_id` (markdown/package.rs:1438) L223; `escape` (markdown/xml.rs:601) L226,227; `format_changes` (markdown/view_stories.rs:525) L263; `footnotes` (markdown/package.rs:1348) L273; `story` (markdown/xml.rs:786) L288; `sibling` (markdown/package.rs:504) L291; `comments_part` (markdown/xml.rs:2464) L294; `PartFs::add_content_type_override` (opc/mod.rs:845) (by name) L296,317; `PartFs::add_document_relationship` (opc/mod.rs:1003) (by name) L300; `relative_rel_target` (opc/mod.rs:180) L303; `threads` (markdown/package.rs:74) L305; `core_properties` (markdown/package.rs:1454) L307; `PartFs::content_type_for` (opc/mod.rs:837) (by name) L316; `write_missing` (markdown/data_stores.rs:63) L320; `PartFs::parts` (opc/mod.rs:733) (by name) L323; `root_start_tag` (markdown/package.rs:448) L335; `Dom::new` (xmllinq/mod.rs:387) L357; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L358; `Dom::root` (xmllinq/mod.rs:822) L359; `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942) L360; `serialize_element` (xmllinq/serialize.rs:401) L363; `unique_revision_ids` (markdown/package.rs:378) L370; `PartFs::to_zip` (opc/mod.rs:749) (by name) L371
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.copied`, `.ends_with`, `.expect`, `.extend`, `.filter`, `.find`, `.flat_map`, `.flatten`, `.insert_str`, `.insert`, `.into_bytes`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_none`, `.iter`, `.map_err`, `.map_or`, `.map`, `.max`, `.min`, `.ok_or_else`, `.read_resolved_event`, `.replace_range`, `.saturating_add`, `.starts_with`, `.to_string`, `.to_vec`, `.unwrap_or_else`, `.values`, `HashMap::new`, `HashSet::new`, `MarkdownError::Reference`, `Media::default`, `Ok`, `Vec::new`, `quick_xml::NsReader::from_str`, `quick_xml::name::ResolveResult::Bound`, `super::view_stories::People::default`
- **Macros:** `format!`, `matches!`
- **Called by:** `markdown_to_docx` (markdown/write.rs:59)

#### `unique_revision_ids`  (L378-444)
`fn unique_revision_ids(package: &mut PartFs, markdown: &str, warnings: &mut Vec<String>)`

- **Does:** The body allocator handles its own ids.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L384; `PartFs::parts` (opc/mod.rs:733) L394; `PartFs::part_string` (opc/mod.rs:636) L399; `Dom::new` (xmllinq/mod.rs:387) L402; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L403; `Dom::root` (xmllinq/mod.rs:822) L404; `renumber_duplicate_revision_ids` (validate.rs:2831) L407; `PartFs::main_document_part` (opc/mod.rs:1070) L411; `ChangeId::new` (ids.rs:142) L420,421; `ChangeId::short` (ids.rs:155) (by name) L421,436; `PartFs::set_part` (opc/mod.rs:661) L439; `serialize_document` (xmllinq/serialize.rs:805) L441
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.ends_with`, `.entry`, `.enumerate`, `.filter`, `.get`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_whitespace`, `.iter`, `.join`, `.last`, `.lines`, `.map`, `.next`, `.or_default`, `.push`, `.rsplit`, `.split`, `.trim_end_matches`, `.unwrap_or`, `HashMap::new`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `root_start_tag`  (L448-464)
`fn root_start_tag(source: &str) -> Option<(usize, usize)>`

- **Does:** The byte range of the root element's start tag, `<` up to its `>`: the first tag after the declaration, comments and doctype.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.starts_with`, `Some`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `styles_part`  (L467-489, pub(crate))
`pub(crate) fn styles_part(package: &mut PartFs, main: &str) -> String`

- **Does:** The styles part `main` relates to, made empty when there is none.
- **Calls (crate):** `related` (markdown/package.rs:512) L468; `sibling` (markdown/package.rs:504) L469; `PartFs::add_document_relationship` (opc/mod.rs:1003) L470; `relative_rel_target` (opc/mod.rs:180) L473; `PartFs::add_content_type_override` (opc/mod.rs:845) L475; `PartFs::set_part` (opc/mod.rs:661) L479
- **Calls (std/external/unresolved):** `.into_bytes`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `add_styles` (edit/structural.rs:267); `assemble` (markdown/package.rs:103)

#### `has_stories`  (L494-502, pub(super))
`pub(super) fn has_stories(reference: &[u8]) -> bool`

- **Does:** The part next to `main` named `name` (`word/` + `name`).
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L495; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L498; `PartFs::part_string` (opc/mod.rs:636) (by name) L500
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_some_and`
- **Called by:** none found in src

#### `sibling`  (L504-509, pub(super))
`pub(super) fn sibling(main: &str, name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rsplit_once`, `.to_string`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103); `styles_part` (markdown/package.rs:467); `numbering_part` (markdown/package.rs:1123); `ensure_notes_part` (markdown/package.rs:1233); `<PartRelate as Relate>::picture` (markdown/package.rs:1703); `story_part` (markdown/view_stories.rs:1710)

#### `related`  (L512-520, pub(super))
`pub(super) fn related(package: &PartFs, main: &str, kind: &str) -> Option<String>`

- **Does:** The part `main` relates to with the relationship type ending in `kind`.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L513; `PartFs::resolve_rel_target` (opc/mod.rs:829) L518; `PartFs::part_bytes` (opc/mod.rs:628) L519
- **Calls (std/external/unresolved):** `.as_deref`, `.ends_with`, `.find`, `.is_some`, `.iter`, `.then_some`, `Some`
- **Called by:** `stores` (markdown/data_stores.rs:155) (?); `assemble` (markdown/package.rs:103); `styles_part` (markdown/package.rs:467); `prune` (markdown/package.rs:595); `numbering_part` (markdown/package.rs:1123); `ensure_notes_part` (markdown/package.rs:1233); `even_and_odd_headers` (markdown/view_stories.rs:2006)

#### `frame`  (L524-548)
`fn frame(source: &str, page: PageSize) -> (String, String)`

- **Does:** The document element's start tag, declaring the prefixes the body uses, and the body's final section properties.
- **Calls (crate):** `default_section` (markdown/package.rs:40) L525; `final_section` (markdown/package.rs:553) L546
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.find`, `.push_str`, `.push`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103); `tests::a_main_part_without_a_usable_root_or_section_falls_back_to_the_page` (markdown/package.rs:1930)

#### `final_section`  (L553-566)
`fn final_section(source: &str) -> Option<String>`

- **Does:** The `w:sectPr` that is the body's last child, without a tracked change of its own (`w:sectPrChange`): the reference's revisions are not the written document's.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L554; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L555; `Dom::root` (xmllinq/mod.rs:822) L556; `Dom::element` (xmllinq/mod.rs:655) L557; `Dom::elements` (xmllinq/mod.rs:640) L558,562; `Dom::name_is` (xmllinq/mod.rs:535) L559; `Dom::remove` (xmllinq/mod.rs:1118) L563; `serialize_element` (xmllinq/serialize.rs:401) L565
- **Calls (std/external/unresolved):** `.last`, `Some`, `W::body`, `W::name`, `W::sect_pr`
- **Called by:** `frame` (markdown/package.rs:524); `tests::the_final_section_is_found_only_as_the_bodys_last_child` (markdown/package.rs:1899)

#### `text_width`  (L569-591)
`fn text_width(section: &str) -> u32`

- **Does:** The page width less the side margins, in twentieths of a point.
- **Calls (crate):** `universal_measure_twips` (util/mod.rs:47) L582; `attribute` (markdown/unified.rs:756) (by name) L585,586,587
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.is_finite`, `.len`, `.max`, `.ok`, `.or_else`, `.parse`, `.round`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103); `tests::the_text_width_reads_universal_measures_as_the_view_does` (markdown/package.rs:1867); `tests::the_text_width_is_the_page_less_its_margins` (markdown/package.rs:1949)

#### `prune`  (L595-630)
`fn prune(package: &mut PartFs, main: &str)`

- **Does:** Drops the reference body's relationships (pictures, links, comments...) and their parts, and the notes the body referenced.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L596,607,612; `PartFs::remove_relationships_by_type` (opc/mod.rs:904) L618; `PartFs::remove_related_part` (opc/mod.rs:923) L620; `related` (markdown/package.rs:512) L624; `PartFs::part_string` (opc/mod.rs:636) L625; `PartFs::set_part` (opc/mod.rs:661) L627; `separators_only` (markdown/package.rs:633) L627
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.find`, `.into_bytes`, `.is_some_and`, `.iter`, `.map`, `.unwrap_or_default`, `Some`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `separators_only`  (L633-671)
`fn separators_only(xml: &str) -> String`

- **Does:** A notes part with only its separator notes.
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.filter_map`, `.find`, `.iter`, `.len`, `.map_or`, `.map`, `.min_by_key`, `.push_str`, `.starts_with`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `prune` (markdown/package.rs:595); `tests::separators_survive_and_notes_do_not` (markdown/package.rs:1961); `literal_package_metadata_contract_tests::separator_notes_keep_complete_authored_markup_without_ordinary_notes` (markdown/package.rs:2089)

#### `renamed`  (L676-691)
`fn renamed(xml: &str, name: &str) -> String`

- **Does:** The table style `xml` with its `w:name` replaced by `name`: a built-in template keeps Word's definition, the name is the document's own.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L688; `word_name` (markdown/package.rs:697) L688
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map`, `.to_string`
- **Macros:** `format!`
- **Called by:** `with_table_styles` (markdown/package.rs:706)

#### `word_name`  (L697-699)
`fn word_name(name: &str) -> String`

- **Does:** The `w:name` value for `name`: no comma, which `w:name` forbids (`[^,]*`; Word reads one as the start of an alias list), so every docx the writer makes is Word-valid.
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `document_text_view` (debug.rs:2181) (?); `renamed` (markdown/package.rs:676); `with_table_styles` (markdown/package.rs:706)

#### `with_table_styles`  (L706-748)
`fn with_table_styles( styles: &str, wanted: &BTreeMap<String, (Option<TableBorders>, Option<String>)>, theme: &str, ) -> String`

- **Does:** `styles` with a definition for each table style the view names that it lacks: one of Word's built-in Grid and List Tables as Word defines it, its colours from `theme`; else the name is the view's (`named …`), Word's own for an id Word names, else the id, on Word's Normal Table, with the legend's border all round when...
- **Calls (crate):** `escape` (markdown/xml.rs:601) L713,737,738; `built_in_table_style` (markdown/package.rs:794) L715; `renamed` (markdown/package.rs:676) L720; `table_border_sides` (markdown/package.rs:753) L727; `word_table_style_name` (markdown/package.rs:877) L732; `word_name` (markdown/package.rs:697) L738
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map_or_else`, `.map`, `.or_else`, `.rfind`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `table_border_sides`  (L753-783)
`fn table_border_sides(border: &TableBorders) -> String`

- **Does:** A table style's `w:tblBorders` children for the legend's borders: one single line of the size all round, or each side the legend names, in the order Word's schema puts them.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L770,775
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter_map`, `.find`, `.is_none`, `.iter`, `.map`, `.unwrap_or_default`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `with_table_styles` (markdown/package.rs:706)

#### `built_in_table_style`  (L794-802)
`fn built_in_table_style(id: &str, theme: &str) -> Option<String>`

- **Does:** The definition of `id` when it is one of Word's built-in Grid or List Tables (`GridTable4-Accent1`, `ListTable6Colorful`), named as Word names it (`Grid Table 4 Accent 1`), its colours from `theme`.
- **Calls (crate):** `built_in` (markdown/package.rs:806) L795; `escape` (markdown/xml.rs:601) L798,799; `with_theme_colours` (markdown/package.rs:912) L800
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `format!`
- **Called by:** `with_table_styles` (markdown/package.rs:706); `is_built_in_table_style` (markdown/package.rs:848); `word_table_style_name` (markdown/package.rs:877)

#### `built_in`  (L806-844)
`fn built_in(id: &str) -> Option<(String, String)>`

- **Does:** `id` as Word names it and its unstyled body, when it is one of the built-in Grid or List Tables.
- **Calls (std/external/unresolved):** `.chars`, `.find_map`, `.flatten`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.is_empty`, `.lines`, `.next`, `.push_str`, `.push`, `.replace`, `.split_once`, `.splitn`, `.then`, `.to_string`, `Some`, `String::new`, `template`
- **Macros:** `format!`, `matches!`
- **Called by:** `built_in_table_style` (markdown/package.rs:794); `check_data_bindings_have_stores` (validate.rs:1189) (?)

#### `is_built_in_table_style`  (L848-850, pub(super))
`pub(super) fn is_built_in_table_style(id: &str) -> bool`

- **Does:** Whether `id` is one of Word's built-in Grid or List Tables, which the writer defines as Word does.
- **Calls (crate):** `built_in_table_style` (markdown/package.rs:794) L849
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `table_style_line` (markdown/from_docx/header.rs:1091); `Writer::metadata` (markdown/write.rs:4453)

#### `word_table_style_name`  (L877-906, pub(super))
`pub(super) fn word_table_style_name(id: &str) -> Option<String>`

- **Does:** Word's name for a built-in table style `id`: the Grid and List Tables from their definitions, the older families and the two defaults by the id itself (`TableGrid` → `Table Grid`, `LightShading-Accent1` → `Light Shading Accent 1`).
- **Calls (crate):** `built_in_table_style` (markdown/package.rs:794) L878
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.find`, `.is_alphabetic`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.is_uppercase`, `.last`, `.len`, `.push`, `.split_once`, `.then`, `.to_string`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `table_style_line` (markdown/from_docx/header.rs:1091); `with_table_styles` (markdown/package.rs:706)

#### `with_theme_colours`  (L912-942)
`fn with_theme_colours(xml: &str, theme: &str) -> String`

- **Does:** `xml` with each `@` colour resolved from `theme` as Word resolves a theme colour: the element's `w:themeColor` (or `w:themeFill`), lightened by its tint and darkened by its shade; `auto` for a colour the theme does not hold.
- **Calls (crate):** `attribute` (markdown/unified.rs:756) (by name) L931,933; `theme_colour` (markdown/package.rs:964) L933
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.map`, `.replace`, `.split_inclusive`, `.split_once`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `built_in_table_style` (markdown/package.rs:794)

#### `drawing_prefix`  (L948-962)
`fn drawing_prefix(theme: &str) -> String`

- **Does:** The theme colour `name` (`accent1`, `text1`), tinted and shaded as Word does: by lightness, in HSL, each channel truncated.
- **Calls (std/external/unresolved):** `.find_map`, `.map`, `.match_indices`, `.rfind`, `.strip_prefix`, `.to_string`, `.unwrap_or_else`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `theme_colour` (markdown/package.rs:964); `tests::a_theme_colour_is_found_under_any_drawingml_prefix` (markdown/package.rs:1881)

#### `theme_colour`  (L964-1015)
`fn theme_colour( theme: &str, name: &str, tint: Option<&str>, shade: Option<&str>, ) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `drawing_prefix` (markdown/package.rs:948) L979; `rgb_to_hls` (markdown/package.rs:1017) L994; `fraction` (convert/math_layout.rs:197) (by name) L1000,1003; `hls_to_rgb` (markdown/package.rs:1040) L1006
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.clamp`, `.filter`, `.find_map`, `.find`, `.floor`, `.get`, `.is_ascii_hexdigit`, `.is_none`, `.iter`, `.len`, `.map`, `.ok`, `.split_once`, `.to_ascii_uppercase`, `.unwrap_or`, `Some`, `byte`, `channel`, `f64::from`, `u8::from_str_radix`
- **Macros:** `format!`
- **Called by:** `with_theme_colours` (markdown/package.rs:912); `tests::a_theme_colour_is_found_under_any_drawingml_prefix` (markdown/package.rs:1881)

#### `rgb_to_hls`  (L1017-1038)
`fn rgb_to_hls(r: f64, g: f64, b: f64) -> (f64, f64, f64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.rem_euclid`
- **Called by:** `theme_colour` (markdown/package.rs:964)

#### `hls_to_rgb`  (L1040-1063)
`fn hls_to_rgb(h: f64, l: f64, s: f64) -> (f64, f64, f64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rem_euclid`, `value`
- **Called by:** `theme_colour` (markdown/package.rs:964)

#### `with_styles`  (L1065-1077)
`fn with_styles(styles: &str, wanted: &BTreeSet<&str>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `style_definition` (markdown/xml.rs:2711) L1069
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.is_empty`, `.iter`, `.rfind`, `.to_string`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103); `tests::missing_styles_are_added_and_present_ones_kept` (markdown/package.rs:1973)

#### `with_named_styles`  (L1082-1094)
`fn with_named_styles(styles: &str, named: &BTreeSet<String>) -> String`

- **Does:** `styles` with a definition for each style an id line names that it lacks: the reader's own definition when it has one (`Quote`), else a paragraph style on Normal.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L1085; `style_definition` (markdown/xml.rs:2711) L1086; `named_style_definition` (markdown/xml.rs:2699) L1086
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map`, `.rfind`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `max_attribute`  (L1097-1099)
`fn max_attribute(xml: &str, element: &str, name: &str) -> Option<i64>`

- **Does:** The largest value of attribute `name` on elements `element` in `xml`.
- **Calls (crate):** `attribute_values` (markdown/package.rs:1102) L1098
- **Calls (std/external/unresolved):** `.into_iter`, `.max`
- **Called by:** `append_numbering` (markdown/package.rs:1185); `footnotes` (markdown/package.rs:1348); `max_drawing_id` (markdown/package.rs:1438); `tests::max_attribute_reads_only_the_named_element` (markdown/package.rs:1982); `literal_package_metadata_contract_tests::separator_notes_keep_complete_authored_markup_without_ordinary_notes` (markdown/package.rs:2089)

#### `attribute_values`  (L1102-1120)
`fn attribute_values(xml: &str, element: &str, name: &str) -> Vec<i64>`

- **Does:** Every numeric `name` attribute of `element` in `xml`.
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.len`, `.map_or`, `.map`, `.ok`, `.parse`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `max_attribute` (markdown/package.rs:1097); `footnotes` (markdown/package.rs:1348)

#### `numbering_part`  (L1123-1145, pub(crate))
`pub(crate) fn numbering_part(package: &mut PartFs, main: &str) -> String`

- **Does:** The numbering part `main` relates to, made empty when there is none.
- **Calls (crate):** `related` (markdown/package.rs:512) L1124; `sibling` (markdown/package.rs:504) L1125; `PartFs::add_document_relationship` (opc/mod.rs:1003) L1126; `relative_rel_target` (opc/mod.rs:180) L1129; `PartFs::add_content_type_override` (opc/mod.rs:845) L1131; `PartFs::set_part` (opc/mod.rs:661) L1135
- **Calls (std/external/unresolved):** `.into_bytes`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `splice_numbering` (edit/structural.rs:377); `append_numbering` (markdown/package.rs:1185)

#### `numbering`  (L1149-1161)
`fn numbering(package: &mut PartFs, main: &str, document: &Document) -> Vec<u32>`

- **Does:** Adds the document's lists to the numbering part, making one if needed; returns each list's `w:numId`.
- **Calls (crate):** `append_numbering` (markdown/package.rs:1185) L1153; `numbering` (markdown/xml.rs:2612) L1154
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.len`, `.map`, `.unwrap_or`, `Vec::new`, `u32::try_from`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `label_numbering`  (L1165-1180)
`fn label_numbering(package: &mut PartFs, main: &str, document: &Document) -> Vec<u32>`

- **Does:** Adds a list for each label an id line names; returns each label's `w:numId`.
- **Calls (crate):** `append_numbering` (markdown/package.rs:1185) L1169; `label_numbering` (markdown/xml.rs:2643) L1170
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.len`, `.take`, `Vec::new`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `append_numbering`  (L1185-1220, pub(crate))
`pub(crate) fn append_numbering( package: &mut PartFs, main: &str, build: impl FnOnce(u32, u32) -> (String, String), ) -> Option<(u32, u32)>`

- **Does:** Appends `w:abstractNum`s and `w:num`s to `main`'s numbering part, making one if needed.
- **Calls (crate):** `numbering_part` (markdown/package.rs:1123) L1190; `PartFs::part_string` (opc/mod.rs:636) L1191; `max_attribute` (markdown/package.rs:1097) L1193,1194; `build` (markdown/unified.rs:526) (by name) L1195; `PartFs::set_part` (opc/mod.rs:661) L1217
- **Calls (std/external/unresolved):** `.filter_map`, `.find`, `.into_bytes`, `.iter`, `.max`, `.min`, `.unwrap_or`, `Some`, `next`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `carry_styles_and_numbering` (append.rs:867); `numbering` (markdown/package.rs:1149); `label_numbering` (markdown/package.rs:1165)

#### `ensure_footnotes_part`  (L1225-1227, pub(crate))
`pub(crate) fn ensure_footnotes_part(package: &mut PartFs, main: &str) -> String`

- **Does:** The footnotes part `main` relates to; created with Word's separator and continuation separator notes, its relationship and its content type when absent.
- **Calls (crate):** `ensure_notes_part` (markdown/package.rs:1233) L1226
- **Called by:** none found in src

#### `ensure_notes_part`  (L1233-1304)
`fn ensure_notes_part( package: &mut PartFs, main: &str, kind: &str, changes: Option<&[String]>, ) -> String`

- **Does:** The notes part of `kind` (`footnote`, `endnote`) `main` relates to, made as [`ensure_footnotes_part`] makes one.
- **Calls (crate):** `related` (markdown/package.rs:512) L1257; `PartFs::part_string` (opc/mod.rs:636) L1258,1276; `PartFs::set_part` (opc/mod.rs:661) L1273,1278,1293; `separator_formats` (markdown/package.rs:1308) L1278; `sibling` (markdown/package.rs:504) L1283; `PartFs::add_document_relationship` (opc/mod.rs:1003) L1284; `relative_rel_target` (opc/mod.rs:180) L1287; `PartFs::add_content_type_override` (opc/mod.rs:845) L1289
- **Calls (std/external/unresolved):** `.contains`, `.first`, `.get`, `.into_bytes`, `.map_or_else`, `.rfind`, `.unwrap_or_default`, `separator`, `separators`
- **Macros:** `format!`
- **Called by:** `ensure_footnotes_part` (markdown/package.rs:1225); `footnotes` (markdown/package.rs:1348)

#### `separator_formats`  (L1308-1344)
`fn separator_formats(xml: &str, kind: &str, changes: &[String]) -> String`

- **Does:** Reusing a reference keeps its separator layout, while its revision metadata follows the current view (including removal of a fmt clause).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1309; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1310,1335; `Dom::root` (xmllinq/mod.rs:822) L1311,1336; `Dom::elements` (xmllinq/mod.rs:640) L1318,1331; `Dom::attribute` (xmllinq/mod.rs:874) L1321; `Dom::element` (xmllinq/mod.rs:655) L1323,1326; `Dom::new_element` (xmllinq/mod.rs:478) L1327; `Dom::add_first` (xmllinq/mod.rs:1101) L1328; `Dom::remove` (xmllinq/mod.rs:1118) L1332; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1337; `Dom::add` (xmllinq/mod.rs:1074) L1338,1339; `serialize_element` (xmllinq/serialize.rs:401) L1343
- **Calls (std/external/unresolved):** `.enumerate`, `.find`, `.get`, `.into_iter`, `.to_string`, `.unwrap_or_else`, `Some`, `W::name`, `W::p_pr`, `W::p`
- **Macros:** `format!`
- **Called by:** `ensure_notes_part` (markdown/package.rs:1233)

#### `footnotes`  (L1348-1435)
`fn footnotes( package: &mut PartFs, main: &str, context: &mut Context<'_>, media: &mut Media, separator_changes: &HashMap<&str, Vec<String>>, )`

- **Does:** Writes the footnotes and the endnotes into their notes parts, making each part a kind of note needs.
- **Calls (crate):** `ensure_notes_part` (markdown/package.rs:1233) L1362,1366; `PartFs::part_string` (opc/mod.rs:636) L1367; `max_attribute` (markdown/package.rs:1097) L1370; `attribute_values` (markdown/package.rs:1102) L1392; `story` (markdown/xml.rs:786) L1413; `PartFs::set_part` (opc/mod.rs:661) L1432
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.find`, `.flat_map`, `.get`, `.insert_str`, `.into_bytes`, `.is_empty`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.ok`, `.push_str`, `.rfind`, `.unwrap_or`, `Some`, `String::new`, `std::mem::replace`, `u32::try_from`
- **Macros:** `format!`, `vec!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `max_drawing_id`  (L1438-1448, pub(crate))
`pub(crate) fn max_drawing_id(package: &PartFs) -> u32`

- **Does:** The largest picture id (`wp:docPr`) the kept parts use.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1439; `PartFs::part_string` (opc/mod.rs:636) L1443; `max_attribute` (markdown/package.rs:1097) L1444
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.filter_map`, `.filter`, `.iter`, `.max`, `.ok`, `.unwrap_or`, `u32::try_from`
- **Called by:** `assemble` (markdown/package.rs:103); `write` (markdown/view_stories.rs:1599)

#### `core_properties`  (L1454-1528)
`fn core_properties(package: &mut PartFs, document: &Document)`

- **Does:** `dc:title`, and the owner (`dc:creator`, or `cp:lastModifiedBy` alone) from the front matter.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1456; `PartFs::remove_part` (opc/mod.rs:710) L1463; `PartFs::remove_content_type_override` (opc/mod.rs:897) L1464; `PartFs::remove_package_relationships_by_type` (opc/mod.rs:914) L1465; `PartFs::part_string` (opc/mod.rs:636) L1503; `PartFs::add_content_type_override` (opc/mod.rs:845) L1510; `PartFs::add_package_relationship` (opc/mod.rs:1018) L1514; `edit_core` (markdown/package.rs:1549) L1525; `PartFs::set_part` (opc/mod.rs:661) L1526
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.as_ref`, `.ends_with`, `.find`, `.into_bytes`, `.into_iter`, `.is_none`, `.iter`, `.map`, `.starts_with`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103); `literal_package_metadata_contract_tests::front_matter_preserves_exact_core_property_owners_for_every_authored_shape` (markdown/package.rs:1999); `literal_package_metadata_contract_tests::absent_front_matter_fields_preserve_every_existing_core_property` (markdown/package.rs:2058)

#### `edit_core`  (L1549-1673)
`fn edit_core(xml: &str, edits: &[CoreEdit<'_>]) -> Option<String>`

- **Does:** `xml` with each edited property's first element given its value in place, keeping its own tag, its later copies dropped, and a property the part lacks written last in the root.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) (by name) L1613,1635; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1620,1647; `edit_core::{fn element}` (markdown/package.rs:1553) L1624,1650; `edit_core::{fn missing}` (markdown/package.rs:1566) L1642,1660
- **Calls (std/external/unresolved):** `.as_ref`, `.checked_sub`, `.filter`, `.flatten`, `.insert`, `.into_inner`, `.into_owned`, `.is_some`, `.iter`, `.len`, `.name`, `.ok`, `.position`, `.read_resolved_event`, `.read_to_end`, `.strip_prefix`, `.to_owned`, `.to_string`, `.write_event`, `BytesEnd::new`, `Event::Empty`, `Event::End`, `Event::Start`, `HashMap::new`, `String::from_utf8`, `Vec::new`, `edited`, `quick_xml::NsReader::from_str`, `quick_xml::Writer::new`
- **Macros:** `vec!`
- **Called by:** `core_properties` (markdown/package.rs:1454); `literal_package_metadata_contract_tests::a_self_closing_core_root_opens_to_take_the_owner` (markdown/package.rs:2040)

#### `edit_core::{fn element}`  (L1553-1564)
`fn element( writer: &mut quick_xml::Writer<Vec<u8>>, start: BytesStart<'_>, value: &str, ) -> Option<()>`

- **Does:** `start` holding `value`.
- **Calls (std/external/unresolved):** `.as_ref`, `.name`, `.ok`, `.to_owned`, `.write_event`, `BytesEnd::new`, `BytesText::new`, `Event::End`, `Event::Start`, `Event::Text`
- **Called by:** `build_table` (edit/structural.rs:153); `edit_core` (markdown/package.rs:1549); `edit_core::{fn missing}` (markdown/package.rs:1566)

#### `edit_core::{fn missing}`  (L1566-1590)
`fn missing( writer: &mut quick_xml::Writer<Vec<u8>>, edits: &[CoreEdit<'_>], written: &[bool], prefixes: &HashMap<String, String>, ) -> Option<()>`

- **Does:** The edits the root lacks, written last in it.
- **Calls (crate):** `edit_core::{fn element}` (markdown/package.rs:1553) L1587
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.get`, `.iter`, `.push_attribute`, `.zip`, `BytesStart::new`, `Some`
- **Macros:** `format!`
- **Called by:** `edit_core` (markdown/package.rs:1549); `missing_foreign_markers` (validate.rs:3454) (?)

#### `<PartRelate as Relate>::hyperlink`  (L1695-1701, impl Relate)
`fn hyperlink(&mut self, url: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::add_document_relationship_external` (opc/mod.rs:1047) L1696
- **Macros:** `format!`
- **Called by:** none found in src

#### `<PartRelate as Relate>::picture`  (L1703-1745, impl Relate)
`fn picture(&mut self, index: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1711; `sibling` (markdown/package.rs:504) L1714; `PartFs::set_part` (opc/mod.rs:661) L1723; `PartFs::content_type_for` (opc/mod.rs:837) L1724; `PartFs::add_content_type_default` (opc/mod.rs:852) L1725; `relative_rel_target` (opc/mod.rs:180) L1734; `PartFs::read_rels_for` (opc/mod.rs:991) L1735; `PartFs::add_document_relationship` (opc/mod.rs:1003) L1743
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.find`, `.insert`, `.into_iter`, `.is_none`, `.iter`, `.len`, `.resize`, `Some`
- **Macros:** `format!`
- **Called by:** none found in src

#### `<PartRelate as Relate>::drawing`  (L1747-1773, impl Relate)
`fn drawing( &mut self, index: usize, next_doc_pr: &mut u32, story: Option<&[(String, String)]>, ) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1754; `materialize` (markdown/from_docx/drawing.rs:1881) L1755; `fill_boxes` (markdown/from_docx/drawing.rs:1619) L1765
- **Calls (std/external/unresolved):** `.collect`, `.get`, `.into_iter`, `Some`
- **Called by:** none found in src

#### `default_template`  (L1777-1859)
`fn default_template(page: PageSize) -> Result<Vec<u8>, std::io::Error>`

- **Does:** The package a document is written into when there is no reference.
- **Calls (crate):** `default_section` (markdown/package.rs:40) L1778
- **Calls (std/external/unresolved):** `.as_bytes`, `.collect`, `.compression_method`, `.finish`, `.into_inner`, `.iter`, `.last_modified_time`, `.len`, `.map_err`, `.map`, `.start_file`, `.to_string`, `.write_all`, `Cursor::new`, `Ok`, `Vec::new`, `crate::opc::zip32_plan`, `std::io::Error::new`, `zip::DateTime::default`, `zip::ZipWriter::new`, `zip::write::SimpleFileOptions::default`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103)

## Test functions (12)

- `tests::the_text_width_reads_universal_measures_as_the_view_does` (L1867): _no doc comment_
- `tests::a_theme_colour_is_found_under_any_drawingml_prefix` (L1881): _no doc comment_
- `tests::the_final_section_is_found_only_as_the_bodys_last_child` (L1899): _no doc comment_
- `tests::a_main_part_without_a_usable_root_or_section_falls_back_to_the_page` (L1930): _no doc comment_
- `tests::the_text_width_is_the_page_less_its_margins` (L1949): _no doc comment_
- `tests::separators_survive_and_notes_do_not` (L1961): _no doc comment_
- `tests::missing_styles_are_added_and_present_ones_kept` (L1973): _no doc comment_
- `tests::max_attribute_reads_only_the_named_element` (L1982): _no doc comment_
- `literal_package_metadata_contract_tests::front_matter_preserves_exact_core_property_owners_for_every_authored_shape` (L1999): _no doc comment_
- `literal_package_metadata_contract_tests::a_self_closing_core_root_opens_to_take_the_owner` (L2040): _no doc comment_
- `literal_package_metadata_contract_tests::absent_front_matter_fields_preserve_every_existing_core_property` (L2058): _no doc comment_
- `literal_package_metadata_contract_tests::separator_notes_keep_complete_authored_markup_without_ordinary_notes` (L2089): _no doc comment_
