# Functions and methods in `src/markdown/package.rs`

46 non-test, 11 test. Part of the [symbol manifest](../README.md).

#### `default_section`  (L39-44)
`fn default_section(page: PageSize) -> &'static str`

- **Does:** The section a document gets when nothing else gives one.
- **Calls:** none
- **Called by:** `frame` (markdown/package.rs:365); `default_template` (markdown/package.rs:1556); `tests::a_main_part_without_a_usable_root_or_section_falls_back_to_the_page` (markdown/package.rs:1686)

#### `threads`  (L73-90)
`fn threads( package: &mut PartFs, main: &str, comments: &[xml::Comment], ) -> Result<(), MarkdownError>`

- **Does:** Replies (`c6 @AS re c5`) and resolved comments (`c5 @AC resolved`): Word keys both by the `w14:paraId` of a comment's last paragraph, in `commentsExtended.xml`.
- **Calls (crate):** `CommentFamily::load` (comments.rs:429) L81; `CommentFamily::set_thread` (comments.rs:763) (by name) L83; `CommentFamily::set_done` (comments.rs:774) (by name) L86; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L88
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.is_none`, `.is_some`, `.iter`, `.map_err`, `Ok`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `err`  (L92-94)
`fn err(message: impl std::fmt::Display) -> MarkdownError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `MarkdownError::Package`
- **Called by:** `Transaction::footnotes_story` (edit/notes.rs:51) (?); `invalid` (edit/v2.rs:447) (?); `PlanV2::validate` (edit/v2.rs:511) (?); `codec_error` (edit/v2.rs:803) (?)

#### `assemble`  (L97-305, pub(super))
`pub(super) fn assemble( document: &Document, options: &DocxOptions<'_>, ) -> Result<Vec<u8>, MarkdownError>`

- **Does:** The finished package.
- **Calls (crate):** `default_template` (markdown/package.rs:1556) L103; `PartFs::open` (opc/mod.rs:456) L105; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L107; `PartFs::part_string` (opc/mod.rs:498) (by name) L110,159,165; `frame` (markdown/package.rs:365) L119; `prune` (markdown/package.rs:426) L120; `styles_part` (markdown/package.rs:308) L122; `with_styles` (markdown/package.rs:864) L160; `with_named_styles` (markdown/package.rs:881) L161; `apply` (markdown/view_styles.rs:497) L162; `with_style_changes` (markdown/view_styles.rs:40) L163; `related` (markdown/package.rs:353) L164; `with_table_styles` (markdown/package.rs:537) L167; `PartFs::set_part` (opc/mod.rs:510) (by name) L168,277,293; `numbering` (markdown/package.rs:948) L171; `label_numbering` (markdown/package.rs:964) L172; `Document::section_breaks` (markdown/xml.rs:399) L174; `write` (markdown/view_stories.rs:1013) L193; `.apply` (ambiguous, 6 candidates) L209; `Document::section` (markdown/xml.rs:386) L209; `text_width` (markdown/package.rs:410) L210; `max_drawing_id` (markdown/package.rs:1232) L211; `escape` (markdown/xml.rs:426) L214,215; `format_changes` (markdown/view_stories.rs:199) L248; `footnotes` (markdown/package.rs:1147) L258; `story` (markdown/xml.rs:587) L273; `sibling` (markdown/package.rs:345) L276; `comments_part` (markdown/xml.rs:2020) L279; `PartFs::add_content_type_override` (opc/mod.rs:616) (by name) L281,302; `PartFs::add_document_relationship` (opc/mod.rs:772) (by name) L285; `relative_rel_target` (opc/mod.rs:129) L288; `threads` (markdown/package.rs:73) L290; `core_properties` (markdown/package.rs:1248) L292; `PartFs::content_type_for` (opc/mod.rs:608) (by name) L301; `PartFs::to_zip` (opc/mod.rs:548) (by name) L304
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.chain`, `.clone`, `.cloned`, `.collect`, `.copied`, `.extend`, `.filter`, `.find`, `.flat_map`, `.flatten`, `.insert`, `.into_bytes`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_none`, `.iter`, `.map_err`, `.map_or`, `.map`, `.max`, `.min`, `.ok_or_else`, `.saturating_add`, `.to_string`, `.to_vec`, `.unwrap_or_else`, `.values`, `HashMap::new`, `HashSet::new`, `MarkdownError::Reference`, `Media::default`, `Vec::new`, `super::view_stories::People::default`
- **Macros:** `format!`, `matches!`
- **Called by:** `markdown_to_docx` (markdown/write.rs:59)

