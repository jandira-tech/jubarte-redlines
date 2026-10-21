# Functions and methods in `src/convert/math_layout.rs`

27 non-test, 18 test. Part of the [symbol manifest](../README.md).

#### `layout_node`  (L50-58, pub(crate))
`pub(crate) fn layout_node( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, ) -> MathBox`

- **Does:** Lay out `node` (`m:oMath`, or one `m:f` / `m:rad` / `m:d`).
- **Calls (crate):** `layout` (convert/math_layout.rs:86) L57
- **Called by:** `layout_omml` (convert/math_layout.rs:64); `emit_math_box` (convert/mod.rs:15918)

#### `layout`  (L86-120)
`fn layout( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `is` (convert/math_layout.rs:122) L98,101,104,107,110,113...; `fraction` (convert/math_layout.rs:197) L99; `radical` (convert/math_layout.rs:305) L102; `delimiter` (convert/math_layout.rs:369) L105; `scripts` (convert/math_layout.rs:439) L108,111; `nary` (convert/math_layout.rs:472) L114; `Dom::name_is` (xmllinq/mod.rs:535) L116; `math_run` (convert/math_layout.rs:509) L117; `row_children` (convert/math_layout.rs:148) L119
- **Calls (std/external/unresolved):** `MathBox::default`, `W::r`
- **Called by:** `layout_node` (convert/math_layout.rs:50); `row_children` (convert/math_layout.rs:148); `fraction` (convert/math_layout.rs:197); `stacked` (convert/math_layout.rs:277); `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `scripts` (convert/math_layout.rs:439); `nary` (convert/math_layout.rs:472)

#### `is`  (L122-124)
`fn is(dom: &Dom, node: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L123
- **Calls (std/external/unresolved):** `M::name`
- **Called by:** `layout` (convert/math_layout.rs:86); `nary` (convert/math_layout.rs:472); `mark_close` (markdown/critic.rs:1750) (?); `mark_closes` (markdown/critic.rs:1774) (?)

#### `is_property`  (L126-129)
`fn is_property(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L127; `XName::local_name` (xmllinq/mod.rs:208) (by name) L128
- **Calls (std/external/unresolved):** `.ends_with`, `.is_some_and`
- **Called by:** `row_children` (convert/math_layout.rs:148)

#### `child`  (L131-133)
`fn child(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L132
- **Calls (std/external/unresolved):** `M::name`
- **Called by:** `flag_on` (convert/math_layout.rs:141); `fraction` (convert/math_layout.rs:197); `stacked` (convert/math_layout.rs:277); `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `scripts` (convert/math_layout.rs:439); `nary` (convert/math_layout.rs:472); `fence_char` (convert/math_layout.rs:559)

#### `val_of`  (L135-138)
`fn val_of(dom: &Dom, node: NodeId) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L136; `attr_any` (convert/mod.rs:19993) L137
- **Calls (std/external/unresolved):** `.or_else`, `M::name`
- **Called by:** `flag_on` (convert/math_layout.rs:141); `fraction` (convert/math_layout.rs:197); `nary` (convert/math_layout.rs:472); `fence_char` (convert/math_layout.rs:559)

#### `flag_on`  (L141-146)
`fn flag_on(dom: &Dom, parent: NodeId, local: &str) -> bool`

- **Does:** A present on/off element is on unless its value is 0, off, or false.
- **Calls (crate):** `child` (convert/math_layout.rs:131) L142; `val_of` (convert/math_layout.rs:135) L145
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `matches!`
- **Called by:** `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369)

#### `row_children`  (L148-163)
`fn row_children( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L156; `is_property` (convert/math_layout.rs:126) L159; `layout` (convert/math_layout.rs:86) L160; `hstack` (convert/math_layout.rs:165) L162
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`
- **Called by:** `layout` (convert/math_layout.rs:86); `fraction` (convert/math_layout.rs:197)

#### `hstack`  (L165-177)
`fn hstack(parts: Vec<MathBox>) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `append` (convert/math_layout.rs:179) L172
- **Calls (std/external/unresolved):** `.max`, `MathBox::default`
- **Called by:** `row_children` (convert/math_layout.rs:148); `delimiter` (convert/math_layout.rs:369); `nary` (convert/math_layout.rs:472)

#### `append`  (L179-190)
`fn append(out: &mut MathBox, mut part: MathBox, x: f32, y: f32)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.append`
- **Called by:** `hstack` (convert/math_layout.rs:165); `fraction` (convert/math_layout.rs:197); `stacked` (convert/math_layout.rs:277); `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `scripts` (convert/math_layout.rs:439); `nary` (convert/math_layout.rs:472); `wrap_listing_runs` (convert/mod.rs:8109) (?)

