# Functions and methods in `src/edit/resolution.rs`

12 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `escape_critic`  (L27-47)
`fn escape_critic(text: &str) -> String`

- **Does:** Escape CriticMarkup delimiters as the agent view does.
- **Calls (std/external/unresolved):** `.contains`, `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `text_from_to` (edit/resolution.rs:83); `resolution_lines` (edit/resolution.rs:238); `tests::critic_escaping_mirrors_the_view` (edit/resolution.rs:388)

#### `quoted`  (L50-52)
`fn quoted(text: &str) -> String`

- **Does:** JSON-quote as the other JSON lines do (`serde_json`).
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string`
- **Called by:** `Line::render` (edit/resolution.rs:124); `tests::quoting_escapes_quotes_and_backslashes` (edit/resolution.rs:383)

#### `short_place`  (L55-60)
`fn short_place(change: &Change) -> String`

- **Does:** The short paragraph (or story) a line names.
- **Calls (crate):** `change_id` (ids.rs:187) L57
- **Calls (std/external/unresolved):** `.as_deref`, `.map_or_else`, `.to_string`
- **Called by:** `resolution_lines` (edit/resolution.rs:238); `single_line` (edit/resolution.rs:316)

#### `pair_ref`  (L64-80)
`fn pair_ref(first: &Change, second: &Change) -> Option<String>`

- **Does:** The combined ref for a substitution pair in one story, or `None` when either half has no short spelling (then the halves print separately).
- **Calls (crate):** `change_id` (ids.rs:187) L65,66
- **Calls (std/external/unresolved):** `.then_some`, `Ok`
- **Macros:** `format!`, `matches!`
- **Called by:** `resolution_lines` (edit/resolution.rs:238)

#### `text_from_to`  (L83-112)
`fn text_from_to(kind: ChangeKind, move_from: bool, text: &str, accepted: bool) -> (String, String)`

- **Does:** The CriticMarkup and the replacement text for one text change.
- **Calls (crate):** `escape_critic` (edit/resolution.rs:27) L84
- **Calls (std/external/unresolved):** `.to_string`, `String::new`
- **Macros:** `format!`
- **Called by:** `single_line` (edit/resolution.rs:316)

#### `Line::render`  (L124-133)
`fn render(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `quoted` (edit/resolution.rs:50) L130,131
- **Macros:** `format!`
- **Called by:** none found in src

#### `is_substitution_pair`  (L139-165)
`fn is_substitution_pair(first: &Change, second: &Change) -> bool`

- **Does:** Whether two text changes in one paragraph read as one substitution in the view: a deletion beside an insertion (either order), the same story and paragraph, both resolved the same way.
- **Calls (crate):** `change_id` (ids.rs:187) L159,160
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Macros:** `matches!`
- **Called by:** `resolution_lines` (edit/resolution.rs:238)

#### `adjacent_changes`  (L170-232)
`fn adjacent_changes(source: &[u8]) -> Result<HashSet<(String, String)>, ChangeError>`

- **Does:** Adjacent revision elements, including across the inline containers that the view treats as transparent.
- **Calls (crate):** `strict_to_transitional_docx` (strict_translation.rs:749) L192; `PartFs::open` (opc/mod.rs:585) L195; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L196; `PartFs::parts` (opc/mod.rs:733) (by name) L200; `PartFs::part_string` (opc/mod.rs:636) (by name) L201; `Dom::new` (xmllinq/mod.rs:387) L204; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L205; `Dom::root` (xmllinq/mod.rs:822) L206; `part_story` (ids.rs:392) L210; `Dom::descendants` (xmllinq/mod.rs:668) L212; `adjacent_changes::{fn flatten}` (edit/resolution.rs:174) L214; `Dom::name_is` (xmllinq/mod.rs:535) L216,217; `Dom::attribute` (xmllinq/mod.rs:874) L219,220; `ChangeId::short` (ids.rs:155) (by name) L224,225; `ChangeId::new` (ids.rs:142) L224,225
- **Calls (std/external/unresolved):** `.ends_with`, `.filter`, `.insert`, `.into`, `.iter`, `.map_err`, `.to_string`, `.unwrap_or_else`, `.windows`, `ChangeError::Package`, `HashSet::new`, `Ok`, `Some`, `Vec::new`, `W::del`, `W::id`, `W::ins`, `W::p`
- **Called by:** `resolution_lines` (edit/resolution.rs:238)

#### `adjacent_changes::{fn flatten}`  (L174-190)
`fn flatten(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L175; `Dom::name` (xmllinq/mod.rs:526) L176; `XName::local_name` (xmllinq/mod.rs:208) (by name) L177; `adjacent_changes::{fn flatten}` (edit/resolution.rs:174) L179,183; `Dom::element` (xmllinq/mod.rs:655) L182
- **Calls (std/external/unresolved):** `.as_ref`, `.map`, `.push`, `W::name`
- **Called by:** `adjacent_changes` (edit/resolution.rs:170); `adjacent_changes::{fn flatten}` (edit/resolution.rs:174)

#### `resolution_lines`  (L238-313, pub)
`pub fn resolution_lines( source: &[u8], resolved: &ResolvedRevisions, ) -> Result<Vec<String>, ChangeError>`

- **Does:** Build the stdout lines for a pure accept/reject: `source` is the bytes before resolution, `resolved` what `apply_plan` reported.
- **Calls (crate):** `list_changes` (changes.rs:564) L242; `adjacent_changes` (edit/resolution.rs:170) L243; `is_substitution_pair` (edit/resolution.rs:139) L264; `pair_ref` (edit/resolution.rs:64) L266; `escape_critic` (edit/resolution.rs:27) L282,283; `.render` (ambiguous, 4 candidates) L291; `short_place` (edit/resolution.rs:55) L298; `single_line` (edit/resolution.rs:316) L309
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.contains`, `.is_some_and`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.zip`, `Ok`, `Vec::new`, `pos`
- **Macros:** `format!`
- **Called by:** `resolution_text` (edit/resolution.rs:369)

#### `single_line`  (L316-366)
`fn single_line(change: &Change, accepted: bool) -> String`

- **Does:** One line for a single (unpaired) change.
- **Calls (crate):** `short_place` (edit/resolution.rs:55) L318; `text_from_to` (edit/resolution.rs:83) L355; `.render` (ambiguous, 4 candidates) L358
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `Some`, `String::new`
- **Called by:** `resolution_lines` (edit/resolution.rs:238)

#### `resolution_text`  (L369-376, pub)
`pub fn resolution_text(source: &[u8], resolved: &ResolvedRevisions) -> Result<String, ChangeError>`

- **Does:** The lines as text: one per line, each with its newline.
- **Calls (crate):** `resolution_lines` (edit/resolution.rs:238) L371
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `Ok`, `String::new`
- **Called by:** none found in src

## Test functions (2)

- `tests::quoting_escapes_quotes_and_backslashes` (L383): _no doc comment_
- `tests::critic_escaping_mirrors_the_view` (L388): _no doc comment_
