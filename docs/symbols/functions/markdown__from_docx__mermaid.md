# Functions and methods in `src/markdown/from_docx/mermaid.rs`

15 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `series`  (L18-65)
`fn series(chart: &Element, kind: &str) -> Vec<(String, Vec<String>, Vec<String>)>`

- **Does:** The chart's series: name, categories and values in document order.
- **Calls (crate):** `Element::find_all` (markdown/from_docx/ooxml.rs:404) L20,24; `.text` (ambiguous, 10 candidates) L29,40,51; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L35,46; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L37,48
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.find`, `.into_iter`, `.map`, `.parse`, `.push`, `.sort_by_key`, `.to_string`, `.trim`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `chart` (markdown/from_docx/mermaid.rs:89)

#### `title`  (L68-79)
`fn title(chart: &Element) -> String`

- **Does:** `chart`'s title, the run text under `c:title`.
- **Calls (crate):** `Element::find` (markdown/from_docx/ooxml.rs:396) L70; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L71; `.text` (ambiguous, 10 candidates) L74
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`, `.to_string`, `.trim`, `Vec::new`
- **Called by:** `chart` (markdown/from_docx/mermaid.rs:89)

#### `quoted`  (L82-84)
`fn quoted(text: &str) -> String`

- **Does:** A category or series name Mermaid quotes: `'` for `"`, one line.
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `chart` (markdown/from_docx/mermaid.rs:89)

#### `chart`  (L89-145, pub(crate))
`pub(crate) fn chart(xml: &str) -> Option<String>`

- **Does:** `chartSpace`'s XML as Mermaid — `pie` for a pie chart, `xychart-beta` for bar and line — `None` when the part is no chart these say.
- **Calls (crate):** `parse_xml` (markdown/from_docx/ooxml.rs:440) L90; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L91; `title` (markdown/from_docx/mermaid.rs:68) L95; `series` (markdown/from_docx/mermaid.rs:18) L96,112,113; `quoted` (markdown/from_docx/mermaid.rs:82) L100,107,128,134
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.chain`, `.collect`, `.find`, `.first`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.ok`, `.zip`, `Some`, `String::from`
- **Macros:** `format!`, `write!`
- **Called by:** `chart_mermaid` (markdown/from_docx/drawing.rs:753)

#### `parse`  (L159-227)
`fn parse(text: &str) -> Option<Mermaid>`

- **Does:** `text` as a [`Mermaid`]: `pie` or `xychart-beta`, `None` for anything else or a line the two cannot say.
- **Calls (crate):** `list` (markdown/from_docx/mermaid.rs:230) L206,208,210
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.lines`, `.map`, `.next`, `.push`, `.replace`, `.rsplit_once`, `.starts_with`, `.strip_prefix`, `.then_some`, `.to_string`, `.trim_matches`, `.trim_start_matches`, `.trim`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `patch` (markdown/from_docx/mermaid.rs:376)

#### `list`  (L230-247)
`fn list(text: &str) -> Option<Vec<String>>`

- **Does:** A Mermaid `[a, "b", …]` list's items; a comma inside quotes is text.
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.clear`, `.is_empty`, `.iter`, `.push`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.to_string`, `.trim_matches`, `.trim`, `String::new`, `Vec::new`
- **Called by:** `family_faces_in_with` (convert/font.rs:2473) (?); `tests::a_control_s_store_value_is_its_text_only_where_word_keeps_it_so` (markdown/data_stores.rs:663) (?); `parse` (markdown/from_docx/mermaid.rs:159)

#### `find_all_mut`  (L250-260)
`fn find_all_mut<'a>(element: &'a mut Element, name: &str, out: &mut Vec<&'a mut Element>)`

- **Does:** Every descendant of `root` with this local name, mutably.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L253; `find_all_mut` (markdown/from_docx/mermaid.rs:250) L256
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `find_all_mut` (markdown/from_docx/mermaid.rs:250); `sers_of` (markdown/from_docx/mermaid.rs:365); `patch` (markdown/from_docx/mermaid.rs:376)

#### `find_mut`  (L263-275)
`fn find_mut<'a>(element: &'a mut Element, name: &str) -> Option<&'a mut Element>`