#### `shifted`  (L193-195)
`fn shifted(part: &MathBox, dy: f32) -> (f32, f32)`

- **Does:** Ascent and descent after a baseline shift of `dy` (+up).
- **Calls (std/external/unresolved):** `.max`
- **Called by:** `nary` (convert/math_layout.rs:472)

#### `fraction`  (L197-273)
`fn fraction( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `child` (convert/math_layout.rs:131) L205,206,238,241; `val_of` (convert/math_layout.rs:135) L207; `stacked` (convert/math_layout.rs:277) L210; `row_children` (convert/math_layout.rs:148) L213; `MathFont::constants` (convert/math_font.rs:131) L215; `MathFont::scale` (convert/math_font.rs:136) L216,217,244,245,246,247; `layout` (convert/math_layout.rs:86) L239,242; `append` (convert/math_layout.rs:179) L262,263; `fit` (convert/math_layout.rs:730) L271
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.map`, `.max`, `.push`, `.unwrap_or_default`, `.unwrap_or`, `MathBox::default`
- **Called by:** `layout` (convert/math_layout.rs:86); `theme_colour` (markdown/package.rs:964) (?)

#### `stacked`  (L277-303)
`fn stacked( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, ) -> MathBox`

- **Does:** noBar, nested: today's stack.
- **Calls (crate):** `child` (convert/math_layout.rs:131) L286,289; `layout` (convert/math_layout.rs:86) L287,290; `append` (convert/math_layout.rs:179) L298,299; `fit` (convert/math_layout.rs:730) L301
- **Calls (std/external/unresolved):** `.map`, `.max`, `.unwrap_or_default`, `MathBox::default`
- **Called by:** `fraction` (convert/math_layout.rs:197)