#### `styles_part`  (L308-330, pub(crate))
`pub(crate) fn styles_part(package: &mut PartFs, main: &str) -> String`

- **Does:** The styles part `main` relates to, made empty when there is none.
- **Calls (crate):** `related` (markdown/package.rs:353) L309; `sibling` (markdown/package.rs:345) L310; `PartFs::add_document_relationship` (opc/mod.rs:772) L311; `relative_rel_target` (opc/mod.rs:129) L314; `PartFs::add_content_type_override` (opc/mod.rs:616) L316; `PartFs::set_part` (opc/mod.rs:510) L320
- **Calls (std/external/unresolved):** `.into_bytes`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `add_styles` (edit/structural.rs:267); `assemble` (markdown/package.rs:97)

#### `has_stories`  (L335-343, pub(super))
`pub(super) fn has_stories(reference: &[u8]) -> bool`

- **Does:** The part next to `main` named `name` (`word/` + `name`).
- **Calls (crate):** `PartFs::open` (opc/mod.rs:456) L336; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L339; `PartFs::part_string` (opc/mod.rs:498) (by name) L341
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_some_and`
- **Called by:** none found in src

#### `sibling`  (L345-350, pub(super))
`pub(super) fn sibling(main: &str, name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rsplit_once`, `.to_string`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97); `styles_part` (markdown/package.rs:308); `numbering_part` (markdown/package.rs:922); `ensure_notes_part` (markdown/package.rs:1032); `<PartRelate as Relate>::picture` (markdown/package.rs:1497); `story_part` (markdown/view_stories.rs:1118)

#### `related`  (L353-361, pub(super))
`pub(super) fn related(package: &PartFs, main: &str, kind: &str) -> Option<String>`

- **Does:** The part `main` relates to with the relationship type ending in `kind`.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L354; `PartFs::resolve_rel_target` (opc/mod.rs:600) L359; `PartFs::part_bytes` (opc/mod.rs:493) L360
- **Calls (std/external/unresolved):** `.as_deref`, `.ends_with`, `.find`, `.is_some`, `.iter`, `.then_some`, `Some`
- **Called by:** `assemble` (markdown/package.rs:97); `styles_part` (markdown/package.rs:308); `prune` (markdown/package.rs:426); `numbering_part` (markdown/package.rs:922); `ensure_notes_part` (markdown/package.rs:1032); `even_and_odd_headers` (markdown/view_stories.rs:1343)

#### `frame`  (L365-389)
`fn frame(source: &str, page: PageSize) -> (String, String)`

- **Does:** The document element's start tag, declaring the prefixes the body uses, and the body's final section properties.
- **Calls (crate):** `default_section` (markdown/package.rs:39) L366; `final_section` (markdown/package.rs:394) L387
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.find`, `.push_str`, `.push`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97); `tests::a_main_part_without_a_usable_root_or_section_falls_back_to_the_page` (markdown/package.rs:1686)

#### `final_section`  (L394-407)
`fn final_section(source: &str) -> Option<String>`

- **Does:** The `w:sectPr` that is the body's last child, without a tracked change of its own (`w:sectPrChange`): the reference's revisions are not the written document's.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L395; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L396; `Dom::root` (xmllinq/mod.rs:822) L397; `Dom::element` (xmllinq/mod.rs:655) L398; `Dom::elements` (xmllinq/mod.rs:640) L399,403; `Dom::name_is` (xmllinq/mod.rs:535) L400; `Dom::remove` (xmllinq/mod.rs:1102) L404; `serialize_element` (xmllinq/serialize.rs:401) L406
- **Calls (std/external/unresolved):** `.last`, `Some`, `W::body`, `W::name`, `W::sect_pr`
- **Called by:** `frame` (markdown/package.rs:365); `tests::the_final_section_is_found_only_as_the_bodys_last_child` (markdown/package.rs:1655)