- **Does:** First descendant of `element` with this local name, mutably.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L266; `find_mut` (markdown/from_docx/mermaid.rs:263) L269
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `find_mut` (markdown/from_docx/mermaid.rs:263); `set_ser` (markdown/from_docx/mermaid.rs:335); `first_mut` (markdown/from_docx/mermaid.rs:353); `patch` (markdown/from_docx/mermaid.rs:376)

#### `pt`  (L278-290)
`fn pt(prefix: &str, idx: usize, v: &str) -> Element`

- **Does:** A `c:pt` element holding `v` at `idx`, with `prefix` the chart's own.
- **Calls (std/external/unresolved):** `.to_string`, `Element::default`, `Node::Element`, `Node::Text`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `evaluate_views` (convert/preset_geom.rs:357) (?); `set_cache` (markdown/from_docx/mermaid.rs:295)

#### `set_cache`  (L295-325)
`fn set_cache(cache: &mut Element, values: &[String])`

- **Does:** `cache` (`strCache`, `numCache`, `strLit`, `numLit`) with `values` as its points: the `pt`s replaced, `ptCount` renumbered, anything else — a `formatCode` — kept.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L304,306; `local_name` (markdown/from_docx/mermaid.rs:327) L308; `pt` (markdown/from_docx/mermaid.rs:278) L322
- **Calls (std/external/unresolved):** `.drain`, `.enumerate`, `.extend`, `.iter`, `.len`, `.map`, `.push`, `.split_once`, `.to_string`, `.unwrap_or_default`, `Node::Element`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `set_ser` (markdown/from_docx/mermaid.rs:335)

#### `local_name`  (L327-329)
`fn local_name(name: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.unwrap_or`
- **Called by:** `set_cache` (markdown/from_docx/mermaid.rs:295)

#### `set_ser`  (L335-350)
`fn set_ser(ser: &mut Element, cats: &[String], vals: &[String]) -> bool`

- **Does:** `ser`'s `c:cat` and `c:val` points replaced by `cats`/`vals`; `false` when the series has no cache to write.
- **Calls (crate):** `find_mut` (markdown/from_docx/mermaid.rs:263) L337,343; `first_mut` (markdown/from_docx/mermaid.rs:353) L338,344; `set_cache` (markdown/from_docx/mermaid.rs:295) L340,346
- **Called by:** `patch` (markdown/from_docx/mermaid.rs:376)

#### `first_mut`  (L353-362)
`fn first_mut<'a>(element: &'a mut Element, names: &[&str]) -> Option<&'a mut Element>`

- **Does:** The first `names` descendant of `element`, mutably.
- **Calls (crate):** `find_mut` (markdown/from_docx/mermaid.rs:263) L356,359
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.is_some`, `.iter`
- **Called by:** `set_ser` (markdown/from_docx/mermaid.rs:335)

#### `sers_of`  (L365-369)
`fn sers_of(chart: &mut Element) -> Vec<&mut Element>`

- **Does:** `chart`'s `c:ser` elements.
- **Calls (crate):** `find_all_mut` (markdown/from_docx/mermaid.rs:250) L367
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `patch` (markdown/from_docx/mermaid.rs:376)

#### `patch`  (L376-414, pub(crate))
`pub(crate) fn patch(xml: &str, text: &str) -> Option<String>`

- **Does:** The edited fence's text applied to `chartSpace` XML: its series' categories and values, its title, rewritten in place so everything else — styling, axes, embedded data, rels — stays.
- **Calls (crate):** `parse` (markdown/from_docx/mermaid.rs:159) L377; `parse_xml` (markdown/from_docx/ooxml.rs:440) L378; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L379; `find_mut` (markdown/from_docx/mermaid.rs:263) L382,408; `find_all_mut` (markdown/from_docx/mermaid.rs:250) L393; `sers_of` (markdown/from_docx/mermaid.rs:365) L396; `set_ser` (markdown/from_docx/mermaid.rs:335) L402; `serialize` (markdown/from_docx/drawing.rs:538) L413
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.clone`, `.collect`, `.extend`, `.filter`, `.first_mut`, `.into_iter`, `.iter`, `.len`, `.map`, `.ok`, `.zip`, `Some`, `Vec::new`
- **Called by:** `Drawing::rebuild_mermaid` (markdown/from_docx/drawing.rs:1858); `tests::open_archive_compares_bit_11_only_for_non_ascii_names` (opc/archive.rs:1014) (?)
