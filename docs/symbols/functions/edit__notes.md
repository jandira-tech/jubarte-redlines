# Functions and methods in `src/edit/notes.rs`

7 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `Transaction::resolve_footnote`  (L24-47, pub(super))
`pub(super) fn resolve_footnote( &self, para: usize, projection: &Projection, after: &str, occurrence: Option<usize>, note: &str, outcome: &mut EditOutcome, ) -> Result<usize, (String, String)>`

- **Does:** Resolve an `insert_footnote`: the projection offset the reference run goes to, or the error code and message.
- **Calls (crate):** `check_text` (edit.rs:5368) L33; `Transaction::find_range` (edit.rs:3901) L43; `Transaction::check_insert_position` (edit.rs:4021) L44
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.map_err`, `.to_string`, `.trim`, `Err`, `Ok`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `Transaction::footnotes_story`  (L51-80, pub(super))
`pub(super) fn footnotes_story(&mut self) -> Result<usize, EditError>`

- **Does:** The footnotes story, parsed into the plan's DOM; the part is created first when the source has none.
- **Calls (crate):** `Opened::related` (inspect.rs:669) L52; `PartFs::part_string` (opc/mod.rs:498) (by name) L62; `err` (ambiguous: `err` (edit.rs:1401); `err` (markdown/package.rs:92)) L66,72; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L67; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L68; `part_story` (ids.rs:392) L74
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.iter`, `.len`, `.ok_or_else`, `.position`, `.push`, `Ok`, `crate::markdown::ensure_footnotes_part`
- **Macros:** `format!`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `Transaction::style_defined`  (L83-98, pub(super))
`pub(super) fn style_defined(&self, id: &str) -> bool`

- **Does:** Whether the styles part defines a style with id `id`.
- **Calls (crate):** `Opened::related` (inspect.rs:669) L84; `PartFs::part_string` (opc/mod.rs:498) L87; `Dom::new` (xmllinq/mod.rs:387) L90; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L91; `Dom::root` (xmllinq/mod.rs:822) L92; `Dom::elements` (xmllinq/mod.rs:640) L95; `Dom::attribute` (xmllinq/mod.rs:874) L97
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.next`, `Some`, `W::name`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `next_footnote_id`  (L102-110, pub(super))
`pub(super) fn next_footnote_id(dom: &Dom, root: NodeId) -> u32`

- **Does:** One past the highest `w:footnote` id under `root`, at least 1.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L103; `Dom::attribute` (xmllinq/mod.rs:874) L105
- **Calls (std/external/unresolved):** `.filter_map`, `.into_iter`, `.map_or`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::id`, `W::name`, `u32::try_from`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `reference_rpr`  (L114-129)
`fn reference_rpr(dom: &mut Dom, style: Option<&str>) -> NodeId`

- **Does:** Run properties of a reference mark: the style when defined, else direct superscript.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L115,118,123; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L119,124; `Dom::add` (xmllinq/mod.rs:1058) L120,125
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::r_pr`, `W::val`
- **Called by:** `append_footnote` (edit/notes.rs:132); `insert_reference` (edit/notes.rs:168)

#### `append_footnote`  (L132-164, pub(super))
`pub(super) fn append_footnote( dom: &mut Dom, root: NodeId, id: u32, text: &str, (text_style, reference_style): (Option<&str>, Option<&str>), )`

- **Does:** Append footnote `id` holding `text` to the footnotes root.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L139,141,143,144,149,152...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L140,145,158; `Dom::add` (xmllinq/mod.rs:1058) L146,147,151,153,154,160...; `reference_rpr` (edit/notes.rs:114) L150; `XNamespace::xml` (xmllinq/mod.rs:128) L158; `Dom::add_text` (xmllinq/mod.rs:1071) L159
- **Calls (std/external/unresolved):** `.name`, `.to_string`, `Some`, `W::id`, `W::name`, `W::p_pr`, `W::p`, `W::r`, `W::t`, `W::val`
- **Macros:** `format!`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `insert_reference`  (L168-190, pub(super))
`pub(super) fn insert_reference( dom: &mut Dom, paragraph: NodeId, at: usize, id: u32, style: Option<&str>, )`

- **Does:** Put a reference run for footnote `id` right after projection offset `at` of `paragraph`, splitting the run that holds it.
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L175,180; `attach_segment` (edit.rs:5588) L176,181; `split_run_at` (edit.rs:5771) L179; `run_of` (edit.rs:6008) L182; `Dom::new_element` (xmllinq/mod.rs:478) L183,186; `reference_rpr` (edit/notes.rs:114) L184; `Dom::add` (xmllinq/mod.rs:1058) L185,188; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L187; `Dom::add_after_self` (xmllinq/mod.rs:1131) L189
- **Calls (std/external/unresolved):** `.clone`, `.expect`, `.to_string`, `Some`, `W::id`, `W::name`, `W::r`
- **Called by:** `Transaction::apply` (edit.rs:4489)