#### `text_width`  (L410-422)
`fn text_width(section: &str) -> u32`

- **Does:** The page width less the side margins, in twentieths of a point.
- **Calls (crate):** `attribute` (markdown/unified.rs:705) (by name) L418,419,420
- **Calls (std/external/unresolved):** `.find`, `.len`, `.max`, `.ok`, `.parse`, `.saturating_sub`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97); `tests::the_text_width_is_the_page_less_its_margins` (markdown/package.rs:1705)

#### `prune`  (L426-461)
`fn prune(package: &mut PartFs, main: &str)`

- **Does:** Drops the reference body's relationships (pictures, links, comments...) and their parts, and the notes the body referenced.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L427,438,443; `PartFs::remove_relationships_by_type` (opc/mod.rs:675) L449; `PartFs::remove_related_part` (opc/mod.rs:694) L451; `related` (markdown/package.rs:353) L455; `PartFs::part_string` (opc/mod.rs:498) L456; `PartFs::set_part` (opc/mod.rs:510) L458; `separators_only` (markdown/package.rs:464) L458
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.find`, `.into_bytes`, `.is_some_and`, `.iter`, `.map`, `.unwrap_or_default`, `Some`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `separators_only`  (L464-502)
`fn separators_only(xml: &str) -> String`

- **Does:** A notes part with only its separator notes.
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.filter_map`, `.find`, `.iter`, `.len`, `.map_or`, `.map`, `.min_by_key`, `.push_str`, `.starts_with`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `prune` (markdown/package.rs:426); `tests::separators_survive_and_notes_do_not` (markdown/package.rs:1717); `literal_package_metadata_contract_tests::separator_notes_keep_complete_authored_markup_without_ordinary_notes` (markdown/package.rs:1845)

#### `renamed`  (L507-522)
`fn renamed(xml: &str, name: &str) -> String`

- **Does:** The table style `xml` with its `w:name` replaced by `name`: a built-in template keeps Word's definition, the name is the document's own.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L519; `word_name` (markdown/package.rs:528) L519
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map`, `.to_string`
- **Macros:** `format!`
- **Called by:** `with_table_styles` (markdown/package.rs:537)

#### `word_name`  (L528-530)
`fn word_name(name: &str) -> String`

- **Does:** The `w:name` value for `name`: no comma, which `w:name` forbids (`[^,]*`; Word reads one as the start of an alias list), so every docx the writer makes is Word-valid.
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `document_text_view` (debug.rs:2011) (?); `renamed` (markdown/package.rs:507); `with_table_styles` (markdown/package.rs:537)

#### `with_table_styles`  (L537-582)
`fn with_table_styles( styles: &str, wanted: &BTreeMap<String, (Option<u32>, Option<String>)>, theme: &str, ) -> String`

- **Does:** `styles` with a definition for each table style the view names that it lacks: one of Word's built-in Grid and List Tables as Word defines it, its colours from `theme`; else the name is the view's (`named …`), Word's own for an id Word names, else the id, on Word's Normal Table, with the legend's border all round when...
- **Calls (crate):** `escape` (markdown/xml.rs:426) L544,571,572; `built_in_table_style` (markdown/package.rs:593) L546; `renamed` (markdown/package.rs:507) L551; `word_table_style_name` (markdown/package.rs:676) L566; `word_name` (markdown/package.rs:528) L572
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map_or_else`, `.map`, `.or_else`, `.rfind`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `built_in_table_style`  (L593-601)
`fn built_in_table_style(id: &str, theme: &str) -> Option<String>`

- **Does:** The definition of `id` when it is one of Word's built-in Grid or List Tables (`GridTable4-Accent1`, `ListTable6Colorful`), named as Word names it (`Grid Table 4 Accent 1`), its colours from `theme`.
- **Calls (crate):** `built_in` (markdown/package.rs:605) L594; `escape` (markdown/xml.rs:426) L597,598; `with_theme_colours` (markdown/package.rs:711) L599
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `format!`
- **Called by:** `with_table_styles` (markdown/package.rs:537); `is_built_in_table_style` (markdown/package.rs:647); `word_table_style_name` (markdown/package.rs:676)

