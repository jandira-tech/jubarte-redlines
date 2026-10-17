# Functions and methods in `src/capabilities/discovery.rs`

8 non-test, 9 test. Part of the [symbol manifest](../README.md).

#### `expand`  (L16-35)
`fn expand(schema: &Value, root: &Value) -> Value`

- **Does:** _no doc comment_
- **Calls (crate):** `expand` (capabilities/discovery.rs:16) L20,28,32
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.expect`, `.get`, `.iter`, `.map`, `.pointer`, `.strip_prefix`, `Value::Array`, `Value::Object`
- **Called by:** `expand` (capabilities/discovery.rs:16); `profiles` (capabilities/discovery.rs:37)

#### `profiles`  (L37-80)
`fn profiles(runtime: &str) -> Vec<Value>`

- **Does:** _no doc comment_
- **Calls (crate):** `operation_schemas` (edit.rs:1446) L42; `expand` (capabilities/discovery.rs:16) L50; `support` (capabilities/discovery.rs:114) L75
- **Calls (std/external/unresolved):** `.as_array`, `.as_object`, `.as_str`, `.clone`, `.collect`, `.expect`, `.iter`, `.map`, `.push`, `.remove`, `.to_vec`, `Vec::new`
- **Macros:** `json!`
- **Called by:** `discovery` (capabilities/discovery.rs:193)

#### `targets`  (L82-112)
`fn targets(version: u32, schema: &Value) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_array`, `.as_str`, `.collect`, `.expect`, `.get`, `.into`, `.is_some`, `.iter`, `.map`, `.to_string`
- **Macros:** `vec!`
- **Called by:** `support` (capabilities/discovery.rs:114)

#### `support`  (L114-159)
`fn support(runtime: &str, version: u32, kind: &str, schema: &Value) -> Vec<Value>`

- **Does:** _no doc comment_
- **Calls (crate):** `targets` (capabilities/discovery.rs:82) L116; `operation_supports_keep` (edit.rs:1489) L125
- **Calls (std/external/unresolved):** `.get`, `.is_some`, `.push_str`, `.push`, `.to_string`, `Some`, `String::from`, `Vec::new`
- **Macros:** `json!`, `matches!`
- **Called by:** `profiles` (capabilities/discovery.rs:37)

#### `nearest`  (L161-168)
`fn nearest(name: &str, candidates: BTreeSet<String>) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `distance` (capabilities/discovery.rs:170) L164
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.sort`, `.take`
- **Called by:** `discovery` (capabilities/discovery.rs:193); `Ids::table_paragraph` (edit/origin.rs:95) (?); `operations` (markdown/patch.rs:496) (?)

#### `distance`  (L170-184)
`fn distance(a: &str, b: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.enumerate`, `.expect`, `.last`, `.min`, `usize::from`
- **Called by:** `nearest` (capabilities/discovery.rs:161); `tests::edit_distance_handles_unicode_and_empty_names` (capabilities/discovery.rs:392)

#### `discovery`  (L193-237, pub)
`pub fn discovery(runtime: &str, operation: Option<&str>, field: Option<&str>) -> Value`

- **Does:** Discover an operation, a field (optionally within an operation), or the full edit matrix when both are absent.
- **Calls (crate):** `profiles` (capabilities/discovery.rs:37) L194; `nearest` (capabilities/discovery.rs:161) L207,220; `dispatched_operation_kinds` (edit.rs:1477) L233
- **Calls (std/external/unresolved):** `.as_object_mut`, `.as_object`, `.as_str`, `.cloned`, `.collect`, `.expect`, `.filter`, `.flat_map`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.keys`, `.map`, `.retain`, `.to_string`, `.unwrap_or_default`
- **Macros:** `json!`
- **Called by:** `tests::assert_runtime_dispatch` (capabilities/discovery.rs:324); `tests::policy_rows_keep_untrackable_operations_and_direct_v1_explicit` (capabilities/discovery.rs:366); `tests::every_discovery_answer_carries_a_schema_version` (capabilities/discovery.rs:399); `tests::unsupported_names_render_as_prose` (capabilities/discovery.rs:412)

#### `discovery_text`  (L244-287, pub)
`pub fn discovery_text(answer: &Value) -> String`

- **Does:** Text presentation for focused queries; the default manifest renderer stays unchanged.
- **Calls (std/external/unresolved):** `.as_array`, `.as_object`, `.as_str`, `.collect`, `.expect`, `.filter_map`, `.filter`, `.flatten`, `.into_iter`, `.is_empty`, `.join`, `.push_str`, `.unwrap_or_default`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `tests::unsupported_names_render_as_prose` (capabilities/discovery.rs:412)

## Test functions (9)

- `tests::minimal` (L294): _no doc comment_
- `tests::assert_runtime_dispatch` (L324): _no doc comment_
- `tests::native_advertised_equals_dispatched` (L353): _no doc comment_
- `tests::python_advertised_equals_dispatched` (L357): _no doc comment_
- `tests::wasm_advertised_equals_dispatched` (L361): _no doc comment_
- `tests::policy_rows_keep_untrackable_operations_and_direct_v1_explicit` (L366): _no doc comment_
- `tests::edit_distance_handles_unicode_and_empty_names` (L392): _no doc comment_
- `tests::every_discovery_answer_carries_a_schema_version` (L399): _no doc comment_
- `tests::unsupported_names_render_as_prose` (L412): _no doc comment_
