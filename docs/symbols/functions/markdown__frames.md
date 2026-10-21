# Functions and methods in `src/markdown/frames.rs`

5 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `valid`  (L37-52, pub(crate))
`pub(crate) fn valid(name: &str, value: &str) -> bool`

- **Does:** Whether `value` is one the schema allows for attribute `name`.
- **Calls (crate):** `measure` (markdown/frames.rs:56) L42,43
- **Calls (std/external/unresolved):** `.contains`, `.is_ok`, `.parse`, `one_of`
- **Called by:** `merged` (markdown/frames.rs:82); `tests::a_value_reads_only_as_its_attribute_allows` (markdown/frames.rs:187); `ParaFormat::read` (markdown/view_stories.rs:94)

#### `measure`  (L56-75)
`fn measure(value: &str, signed: bool) -> bool`

- **Does:** A twips measure: whole twips, or a universal measure (`12pt`, `1.5cm`), signed when `signed`.
- **Calls (std/external/unresolved):** `.all`, `.any`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.iter`, `.next`, `.splitn`, `.strip_prefix`, `.strip_suffix`, `.unwrap_or`
- **Called by:** `valid` (markdown/frames.rs:37)

#### `merged`  (L82-103, pub(crate))
`pub(crate) fn merged<'a>( chain: impl IntoIterator<Item = &'a [(String, String)]>, own: Option<&'a [(String, String)]>, ) -> Option<Vec<(String, String)>>`

- **Does:** The frame of a paragraph: `chain`, a style chain's frames from its base style to the paragraph's own style, then `own`, the paragraph's `w:framePr`, each overlaying the attributes before it.
- **Calls (crate):** `valid` (markdown/frames.rs:37) L91
- **Calls (std/external/unresolved):** `.chain`, `.clone_from`, `.clone`, `.find`, `.get_or_insert_with`, `.into_iter`, `.iter_mut`, `.iter`, `.next`, `.position`, `.push`, `.rsplit`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `Some`
- **Called by:** `tests::a_frame_reads_in_the_schemas_order_and_writes_back` (markdown/frames.rs:140); `tests::the_paragraph_s_own_frame_overlays_its_style_s` (markdown/frames.rs:160); `story_frame` (markdown/from_docx/mod.rs:4880)

#### `describe`  (L107-113, pub(crate))
`pub(crate) fn describe(frame: &[(String, String)]) -> String`

- **Does:** `{wrap: none, hAnchor: margin, y: 1}`, in the schema's order; `{}` for an empty frame.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `tests::a_frame_reads_in_the_schemas_order_and_writes_back` (markdown/frames.rs:140); `tests::the_paragraph_s_own_frame_overlays_its_style_s` (markdown/frames.rs:160); `story_frame` (markdown/from_docx/mod.rs:4880)

#### `xml`  (L116-125, pub(crate))
`pub(crate) fn xml(frame: &[(String, String)]) -> String`

- **Does:** `<w:framePr w:wrap="none" …/>`, attributes in the schema's order.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L120
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.push_str`, `String::from`
- **Macros:** `write!`
- **Called by:** `tests::a_frame_reads_in_the_schemas_order_and_writes_back` (markdown/frames.rs:140); `story_part` (markdown/view_stories.rs:1710)

## Test functions (4)

- `tests::pairs` (L132): _no doc comment_
- `tests::a_frame_reads_in_the_schemas_order_and_writes_back` (L140): _no doc comment_
- `tests::the_paragraph_s_own_frame_overlays_its_style_s` (L160): _no doc comment_
- `tests::a_value_reads_only_as_its_attribute_allows` (L187): _no doc comment_