#### `built_in`  (L605-643)
`fn built_in(id: &str) -> Option<(String, String)>`

- **Does:** `id` as Word names it and its unstyled body, when it is one of the built-in Grid or List Tables.
- **Calls (std/external/unresolved):** `.chars`, `.find_map`, `.flatten`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.is_empty`, `.lines`, `.next`, `.push_str`, `.push`, `.replace`, `.split_once`, `.splitn`, `.then`, `.to_string`, `Some`, `String::new`, `template`
- **Macros:** `format!`, `matches!`
- **Called by:** `built_in_table_style` (markdown/package.rs:593)

#### `is_built_in_table_style`  (L647-649, pub(super))
`pub(super) fn is_built_in_table_style(id: &str) -> bool`

- **Does:** Whether `id` is one of Word's built-in Grid or List Tables, which the writer defines as Word does.
- **Calls (crate):** `built_in_table_style` (markdown/package.rs:593) L648
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `word_table_style_name`  (L676-705, pub(super))
`pub(super) fn word_table_style_name(id: &str) -> Option<String>`

- **Does:** Word's name for a built-in table style `id`: the Grid and List Tables from their definitions, the older families and the two defaults by the id itself (`TableGrid` → `Table Grid`, `LightShading-Accent1` → `Light Shading Accent 1`).
- **Calls (crate):** `built_in_table_style` (markdown/package.rs:593) L677
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.find`, `.is_alphabetic`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.is_uppercase`, `.last`, `.len`, `.push`, `.split_once`, `.then`, `.to_string`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `table_style_line` (markdown/from_docx/header.rs:961); `with_table_styles` (markdown/package.rs:537)

#### `with_theme_colours`  (L711-741)
`fn with_theme_colours(xml: &str, theme: &str) -> String`

- **Does:** `xml` with each `@` colour resolved from `theme` as Word resolves a theme colour: the element's `w:themeColor` (or `w:themeFill`), lightened by its tint and darkened by its shade; `auto` for a colour the theme does not hold.
- **Calls (crate):** `attribute` (markdown/unified.rs:705) (by name) L730,732; `theme_colour` (markdown/package.rs:763) L732
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.map`, `.replace`, `.split_inclusive`, `.split_once`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `built_in_table_style` (markdown/package.rs:593)

#### `drawing_prefix`  (L747-761)
`fn drawing_prefix(theme: &str) -> String`

- **Does:** The theme colour `name` (`accent1`, `text1`), tinted and shaded as Word does: by lightness, in HSL, each channel truncated.
- **Calls (std/external/unresolved):** `.find_map`, `.map`, `.match_indices`, `.rfind`, `.strip_prefix`, `.to_string`, `.unwrap_or_else`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `theme_colour` (markdown/package.rs:763); `tests::a_theme_colour_is_found_under_any_drawingml_prefix` (markdown/package.rs:1637)

#### `theme_colour`  (L763-814)
`fn theme_colour( theme: &str, name: &str, tint: Option<&str>, shade: Option<&str>, ) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `drawing_prefix` (markdown/package.rs:747) L778; `rgb_to_hls` (markdown/package.rs:816) L793; `fraction` (convert/math_layout.rs:197) (by name) L799,802; `hls_to_rgb` (markdown/package.rs:839) L805
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.clamp`, `.filter`, `.find_map`, `.find`, `.floor`, `.get`, `.is_ascii_hexdigit`, `.is_none`, `.iter`, `.len`, `.map`, `.ok`, `.split_once`, `.to_ascii_uppercase`, `.unwrap_or`, `Some`, `byte`, `channel`, `f64::from`, `u8::from_str_radix`
- **Macros:** `format!`
- **Called by:** `with_theme_colours` (markdown/package.rs:711); `tests::a_theme_colour_is_found_under_any_drawingml_prefix` (markdown/package.rs:1637)

#### `rgb_to_hls`  (L816-837)
`fn rgb_to_hls(r: f64, g: f64, b: f64) -> (f64, f64, f64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.abs`, `.max`, `.min`, `.rem_euclid`
- **Called by:** `theme_colour` (markdown/package.rs:763)