#### `radical`  (L305-367)
`fn radical( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `MathFont::constants` (convert/math_font.rs:131) L313; `MathFont::scale` (convert/math_font.rs:136) L314,316,318,320,339,340; `child` (convert/math_layout.rs:131) L321,322,334; `flag_on` (convert/math_layout.rs:141) L321; `layout` (convert/math_layout.rs:86) L323,335; `stretch_glyph` (convert/math_layout.rs:575) L326; `append` (convert/math_layout.rs:179) L350,354,355; `fit` (convert/math_layout.rs:730) L363
- **Calls (std/external/unresolved):** `.is_some_and`, `.map`, `.max`, `.push`, `.unwrap_or_default`, `MathBox::default`
- **Called by:** `layout` (convert/math_layout.rs:86); `tests::math_layout_radical_rule_clears_the_body` (convert/math_layout.rs:882); `tests::math_layout_radical_degree_sits_above_and_left` (convert/math_layout.rs:899); `tests::math_layout_nests_a_fraction_in_a_radical_in_brackets` (convert/math_layout.rs:986)

#### `delimiter`  (L369-437)
`fn delimiter( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `child` (convert/math_layout.rs:131) L377,381; `fence_char` (convert/math_layout.rs:559) L378,379,380; `flag_on` (convert/math_layout.rs:141) L381; `Dom::elements` (xmllinq/mod.rs:640) L382; `layout` (convert/math_layout.rs:86) L385; `hstack` (convert/math_layout.rs:165) L387; `MathFont::scale` (convert/math_font.rs:136) L388; `MathFont::constants` (convert/math_font.rs:131) L388; `stretch_glyph` (convert/math_layout.rs:575) L396,413,427; `center_fence` (convert/math_layout.rs:704) L403,411,425; `append` (convert/math_layout.rs:179) L404,417,421,431; `fit` (convert/math_layout.rs:730) L435
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.into_iter`, `.is_none_or`, `.map`, `.max`, `M::name`, `MathBox::default`, `Some`
- **Called by:** `layout` (convert/math_layout.rs:86)

#### `scripts`  (L439-470)
`fn scripts( dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32, pre: bool, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `child` (convert/math_layout.rs:131) L449,452,453; `layout` (convert/math_layout.rs:86) L450,452,453; `append` (convert/math_layout.rs:179) L460,462,465; `fit` (convert/math_layout.rs:730) L468
- **Calls (std/external/unresolved):** `.as_ref`, `.map`, `.max`, `.unwrap_or_default`, `.unwrap_or`, `MathBox::default`
- **Called by:** `layout` (convert/math_layout.rs:86)

#### `nary`  (L472-507)
`fn nary(dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `child` (convert/math_layout.rs:131) L473,474; `val_of` (convert/math_layout.rs:135) L475; `text_run` (convert/math_layout.rs:530) L480; `Dom::elements` (xmllinq/mod.rs:640) L482; `is` (convert/math_layout.rs:122) L483,486,488,490; `layout` (convert/math_layout.rs:86) L495; `shifted` (convert/math_layout.rs:193) L496; `append` (convert/math_layout.rs:179) L500; `hstack` (convert/math_layout.rs:165) L506
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.push`, `.to_string`, `.unwrap_or`, `MathBox::default`, `Vec::new`
- **Called by:** `layout` (convert/math_layout.rs:86)

#### `math_run`  (L509-512)
`fn math_run(dom: &Dom, node: NodeId, font: &MathFont, size: f32) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `math_run_upright` (convert/mod.rs:15975) L510; `text_run` (convert/math_layout.rs:530) L511; `text_in` (convert/math_layout.rs:514) L511
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `layout` (convert/math_layout.rs:86)

#### `text_in`  (L514-518)
`fn text_in(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_text` (convert/math_layout.rs:520) L516
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `math_run` (convert/math_layout.rs:509)

#### `collect_text`  (L520-528)
`fn collect_text(dom: &Dom, node: NodeId, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::text_value` (xmllinq/mod.rs:547) L521; `Dom::child_count` (xmllinq/mod.rs:622) L525; `collect_text` (convert/math_layout.rs:520) L526; `Dom::child_at` (xmllinq/mod.rs:632) L526
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `text_in` (convert/math_layout.rs:514); `collect_text` (convert/math_layout.rs:520)

#### `text_run`  (L530-556)
`fn text_run(font: &MathFont, text: &str, upright: bool, size: f32) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `math_alphanumerics_cover` (convert/mod.rs:16008) L534; `MathFont::glyph_id` (convert/math_font.rs:150) L539,541; `MathFont::scale` (convert/math_font.rs:136) L543; `MathFont::glyph` (convert/math_font.rs:155) L543; `fit` (convert/math_layout.rs:730) L554
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.map`, `.or_else`, `.push`, `.to_string`, `.unwrap_or`, `.zip`, `MathBox::default`
- **Called by:** `holds_only_text` (comparer/finalize.rs:13600) (?); `nary` (convert/math_layout.rs:472); `math_run` (convert/math_layout.rs:509); `stretch_glyph` (convert/math_layout.rs:575)

#### `fence_char`  (L559-571)
`fn fence_char(dom: &Dom, pr: Option<NodeId>, local: &str, default: char) -> Option<char>`

- **Does:** A delimiter character.
- **Calls (crate):** `child` (convert/math_layout.rs:131) L563; `val_of` (convert/math_layout.rs:135) L566
- **Calls (std/external/unresolved):** `.chars`, `.next`, `Some`
- **Called by:** `delimiter` (convert/math_layout.rs:369)

#### `stretch_glyph`  (L575-609)
`fn stretch_glyph( font: &MathFont, ch: char, target: f32, size: f32, bottom_y: f32, grow: bool, ) -> MathBox`

- **Does:** Vertical glyph, ink bottom at `bottom_y`.
- **Calls (crate):** `MathFont::glyph_id` (convert/math_font.rs:150) L583; `text_run` (convert/math_layout.rs:530) L584; `fit` (convert/math_layout.rs:730) L588; `place_single` (convert/math_layout.rs:611) L593,603; `MathFont::units_per_em` (convert/math_font.rs:127) L595; `MathFont::stretch_vertical` (convert/math_font.rs:167) L601; `assemble` (convert/math_layout.rs:645) L606
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `tests::math_layout_radical_rule_reaches_the_top_of_an_assembled_sign` (convert/math_layout.rs:1063)

#### `place_single`  (L611-641)
`fn place_single( font: &MathFont, gid: u16, size: f32, bottom_y: f32, label: &str, may_scale: bool, target: f32, ) -> MathBox`

- **Does:** _no doc comment_
- **Calls (crate):** `MathFont::glyph` (convert/math_font.rs:155) L620,627; `MathFont::scale` (convert/math_font.rs:136) L621,629,638; `Glyph::height` (convert/math_font.rs:46) (by name) L621; `Glyph::depth` (convert/math_font.rs:50) (by name) L621; `fit` (convert/math_layout.rs:730) L639
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `MathBox::default`
- **Called by:** `stretch_glyph` (convert/math_layout.rs:575)

#### `assemble`  (L645-699)
`fn assemble( font: &MathFont, parts: &[super::math_font::AssemblyPart], overlap: f32, height: f32, size: f32, bottom_y: f32, label: &str, ) -> MathBox`

- **Does:** Stack assembly parts bottom to top.
- **Calls (crate):** `MathFont::scale` (convert/math_font.rs:136) L654,658,664,679,693,694; `MathFont::glyph` (convert/math_font.rs:155) L678; `fit` (convert/math_layout.rs:730) L697
- **Calls (std/external/unresolved):** `.ceil`, `.count`, `.filter`, `.find`, `.iter`, `.len`, `.map`, `.max`, `.push`, `.sum`, `.to_string`, `.unwrap_or`, `MathBox::default`, `String::new`
- **Called by:** `stretch_glyph` (convert/math_layout.rs:575)

#### `center_fence`  (L704-728)
`fn center_fence(font: &MathFont, mut fence: MathBox, axis: f32) -> MathBox`

- **Does:** A fence planted with its ink bottom at `axis - target/2` is already centered when it reached `target`.
- **Calls (crate):** `MathFont::glyph` (convert/math_font.rs:155) L708,713; `MathFont::scale` (convert/math_font.rs:136) L709,710,715,716; `Glyph::height` (convert/math_font.rs:46) (by name) L709,715; `Glyph::depth` (convert/math_font.rs:50) (by name) L710,716; `fit` (convert/math_layout.rs:730) L726
- **Calls (std/external/unresolved):** `.abs`, `.first`, `.fold`, `.iter`, `.max`, `.min`
- **Called by:** `delimiter` (convert/math_layout.rs:369)

#### `fit`  (L730-767)
`fn fit(b: &mut MathBox, font: &MathFont)`

- **Does:** _no doc comment_
- **Calls (crate):** `MathFont::glyph` (convert/math_font.rs:155) L752; `MathFont::scale` (convert/math_font.rs:136) L753,754,757; `Glyph::height` (convert/math_font.rs:46) (by name) L753; `Glyph::depth` (convert/math_font.rs:50) (by name) L754
- **Calls (std/external/unresolved):** `.max`, `.min`
- **Called by:** `fraction` (convert/math_layout.rs:197); `stacked` (convert/math_layout.rs:277); `radical` (convert/math_layout.rs:305); `delimiter` (convert/math_layout.rs:369); `scripts` (convert/math_layout.rs:439); `text_run` (convert/math_layout.rs:530); `stretch_glyph` (convert/math_layout.rs:575); `place_single` (convert/math_layout.rs:611); `assemble` (convert/math_layout.rs:645); `center_fence` (convert/math_layout.rs:704)

## Test functions (18)

- `layout_omml` (L64): Lay out the first `m:oMath` child of `omml`.
- `tests::stix` (L781): _no doc comment_
- `tests::lay` (L785): _no doc comment_
- `tests::italic` (L790): Mathematical italic, the character Word puts in the text layer.
- `tests::glyph` (L799): _no doc comment_
- `tests::radical` (L807): _no doc comment_
- `tests::opening` (L814): _no doc comment_
- `tests::span` (L821): _no doc comment_
- `tests::math_layout_bar_fraction_puts_the_rule_on_the_axis` (L829): _no doc comment_
- `tests::math_layout_inline_fraction_shrinks_numerator_and_denominator` (L869): _no doc comment_
- `tests::math_layout_radical_rule_clears_the_body` (L882): _no doc comment_
- `tests::math_layout_radical_degree_sits_above_and_left` (L899): _no doc comment_
- `tests::math_layout_delimiters_grow_to_cover_a_fraction` (L925): _no doc comment_
- `tests::math_layout_delimiters_cover_a_nobar_stack` (L959): _no doc comment_
- `tests::math_layout_nests_a_fraction_in_a_radical_in_brackets` (L986): _no doc comment_
- `tests::math_layout_superscript_inside_a_radical_uses_todays_script_shift` (L1024): _no doc comment_
- `tests::math_layout_fallback_font_still_stacks_a_bar_fraction` (L1047): _no doc comment_
- `tests::math_layout_radical_rule_reaches_the_top_of_an_assembled_sign` (L1063): _no doc comment_
