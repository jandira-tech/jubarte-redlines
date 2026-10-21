# Functions and methods in `src/edit/template.rs`

2 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `pair`  (L71-76)
`fn pair(target: &TargetPair) -> v2::Target`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `generate` (edit/template.rs:84)

#### `generate`  (L84-285, pub)
`pub fn generate(source: &[u8], request: &TemplateRequest) -> Result<GeneratedTemplate, EditError>`

- **Does:** Generate unchanged desired-text rows and an honest inventory.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L86,100,107,133; `decisions` (edit/source_refs.rs:25) L93; `for_edit` (edit/for_edit.rs:1581) L98; `pair` (edit/template.rs:71) L136,159,177,198,267; `PlanV2::validate` (edit/v2.rs:523) L274
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.filter`, `.find`, `.insert`, `.into`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.ok_or_else`, `.push`, `.take`, `.then_some`, `.truncate`, `BTreeSet::new`, `Default::default`, `Err`, `Ok`, `Some`, `Vec::new`, `usize::from`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32); `tests::read_cursor_cannot_generate_a_partial_template` (edit/template.rs:293); `tests::invalid_attribution_never_produces_a_saved_request` (edit/template.rs:306)

## Test functions (2)

- `tests::read_cursor_cannot_generate_a_partial_template` (L293): _no doc comment_
- `tests::invalid_attribution_never_produces_a_saved_request` (L306): _no doc comment_
