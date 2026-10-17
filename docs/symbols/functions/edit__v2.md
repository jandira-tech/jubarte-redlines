# Functions and methods in `src/edit/v2.rs`

24 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `present`  (L405-407)
`fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error>`

- **Does:** Optional means absent, never explicit JSON null. _(from a code comment)_
- **Calls (std/external/unresolved):** `.map`, `T::deserialize`
- **Called by:** none found in src

#### `integer`  (L412-432)
`fn integer<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error>`

- **Does:** JSON Schema's integer type includes 1.0 and 1e0, unlike serde's u64. _(from a code comment)_
- **Calls (std/external/unresolved):** `.as_f64`, `.as_u64`, `.fract`, `Err`, `Ok`, `serde::de::Error::custom`, `serde_json::Number::deserialize`
- **Called by:** `present_integer` (edit/v2.rs:444)

#### `string_enum`  (L435-438)
`fn string_enum<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error>`

- **Does:** Serde's externally tagged unit enums also accept {"on": null}; the production schema permits only strings. _(from a code comment)_
- **Calls (std/external/unresolved):** `String::deserialize`, `T::deserialize`, `serde::de::value::StringDeserializer::::new`
- **Called by:** `present_string_enum` (edit/v2.rs:439)

#### `present_string_enum`  (L439-443)
`fn present_string_enum<'de, D: Deserializer<'de>, T: Deserialize<'de>>( d: D, ) -> Result<Option<T>, D::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `string_enum` (edit/v2.rs:435) L442
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `present_integer`  (L444-446)
`fn present_integer<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `integer` (edit/v2.rs:412) L445
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `invalid`  (L447-449)
`fn invalid(message: impl Into<String>) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `super::err` (ambiguous: `err` (edit.rs:1401); `err` (markdown/package.rs:92)) L448
- **Called by:** `declared_entry_count` (admission.rs:470) (?); `check_xml_depth` (admission.rs:678) (?); `picture` (edit/images.rs:86) (?); `require` (edit/v2.rs:450); `PlanV2::from_json` (edit/v2.rs:503); `load_stories` (edit.rs:1311) (?); `apply_markdown` (markdown/patch.rs:121) (?)

#### `require`  (L450-456)
`fn require(condition: bool, message: &str) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `invalid` (edit/v2.rs:447) L454
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Called by:** `Target::validate` (edit/v2.rs:467); `targets` (edit/v2.rs:474); `occurrence` (edit/v2.rs:481); `attachment` (edit/v2.rs:484); `PlanV2::validate` (edit/v2.rs:511); `Operation::validate` (edit/v2.rs:588)

#### `hash`  (L457-462)
`fn hash(value: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.contains`, `.is_ascii_digit`, `.len`
- **Called by:** `PlanV2::validate` (edit/v2.rs:511); `Operation::validate` (edit/v2.rs:588)

#### `unique`  (L463-465)
`fn unique<T: Ord>(values: &[T]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.len`
- **Called by:** `run_debug_diff` (bin/jubarte.rs:2836) (?); `targets` (edit/v2.rs:474); `PlanV2::validate` (edit/v2.rs:511)

#### `Target::validate`  (L467-472)
`fn validate(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:450) L468
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `targets`  (L474-480)
`fn targets(values: &[Target]) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:450) L475; `unique` (edit/v2.rs:463) L475; `.validate` (ambiguous, 4 candidates) L477
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `PlanV2::validate` (edit/v2.rs:511)

#### `occurrence`  (L481-483)
`fn occurrence(value: Option<u64>) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:450) L482
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Operation::validate` (edit/v2.rs:588)

#### `attachment`  (L484-496)
`fn attachment(text: &str, comment: Option<&AttachedComment>) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:450) L486,490
- **Calls (std/external/unresolved):** `.is_empty`, `Ok`
- **Called by:** `Operation::validate` (edit/v2.rs:588)

#### `PlanV2::from_json`  (L503-507, pub)
`pub fn from_json(json: &str) -> Result<Self, EditError>`

- **Does:** Parse strictly with stable [`EditError`] codes: `INVALID_PLAN` for malformed JSON, unknown/missing fields, invalid values or incompatible field combinations; `UNSUPPORTED_SCHEMA` for another integer version.
- **Calls (crate):** `invalid` (edit/v2.rs:447) L504
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `.validate`, `Ok`, `serde_json::from_str`
- **Called by:** none found in src

#### `PlanV2::validate`  (L511-562, pub)
`pub fn validate(&self) -> Result<(), EditError>`

- **Does:** Validate the production request contract after constructing/modifying a value.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit.rs:1401); `err` (markdown/package.rs:92)) L513; `require` (edit/v2.rs:450) L522,526,530,537,543,547; `hash` (edit/v2.rs:457) L523; `is_author_whitespace` (edit/v2.rs:695) L527; `timestamp` (edit/v2.rs:704) L531; `targets` (edit/v2.rs:474) L534,535,538; `.validate` (ambiguous, 4 candidates) L542,556; `unique` (edit/v2.rs:463) L550
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.enumerate`, `.is_empty`, `.is_some`, `.iter`, `.map_err`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** none found in src

