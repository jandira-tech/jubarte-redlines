# Functions and methods in `src/capabilities/discovery.rs`

8 non-test, 10 test. Part of the [symbol manifest](../README.md).

#### `expand`  (L17-36)
`fn expand(schema: &Value, root: &Value) -> Value`

- **Does:** _no doc comment_
- **Calls (crate):** `expand` (capabilities/discovery.rs:17) L21,29,33
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.expect`, `.get`, `.iter`, `.map`, `.pointer`, `.strip_prefix`, `Value::Array`, `Value::Object`
- **Called by:** `expand` (capabilities/discovery.rs:17); `profiles` (capabilities/discovery.rs:38)

#### `profiles`  (L38-83)
`fn profiles(runtime: &str) -> Vec<Value>`

- **Does:** _no doc comment_
- **Calls (crate):** `operation_schemas` (edit.rs:1854) L43; `expand` (capabilities/discovery.rs:17) L51; `dispatches_v2` (capabilities.rs:372) L72; `support` (capabilities/discovery.rs:117) L78
- **Calls (std/external/unresolved):** `.as_array`, `.as_object`, `.as_str`, `.clone`, `.collect`, `.expect`, `.iter`, `.map`, `.push`, `.remove`, `.to_vec`, `Vec::new`
- **Macros:** `json!`
- **Called by:** `discovery` (capabilities/discovery.rs:225)

#### `targets`  (L85-115)
`fn targets(version: u32, schema: &Value) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_array`, `.as_str`, `.collect`, `.expect`, `.get`, `.into`, `.is_some`, `.iter`, `.map`, `.to_string`
- **Macros:** `vec!`
- **Called by:** `support` (capabilities/discovery.rs:117)

#### `support`  (L117-189)
`fn support(runtime: &str, version: u32, kind: &str, schema: &Value) -> Vec<Value>`

- **Does:** _no doc comment_
- **Calls (crate):** `targets` (capabilities/discovery.rs:85) L119; `dispatches_v2` (capabilities.rs:372) L122; `operation_supports_keep` (edit.rs:1912) L133; `operation_needs_keep` (edit.rs:1905) L135
- **Calls (std/external/unresolved):** `.get`, `.is_none`, `.is_some`, `.push_str`, `.push`, `.to_string`, `Some`, `String::from`, `Vec::new`
- **Macros:** `json!`, `matches!`
- **Called by:** `profiles` (capabilities/discovery.rs:38)

#### `nearest`  (L191-198)
`fn nearest(name: &str, candidates: BTreeSet<String>) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `distance` (capabilities/discovery.rs:200) L194
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.sort`, `.take`
- **Called by:** `discovery` (capabilities/discovery.rs:225)

#### `distance`  (L200-214)
`fn distance(a: &str, b: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.enumerate`, `.expect`, `.last`, `.min`, `usize::from`
- **Called by:** `nearest` (capabilities/discovery.rs:191); `tests::edit_distance_handles_unicode_and_empty_names` (capabilities/discovery.rs:468)

#### `discovery`  (L225-272, pub)
`pub fn discovery(runtime: &str, operation: Option<&str>, field: Option<&str>) -> Value`

- **Does:** Discover an operation, a field (optionally within an operation), or the full edit matrix when both are absent.
- **Calls (crate):** `profiles` (capabilities/discovery.rs:38) L226; `nearest` (capabilities/discovery.rs:191) L239,255; `dispatched_operation_kinds` (edit.rs:1891) L268; `dispatched_plan_versions` (capabilities.rs:385) L269
- **Calls (std/external/unresolved):** `.as_object_mut`, `.as_object`, `.as_str`, `.cloned`, `.collect`, `.expect`, `.filter`, `.flat_map`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.keys`, `.map`, `.retain`, `.to_string`, `.unwrap_or_default`
- **Macros:** `json!`
- **Called by:** `tests::assert_runtime_dispatch` (capabilities/discovery.rs:359); `tests::policy_rows_keep_untrackable_operations_and_direct_v1_explicit` (capabilities/discovery.rs:414); `tests::strike_and_restore_are_offered_under_keep_only` (capabilities/discovery.rs:440); `tests::every_discovery_answer_carries_a_schema_version` (capabilities/discovery.rs:475); `tests::unsupported_names_render_as_prose` (capabilities/discovery.rs:488)

#### `discovery_text`  (L279-322, pub)
`pub fn discovery_text(answer: &Value) -> String`

- **Does:** Text presentation for focused queries; the default manifest renderer stays unchanged.
- **Calls (std/external/unresolved):** `.as_array`, `.as_object`, `.as_str`, `.collect`, `.expect`, `.filter_map`, `.filter`, `.flatten`, `.into_iter`, `.is_empty`, `.join`, `.push_str`, `.unwrap_or_default`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `tests::unsupported_names_render_as_prose` (capabilities/discovery.rs:488)

## Test functions (10)

- `tests::minimal` (L329): _no doc comment_
- `tests::assert_runtime_dispatch` (L359): _no doc comment_
- `tests::native_advertised_equals_dispatched` (L401): _no doc comment_
- `tests::python_advertised_equals_dispatched` (L405): _no doc comment_
- `tests::wasm_advertised_equals_dispatched` (L409): _no doc comment_
- `tests::policy_rows_keep_untrackable_operations_and_direct_v1_explicit` (L414): _no doc comment_
- `tests::strike_and_restore_are_offered_under_keep_only` (L440): _no doc comment_
- `tests::edit_distance_handles_unicode_and_empty_names` (L468): _no doc comment_
- `tests::every_discovery_answer_carries_a_schema_version` (L475): _no doc comment_
- `tests::unsupported_names_render_as_prose` (L488): _no doc comment_