#### `hls_to_rgb`  (L839-862)
`fn hls_to_rgb(h: f64, l: f64, s: f64) -> (f64, f64, f64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rem_euclid`, `value`
- **Called by:** `theme_colour` (markdown/package.rs:763)

#### `with_styles`  (L864-876)
`fn with_styles(styles: &str, wanted: &BTreeSet<&str>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `style_definition` (markdown/xml.rs:2267) L868
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.is_empty`, `.iter`, `.rfind`, `.to_string`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97); `tests::missing_styles_are_added_and_present_ones_kept` (markdown/package.rs:1729)

#### `with_named_styles`  (L881-893)
`fn with_named_styles(styles: &str, named: &BTreeSet<String>) -> String`

- **Does:** `styles` with a definition for each style an id line names that it lacks: the reader's own definition when it has one (`Quote`), else a paragraph style on Normal.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L884; `style_definition` (markdown/xml.rs:2267) L885; `named_style_definition` (markdown/xml.rs:2255) L885
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map`, `.rfind`, `.to_string`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `max_attribute`  (L896-898)
`fn max_attribute(xml: &str, element: &str, name: &str) -> Option<i64>`

- **Does:** The largest value of attribute `name` on elements `element` in `xml`.
- **Calls (crate):** `attribute_values` (markdown/package.rs:901) L897
- **Calls (std/external/unresolved):** `.into_iter`, `.max`
- **Called by:** `append_numbering` (markdown/package.rs:984); `footnotes` (markdown/package.rs:1147); `max_drawing_id` (markdown/package.rs:1232); `tests::max_attribute_reads_only_the_named_element` (markdown/package.rs:1738); `literal_package_metadata_contract_tests::separator_notes_keep_complete_authored_markup_without_ordinary_notes` (markdown/package.rs:1845)

#### `attribute_values`  (L901-919)
`fn attribute_values(xml: &str, element: &str, name: &str) -> Vec<i64>`

- **Does:** Every numeric `name` attribute of `element` in `xml`.
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.len`, `.map_or`, `.map`, `.ok`, `.parse`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `max_attribute` (markdown/package.rs:896); `footnotes` (markdown/package.rs:1147)

#### `numbering_part`  (L922-944, pub(crate))
`pub(crate) fn numbering_part(package: &mut PartFs, main: &str) -> String`

- **Does:** The numbering part `main` relates to, made empty when there is none.
- **Calls (crate):** `related` (markdown/package.rs:353) L923; `sibling` (markdown/package.rs:345) L924; `PartFs::add_document_relationship` (opc/mod.rs:772) L925; `relative_rel_target` (opc/mod.rs:129) L928; `PartFs::add_content_type_override` (opc/mod.rs:616) L930; `PartFs::set_part` (opc/mod.rs:510) L934
- **Calls (std/external/unresolved):** `.into_bytes`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `splice_numbering` (edit/structural.rs:377); `append_numbering` (markdown/package.rs:984)

#### `numbering`  (L948-960)
`fn numbering(package: &mut PartFs, main: &str, document: &Document) -> Vec<u32>`

- **Does:** Adds the document's lists to the numbering part, making one if needed; returns each list's `w:numId`.
- **Calls (crate):** `append_numbering` (markdown/package.rs:984) L952; `numbering` (markdown/xml.rs:2168) L953
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.len`, `.map`, `.unwrap_or`, `Vec::new`, `u32::try_from`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `label_numbering`  (L964-979)
`fn label_numbering(package: &mut PartFs, main: &str, document: &Document) -> Vec<u32>`

- **Does:** Adds a list for each label an id line names; returns each label's `w:numId`.
- **Calls (crate):** `append_numbering` (markdown/package.rs:984) L968; `label_numbering` (markdown/xml.rs:2199) L969
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.len`, `.take`, `Vec::new`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `append_numbering`  (L984-1019, pub(crate))
`pub(crate) fn append_numbering( package: &mut PartFs, main: &str, build: impl FnOnce(u32, u32) -> (String, String), ) -> Option<(u32, u32)>`