#### `PlanV2::to_json`  (L566-568, pub)
`pub fn to_json(&self) -> String`

- **Does:** Serialize without changing desired characters, attribution or array order.
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** none found in src

#### `Operation::target`  (L573-587, pub)
`pub fn target(&self) -> &Target`

- **Does:** Original source reference shared by every operation kind.
- **Calls:** none
- **Called by:** `parse_json` (cli.rs:3053) (?); `Operation::validate` (edit/v2.rs:588); `Transaction::resolve_thread` (edit.rs:3342) (?); `Transaction::check_thread_ops` (edit.rs:4326) (?); `run` (update.rs:186) (?)

#### `Operation::validate`  (L588-689)
`fn validate(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `.validate` (ambiguous, 4 candidates) L589,678; `Operation::target` (edit/v2.rs:573) L589; `require` (edit/v2.rs:450) L597,610,619,631,632,639...; `hash` (edit/v2.rs:457) L598; `attachment` (edit/v2.rs:484) L601,612,649,655; `occurrence` (edit/v2.rs:481) L611,620,648,676
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `Ok`, `usize::from`
- **Called by:** none found in src

#### `is_author_whitespace`  (L695-697)
`fn is_author_whitespace(c: char) -> bool`

- **Does:** Engine-independent whitespace class for the author field: the union of Rust's `char::is_whitespace` (Unicode White_Space) and U+FEFF. _(from a code comment)_
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `PlanV2::validate` (edit/v2.rs:511)

#### `timestamp`  (L704-762)
`fn timestamp(value: &str) -> bool`

- **Does:** Calendar/time/offset validation for the schema's uppercase RFC 3339 shape. _(from a code comment)_
- **Calls (crate):** `number` (ambiguous: `deep_lcs_source_conservation_tests::number` (comparer/lcs.rs:15436); `number` (ids.rs:211)) L722,723,724,725,726,727
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_bytes`, `.bytes`, `.contains`, `.filter`, `.get`, `.is_ascii_digit`, `.iter`, `.len`, `.ok`, `.parse`
- **Called by:** `PlanV2::validate` (edit/v2.rs:511)

#### `logical_text`  (L783-789, pub)
`pub fn logical_text(dom: &Dom, paragraph: NodeId) -> Result<String, EditError>`

- **Does:** Exact logical text codec v1 for a supported `w:p` in an admitted DOM.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L784; `codec_error` (edit/v2.rs:803) L785; `check_text_content` (edit/v2.rs:806) L787; `project_paragraph` (inspect.rs:1024) L788
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`, `W::p`
- **Called by:** `before_sha256` (edit/v2.rs:797)

#### `before_sha256`  (L797-801, pub)
`pub fn before_sha256(dom: &Dom, paragraph: NodeId) -> Result<String, EditError>`

- **Does:** Lowercase hexadecimal SHA-256 of the final logical projection of `paragraph` in `dom` (the same string [`logical_text`] returns for that pair).
- **Calls (crate):** `source_sha256` (inspect.rs:271) L798; `logical_text` (edit/v2.rs:783) L799
- **Calls (std/external/unresolved):** `.as_bytes`, `Ok`
- **Called by:** none found in src

#### `codec_error`  (L803-805)
`fn codec_error(message: impl Into<String>) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `super::err` (ambiguous: `err` (edit.rs:1401); `err` (markdown/package.rs:92)) L804
- **Called by:** `logical_text` (edit/v2.rs:783); `check_text_content` (edit/v2.rs:806)

#### `check_text_content`  (L806-836)
`fn check_text_content(dom: &Dom, container: NodeId) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L807,809; `Dom::elements` (xmllinq/mod.rs:640) L808; `codec_error` (edit/v2.rs:803) L810,813,832; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L812; `XName::local_name` (xmllinq/mod.rs:208) (by name) L817; `check_text_content` (edit/v2.rs:806) L825; `Dom::has_elements` (xmllinq/mod.rs:862) L826,831; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L829
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`, `W::name`, `W::r`
- **Macros:** `format!`, `matches!`
- **Called by:** `logical_text` (edit/v2.rs:783); `check_text_content` (edit/v2.rs:806)
