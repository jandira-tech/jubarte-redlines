# Functions and methods in `src/edit/resolution.rs`

12 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `escape_critic`  (L27-47)
`fn escape_critic(text: &str) -> String`

- **Does:** Escape CriticMarkup delimiters as the agent view does.
- **Calls (std/external/unresolved):** `.contains`, `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `text_from_to` (edit/resolution.rs:83); `resolution_lines` (edit/resolution.rs:236); `tests::critic_escaping_mirrors_the_view` (edit/resolution.rs:386)

#### `quoted`  (L50-52)
`fn quoted(text: &str) -> String`

- **Does:** JSON-quote as the other JSON lines do (`serde_json`).
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string`
- **Called by:** `Line::render` (edit/resolution.rs:123); `tests::quoting_escapes_quotes_and_backslashes` (edit/resolution.rs:381)

#### `short_place`  (L55-60)
`fn short_place(change: &Change) -> String`

- **Does:** The short paragraph (or story) a line names.
- **Calls (crate):** `change_id` (ids.rs:187) L57
- **Calls (std/external/unresolved):** `.as_deref`, `.map_or_else`, `.to_string`
- **Called by:** `resolution_lines` (edit/resolution.rs:236); `single_line` (edit/resolution.rs:314)

#### `pair_ref`  (L64-80)
`fn pair_ref(first: &Change, second: &Change) -> Option<String>`

- **Does:** The combined ref for a substitution pair in one story, or `None` when either half has no short spelling (then the halves print separately).
- **Calls (crate):** `change_id` (ids.rs:187) L65,66
- **Calls (std/external/unresolved):** `.then_some`, `Ok`
- **Macros:** `format!`, `matches!`
- **Called by:** `resolution_lines` (edit/resolution.rs:236)

#### `text_from_to`  (L83-111)
`fn text_from_to(kind: ChangeKind, move_from: bool, text: &str, accepted: bool) -> (String, String)`

- **Does:** The CriticMarkup and the replacement text for one text change.
- **Calls (crate):** `escape_critic` (edit/resolution.rs:27) L84
- **Calls (std/external/unresolved):** `.to_string`, `String::new`
- **Macros:** `format!`
- **Called by:** `single_line` (edit/resolution.rs:314)

#### `Line::render`  (L123-132)
`fn render(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `quoted` (edit/resolution.rs:50) L129,130
- **Macros:** `format!`
- **Called by:** none found in src

#### `is_substitution_pair`  (L138-164)
`fn is_substitution_pair(first: &Change, second: &Change) -> bool`

- **Does:** Whether two text changes in one paragraph read as one substitution in the view: a deletion beside an insertion (either order), the same story and paragraph, both resolved the same way.
- **Calls (crate):** `change_id` (ids.rs:187) L158,159
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Macros:** `matches!`
- **Called by:** `resolution_lines` (edit/resolution.rs:236)

#### `adjacent_changes`  (L169-230)
`fn adjacent_changes(source: &[u8]) -> Result<HashSet<(String, String)>, ChangeError>`

- **Does:** Adjacent revision elements, including across the inline containers that the view treats as transparent.
- **Calls (crate):** `strict_to_transitional_docx` (strict_translation.rs:743) L191; `PartFs::open` (opc/mod.rs:456) L193; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L194; `PartFs::parts` (opc/mod.rs:532) (by name) L198; `PartFs::part_string` (opc/mod.rs:498) (by name) L199; `Dom::new` (xmllinq/mod.rs:387) L202; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L203; `Dom::root` (xmllinq/mod.rs:822) L204; `part_story` (ids.rs:392) L208; `Dom::descendants` (xmllinq/mod.rs:668) L210; `adjacent_changes::{fn flatten}` (edit/resolution.rs:173) L212; `Dom::name_is` (xmllinq/mod.rs:535) L214,215; `Dom::attribute` (xmllinq/mod.rs:874) L217,218; `ChangeId::short` (ids.rs:155) (by name) L222,223; `ChangeId::new` (ids.rs:142) L222,223
- **Calls (std/external/unresolved):** `.ends_with`, `.filter`, `.insert`, `.into`, `.iter`, `.map_err`, `.to_string`, `.unwrap_or_else`, `.windows`, `ChangeError::Package`, `HashSet::new`, `Ok`, `Some`, `Vec::new`, `W::del`, `W::id`, `W::ins`, `W::p`
- **Called by:** `resolution_lines` (edit/resolution.rs:236)

#### `adjacent_changes::{fn flatten}`  (L173-189)
`fn flatten(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L174; `Dom::name` (xmllinq/mod.rs:526) L175; `XName::local_name` (xmllinq/mod.rs:208) (by name) L176; `adjacent_changes::{fn flatten}` (edit/resolution.rs:173) L178,182; `Dom::element` (xmllinq/mod.rs:655) L181
- **Calls (std/external/unresolved):** `.as_ref`, `.map`, `.push`, `W::name`
- **Called by:** `adjacent_changes` (edit/resolution.rs:169); `adjacent_changes::{fn flatten}` (edit/resolution.rs:173)

#### `resolution_lines`  (L236-311, pub)
`pub fn resolution_lines( source: &[u8], resolved: &ResolvedRevisions, ) -> Result<Vec<String>, ChangeError>`

- **Does:** Build the stdout lines for a pure accept/reject: `source` is the bytes before resolution, `resolved` what `apply_plan` reported.
- **Calls (crate):** `list_changes` (changes.rs:287) L240; `adjacent_changes` (edit/resolution.rs:169) L241; `is_substitution_pair` (edit/resolution.rs:138) L262; `pair_ref` (edit/resolution.rs:64) L264; `escape_critic` (edit/resolution.rs:27) L280,281; `.render` (ambiguous, 4 candidates) L289; `short_place` (edit/resolution.rs:55) L296; `single_line` (edit/resolution.rs:314) L307
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.contains`, `.is_some_and`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.zip`, `Ok`, `Vec::new`, `pos`
- **Macros:** `format!`
- **Called by:** `resolution_text` (edit/resolution.rs:367)

#### `single_line`  (L314-364)
`fn single_line(change: &Change, accepted: bool) -> String`

- **Does:** One line for a single (unpaired) change.
- **Calls (crate):** `short_place` (edit/resolution.rs:55) L316; `text_from_to` (edit/resolution.rs:83) L353; `.render` (ambiguous, 4 candidates) L356
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `Some`, `String::new`
- **Called by:** `resolution_lines` (edit/resolution.rs:236)

#### `resolution_text`  (L367-374, pub)
`pub fn resolution_text(source: &[u8], resolved: &ResolvedRevisions) -> Result<String, ChangeError>`

- **Does:** The lines as text: one per line, each with its newline.
- **Calls (crate):** `resolution_lines` (edit/resolution.rs:236) L369
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `Ok`, `String::new`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `resolved_or_view` (bin/jubarte.rs:1635)

## Test functions (2)

- `tests::quoting_escapes_quotes_and_backslashes` (L381): _no doc comment_
- `tests::critic_escaping_mirrors_the_view` (L386): _no doc comment_