- **Does:** Appends `w:abstractNum`s and `w:num`s to `main`'s numbering part, making one if needed.
- **Calls (crate):** `numbering_part` (markdown/package.rs:922) L989; `PartFs::part_string` (opc/mod.rs:498) L990; `max_attribute` (markdown/package.rs:896) L992,993; `build` (markdown/unified.rs:516) (by name) L994; `PartFs::set_part` (opc/mod.rs:510) L1016
- **Calls (std/external/unresolved):** `.filter_map`, `.find`, `.into_bytes`, `.iter`, `.max`, `.min`, `.unwrap_or`, `Some`, `next`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `carry_styles_and_numbering` (append.rs:861); `numbering` (markdown/package.rs:948); `label_numbering` (markdown/package.rs:964)

#### `ensure_footnotes_part`  (L1024-1026, pub(crate))
`pub(crate) fn ensure_footnotes_part(package: &mut PartFs, main: &str) -> String`

- **Does:** The footnotes part `main` relates to; created with Word's separator and continuation separator notes, its relationship and its content type when absent.
- **Calls (crate):** `ensure_notes_part` (markdown/package.rs:1032) L1025
- **Called by:** none found in src

#### `ensure_notes_part`  (L1032-1103)
`fn ensure_notes_part( package: &mut PartFs, main: &str, kind: &str, changes: Option<&[String]>, ) -> String`

- **Does:** The notes part of `kind` (`footnote`, `endnote`) `main` relates to, made as [`ensure_footnotes_part`] makes one.
- **Calls (crate):** `related` (markdown/package.rs:353) L1056; `PartFs::part_string` (opc/mod.rs:498) L1057,1075; `PartFs::set_part` (opc/mod.rs:510) L1072,1077,1092; `separator_formats` (markdown/package.rs:1107) L1077; `sibling` (markdown/package.rs:345) L1082; `PartFs::add_document_relationship` (opc/mod.rs:772) L1083; `relative_rel_target` (opc/mod.rs:129) L1086; `PartFs::add_content_type_override` (opc/mod.rs:616) L1088
- **Calls (std/external/unresolved):** `.contains`, `.first`, `.get`, `.into_bytes`, `.map_or_else`, `.rfind`, `.unwrap_or_default`, `separator`, `separators`
- **Macros:** `format!`
- **Called by:** `ensure_footnotes_part` (markdown/package.rs:1024); `footnotes` (markdown/package.rs:1147)

#### `separator_formats`  (L1107-1143)
`fn separator_formats(xml: &str, kind: &str, changes: &[String]) -> String`

- **Does:** Reusing a reference keeps its separator layout, while its revision metadata follows the current view (including removal of a fmt clause).
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1108; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1109,1134; `Dom::root` (xmllinq/mod.rs:822) L1110,1135; `Dom::elements` (xmllinq/mod.rs:640) L1117,1130; `Dom::attribute` (xmllinq/mod.rs:874) L1120; `Dom::element` (xmllinq/mod.rs:655) L1122,1125; `Dom::new_element` (xmllinq/mod.rs:478) L1126; `Dom::add_first` (xmllinq/mod.rs:1085) L1127; `Dom::remove` (xmllinq/mod.rs:1102) L1131; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1136; `Dom::add` (xmllinq/mod.rs:1058) L1137,1138; `serialize_element` (xmllinq/serialize.rs:401) L1142
- **Calls (std/external/unresolved):** `.enumerate`, `.find`, `.get`, `.into_iter`, `.to_string`, `.unwrap_or_else`, `Some`, `W::name`, `W::p_pr`, `W::p`
- **Macros:** `format!`
- **Called by:** `ensure_notes_part` (markdown/package.rs:1032)

#### `footnotes`  (L1147-1229)
`fn footnotes( package: &mut PartFs, main: &str, context: &mut Context<'_>, media: &mut Media, separator_changes: &HashMap<&str, Vec<String>>, )`

- **Does:** Writes the footnotes and the endnotes into their notes parts, making each part a kind of note needs.
- **Calls (crate):** `ensure_notes_part` (markdown/package.rs:1032) L1161,1165; `PartFs::part_string` (opc/mod.rs:498) L1166; `max_attribute` (markdown/package.rs:896) L1169; `attribute_values` (markdown/package.rs:901) L1191; `story` (markdown/xml.rs:587) L1208; `PartFs::set_part` (opc/mod.rs:510) L1226
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.find`, `.flat_map`, `.get`, `.insert_str`, `.into_bytes`, `.is_empty`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.ok`, `.push_str`, `.rfind`, `.unwrap_or`, `Some`, `String::new`, `std::mem::replace`, `u32::try_from`
- **Macros:** `format!`, `vec!`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `max_drawing_id`  (L1232-1242, pub(crate))
`pub(crate) fn max_drawing_id(package: &PartFs) -> u32`

- **Does:** The largest picture id (`wp:docPr`) the kept parts use.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L1233; `PartFs::part_string` (opc/mod.rs:498) L1237; `max_attribute` (markdown/package.rs:896) L1238
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.filter_map`, `.filter`, `.iter`, `.max`, `.ok`, `.unwrap_or`, `u32::try_from`
- **Called by:** `assemble` (markdown/package.rs:97); `write` (markdown/view_stories.rs:1013)

#### `core_properties`  (L1248-1322)
`fn core_properties(package: &mut PartFs, document: &Document)`

- **Does:** `dc:title`, and the owner (`dc:creator`, or `cp:lastModifiedBy` alone) from the front matter.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L1250; `PartFs::remove_part` (opc/mod.rs:527) L1257; `PartFs::remove_content_type_override` (opc/mod.rs:668) L1258; `PartFs::remove_package_relationships_by_type` (opc/mod.rs:685) L1259; `PartFs::part_string` (opc/mod.rs:498) L1297; `PartFs::add_content_type_override` (opc/mod.rs:616) L1304; `PartFs::add_package_relationship` (opc/mod.rs:787) L1308; `edit_core` (markdown/package.rs:1343) L1319; `PartFs::set_part` (opc/mod.rs:510) L1320
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.as_ref`, `.ends_with`, `.find`, `.into_bytes`, `.into_iter`, `.is_none`, `.iter`, `.map`, `.starts_with`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97); `literal_package_metadata_contract_tests::front_matter_preserves_exact_core_property_owners_for_every_authored_shape` (markdown/package.rs:1755); `literal_package_metadata_contract_tests::absent_front_matter_fields_preserve_every_existing_core_property` (markdown/package.rs:1814)

#### `edit_core`  (L1343-1467)
`fn edit_core(xml: &str, edits: &[CoreEdit<'_>]) -> Option<String>`

- **Does:** `xml` with each edited property's first element given its value in place, keeping its own tag, its later copies dropped, and a property the part lacks written last in the root.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) (by name) L1407,1429; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1414,1441; `edit_core::{fn element}` (markdown/package.rs:1347) L1418,1444; `edit_core::{fn missing}` (markdown/package.rs:1360) L1436,1454
- **Calls (std/external/unresolved):** `.as_ref`, `.checked_sub`, `.filter`, `.flatten`, `.insert`, `.into_inner`, `.into_owned`, `.is_some`, `.iter`, `.len`, `.name`, `.ok`, `.position`, `.read_resolved_event`, `.read_to_end`, `.strip_prefix`, `.to_owned`, `.to_string`, `.write_event`, `BytesEnd::new`, `Event::Empty`, `Event::End`, `Event::Start`, `HashMap::new`, `String::from_utf8`, `Vec::new`, `edited`, `quick_xml::NsReader::from_str`, `quick_xml::Writer::new`
- **Macros:** `vec!`
- **Called by:** `core_properties` (markdown/package.rs:1248); `literal_package_metadata_contract_tests::a_self_closing_core_root_opens_to_take_the_owner` (markdown/package.rs:1796)

#### `edit_core::{fn element}`  (L1347-1358)
`fn element( writer: &mut quick_xml::Writer<Vec<u8>>, start: BytesStart<'_>, value: &str, ) -> Option<()>`

- **Does:** `start` holding `value`.
- **Calls (std/external/unresolved):** `.as_ref`, `.name`, `.ok`, `.to_owned`, `.write_event`, `BytesEnd::new`, `BytesText::new`, `Event::End`, `Event::Start`, `Event::Text`
- **Called by:** `build_table` (edit/structural.rs:153); `edit_core` (markdown/package.rs:1343); `edit_core::{fn missing}` (markdown/package.rs:1360)

#### `edit_core::{fn missing}`  (L1360-1384)
`fn missing( writer: &mut quick_xml::Writer<Vec<u8>>, edits: &[CoreEdit<'_>], written: &[bool], prefixes: &HashMap<String, String>, ) -> Option<()>`

- **Does:** The edits the root lacks, written last in it.
- **Calls (crate):** `edit_core::{fn element}` (markdown/package.rs:1347) L1381
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.get`, `.iter`, `.push_attribute`, `.zip`, `BytesStart::new`, `Some`
- **Macros:** `format!`
- **Called by:** `edit_core` (markdown/package.rs:1343)

#### `<PartRelate as Relate>::hyperlink`  (L1489-1495, impl Relate)
`fn hyperlink(&mut self, url: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::add_document_relationship_external` (opc/mod.rs:806) L1490
- **Macros:** `format!`
- **Called by:** none found in src

#### `<PartRelate as Relate>::picture`  (L1497-1539, impl Relate)
`fn picture(&mut self, index: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L1505; `sibling` (markdown/package.rs:345) L1508; `PartFs::set_part` (opc/mod.rs:510) L1517; `PartFs::content_type_for` (opc/mod.rs:608) L1518; `PartFs::add_content_type_default` (opc/mod.rs:623) L1519; `relative_rel_target` (opc/mod.rs:129) L1528; `PartFs::read_rels_for` (opc/mod.rs:756) L1529; `PartFs::add_document_relationship` (opc/mod.rs:772) L1537
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.find`, `.insert`, `.into_iter`, `.is_none`, `.iter`, `.len`, `.resize`, `Some`
- **Macros:** `format!`
- **Called by:** none found in src

#### `<PartRelate as Relate>::drawing`  (L1541-1552, impl Relate)
`fn drawing(&mut self, index: usize, next_doc_pr: &mut u32) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L1543; `materialize` (markdown/from_docx/drawing.rs:1371) L1544
- **Calls (std/external/unresolved):** `.collect`, `.get`, `.into_iter`
- **Called by:** none found in src

#### `default_template`  (L1556-1629)
`fn default_template(page: PageSize) -> Result<Vec<u8>, std::io::Error>`

- **Does:** The package a document is written into when there is no reference.
- **Calls (crate):** `default_section` (markdown/package.rs:39) L1557
- **Calls (std/external/unresolved):** `.as_bytes`, `.compression_method`, `.finish`, `.into_inner`, `.last_modified_time`, `.start_file`, `.to_string`, `.write_all`, `Cursor::new`, `Ok`, `Vec::new`, `zip::DateTime::default`, `zip::ZipWriter::new`, `zip::write::SimpleFileOptions::default`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97)

## Test functions (11)

- `tests::a_theme_colour_is_found_under_any_drawingml_prefix` (L1637): _no doc comment_
- `tests::the_final_section_is_found_only_as_the_bodys_last_child` (L1655): _no doc comment_
- `tests::a_main_part_without_a_usable_root_or_section_falls_back_to_the_page` (L1686): _no doc comment_
- `tests::the_text_width_is_the_page_less_its_margins` (L1705): _no doc comment_
- `tests::separators_survive_and_notes_do_not` (L1717): _no doc comment_
- `tests::missing_styles_are_added_and_present_ones_kept` (L1729): _no doc comment_
- `tests::max_attribute_reads_only_the_named_element` (L1738): _no doc comment_
- `literal_package_metadata_contract_tests::front_matter_preserves_exact_core_property_owners_for_every_authored_shape` (L1755): _no doc comment_
- `literal_package_metadata_contract_tests::a_self_closing_core_root_opens_to_take_the_owner` (L1796): _no doc comment_
- `literal_package_metadata_contract_tests::absent_front_matter_fields_preserve_every_existing_core_property` (L1814): _no doc comment_
- `literal_package_metadata_contract_tests::separator_notes_keep_complete_authored_markup_without_ordinary_notes` (L1845): _no doc comment_
