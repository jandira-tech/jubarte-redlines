# Functions and methods in `src/edit/v2.rs`

25 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `present`  (L420-422)
`fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error>`

- **Does:** Optional means absent, never explicit JSON null. _(from a code comment)_
- **Calls (std/external/unresolved):** `.map`, `T::deserialize`
- **Called by:** `validate` (cli/edit_actions.rs:289) (?)

#### `integer`  (L427-443)
`fn integer<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error>`

- **Does:** JSON Schema's integer type includes 1.0 and 1e0, unlike serde's u64. _(from a code comment)_
- **Calls (std/external/unresolved):** `.as_f64`, `.as_u64`, `.contains`, `.fract`, `Err`, `Ok`, `serde::de::Error::custom`, `serde_json::Number::deserialize`
- **Called by:** `present_integer` (edit/v2.rs:455)

#### `string_enum`  (L446-449)
`fn string_enum<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error>`

- **Does:** Serde's externally tagged unit enums also accept {"on": null}; the production schema permits only strings. _(from a code comment)_
- **Calls (std/external/unresolved):** `String::deserialize`, `T::deserialize`, `serde::de::value::StringDeserializer::::new`
- **Called by:** `present_string_enum` (edit/v2.rs:450)

#### `present_string_enum`  (L450-454)
`fn present_string_enum<'de, D: Deserializer<'de>, T: Deserialize<'de>>( d: D, ) -> Result<Option<T>, D::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `string_enum` (edit/v2.rs:446) L453
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `present_integer`  (L455-457)
`fn present_integer<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `integer` (edit/v2.rs:427) L456
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `invalid`  (L458-460)
`fn invalid(message: impl Into<String>) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L459
- **Called by:** `declared_entry_count` (admission.rs:571) (?); `check_xml_depth` (admission.rs:858) (?); `picture` (edit/images.rs:86) (?); `require` (edit/v2.rs:461); `PlanV2::from_json` (edit/v2.rs:514); `load_stories` (edit.rs:1680) (?); `check_operations` (edit.rs:2072) (?); `apply_markdown` (markdown/patch.rs:121) (?)

#### `require`  (L461-467)
`fn require(condition: bool, message: &str) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `invalid` (edit/v2.rs:458) L465
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Called by:** `Target::validate` (edit/v2.rs:478); `targets` (edit/v2.rs:485); `occurrence` (edit/v2.rs:492); `attachment` (edit/v2.rs:495); `PlanV2::validate` (edit/v2.rs:523); `Operation::validate` (edit/v2.rs:600)

#### `hash`  (L468-473)
`fn hash(value: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.contains`, `.is_ascii_digit`, `.len`
- **Called by:** `PlanV2::validate` (edit/v2.rs:523); `Operation::validate` (edit/v2.rs:600)

#### `unique`  (L474-476)
`fn unique<T: Ord>(values: &[T]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.len`
- **Called by:** `run_debug_diff` (bin/jubarte.rs:3871) (?); `targets` (edit/v2.rs:485); `PlanV2::validate` (edit/v2.rs:523); `Transaction::text_selector` (edit.rs:4945) (?)

#### `Target::validate`  (L478-483)
`fn validate(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:461) L479
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `targets`  (L485-491)
`fn targets(values: &[Target]) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:461) L486; `unique` (edit/v2.rs:474) L486; `.validate` (ambiguous, 4 candidates) L488
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `PlanV2::validate` (edit/v2.rs:523)

#### `occurrence`  (L492-494)
`fn occurrence(value: Option<u64>) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:461) L493
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Operation::validate` (edit/v2.rs:600); `SourceModel::operation` (edit/v2_apply.rs:1388) (?); `resolve_operation` (edit/v2_apply.rs:2084) (?)

#### `attachment`  (L495-507)
`fn attachment(text: &str, comment: Option<&AttachedComment>) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `require` (edit/v2.rs:461) L497,501
- **Calls (std/external/unresolved):** `.is_empty`, `Ok`
- **Called by:** `Operation::validate` (edit/v2.rs:600); `SourceModel::operation` (edit/v2_apply.rs:1388) (?)

#### `PlanV2::from_json`  (L514-519, pub)
`pub fn from_json(json: &str) -> Result<Self, EditError>`

- **Does:** Parse strictly with stable [`EditError`] codes: `INVALID_PLAN` for malformed JSON, unknown/missing fields, invalid values or incompatible field combinations; `UNSUPPORTED_SCHEMA` for another integer version.
- **Calls (crate):** `without_byte_order_mark` (edit.rs:1966) L515; `invalid` (edit/v2.rs:458) L516
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `.validate`, `Ok`, `serde_json::from_str`
- **Called by:** `request` (bin/jubarte/edit_typed.rs:294); `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (edit/prepared.rs:1292); `tests::effectful_apply_never_reopens_its_primary_for_the_correspondence` (edit/v2_apply.rs:2745); `tests::final_views_read_off_the_dom_match_a_reparse_of_the_emitted_primary` (edit/v2_apply.rs:2801)

#### `PlanV2::validate`  (L523-574, pub)
`pub fn validate(&self) -> Result<(), EditError>`

- **Does:** Validate the production request contract after constructing/modifying a value.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L525; `require` (edit/v2.rs:461) L534,538,542,549,555,559; `hash` (edit/v2.rs:468) L535; `is_author_whitespace` (edit/v2.rs:714) L539; `timestamp` (edit/v2.rs:723) L543; `targets` (edit/v2.rs:485) L546,547,550; `.validate` (ambiguous, 4 candidates) L554,568; `unique` (edit/v2.rs:474) L562
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.enumerate`, `.is_empty`, `.is_some`, `.iter`, `.map_err`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `compile` (edit/inline.rs:38); `generate` (edit/template.rs:84); `apply_bound` (edit/v2_apply.rs:121)

#### `PlanV2::to_json`  (L578-580, pub)
`pub fn to_json(&self) -> String`

- **Does:** Serialize without changing desired characters, attribution or array order.
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** none found in src

#### `Operation::target`  (L585-599, pub)
`pub fn target(&self) -> &Target`

- **Does:** Original source reference shared by every operation kind.
- **Calls:** none
- **Called by:** `parse_subcommand_json` (cli.rs:3779) (?); `Operation::validate` (edit/v2.rs:600); `with_operation_targets` (edit/v2_apply.rs:97) (?); `compose_threads` (edit/v2_apply.rs:1821) (?); `Transaction::resolve_thread` (edit.rs:4379) (?); `Transaction::check_thread_ops` (edit.rs:5715) (?); `run` (update.rs:225) (?)

#### `Operation::validate`  (L600-708)
`fn validate(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `.validate` (ambiguous, 4 candidates) L601,697; `Operation::target` (edit/v2.rs:585) L601; `check_payload_text` (edit/v2.rs:812) L606; `require` (edit/v2.rs:461) L616,629,638,650,651,658...; `hash` (edit/v2.rs:468) L617; `attachment` (edit/v2.rs:495) L620,631,668,674; `occurrence` (edit/v2.rs:492) L630,639,667,695
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `Ok`, `usize::from`
- **Called by:** none found in src

#### `is_author_whitespace`  (L714-716)
`fn is_author_whitespace(c: char) -> bool`

- **Does:** Engine-independent whitespace class for the author field: the union of Rust's `char::is_whitespace` (Unicode White_Space) and U+FEFF. _(from a code comment)_
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `PlanV2::validate` (edit/v2.rs:523)

#### `timestamp`  (L723-781)
`fn timestamp(value: &str) -> bool`

- **Does:** Calendar/time/offset validation for the schema's uppercase RFC 3339 shape. _(from a code comment)_
- **Calls (crate):** `number` (ambiguous: `deep_lcs_source_conservation_tests::number` (comparer/lcs.rs:15436); `number` (ids.rs:211)) L741,742,743,744,745,746
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_bytes`, `.bytes`, `.contains`, `.filter`, `.get`, `.is_ascii_digit`, `.iter`, `.len`, `.ok`, `.parse`
- **Called by:** `PlanV2::validate` (edit/v2.rs:523)

#### `logical_text`  (L802-808, pub)
`pub fn logical_text(dom: &Dom, paragraph: NodeId) -> Result<String, EditError>`

- **Does:** Exact logical text codec v1 for a supported `w:p` in an admitted DOM.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L803; `codec_error` (edit/v2.rs:844) L804; `check_text_content` (edit/v2.rs:847) L806; `project_paragraph` (inspect.rs:1265) L807
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`, `W::p`
- **Called by:** `make_row` (edit/for_edit.rs:1293); `context_of` (edit/for_edit.rs:1505); `before_sha256` (edit/v2.rs:838); `SourceModel::check_context` (edit/v2_apply.rs:1356); `resolve` (edit/v2_apply.rs:1656); `resolve_operation` (edit/v2_apply.rs:2084)

#### `check_payload_text`  (L812-830, pub(super))
`pub(super) fn check_payload_text(text: &str, from_file: bool) -> Result<(), EditError>`

- **Does:** Validate exact typed payloads without normalization.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L814,821
- **Calls (std/external/unresolved):** `.chars`, `.position`, `.starts_with`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `compile` (edit/inline.rs:38); `Operation::validate` (edit/v2.rs:600)

#### `before_sha256`  (L838-842, pub)
`pub fn before_sha256(dom: &Dom, paragraph: NodeId) -> Result<String, EditError>`

- **Does:** Lowercase hexadecimal SHA-256 of the final logical projection of `paragraph` in `dom` (the same string [`logical_text`] returns for that pair).
- **Calls (crate):** `source_sha256` (inspect.rs:299) L839; `logical_text` (edit/v2.rs:802) L840
- **Calls (std/external/unresolved):** `.as_bytes`, `Ok`
- **Called by:** none found in src

#### `codec_error`  (L844-846)
`fn codec_error(message: impl Into<String>) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L845
- **Called by:** `logical_text` (edit/v2.rs:802); `check_text_content` (edit/v2.rs:847)

#### `check_text_content`  (L847-877)
`fn check_text_content(dom: &Dom, container: NodeId) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L848,850; `Dom::elements` (xmllinq/mod.rs:640) L849; `codec_error` (edit/v2.rs:844) L851,854,873; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L853; `XName::local_name` (xmllinq/mod.rs:208) (by name) L858; `check_text_content` (edit/v2.rs:847) L866; `Dom::has_elements` (xmllinq/mod.rs:862) L867,872; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L870
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`, `W::name`, `W::r`
- **Macros:** `format!`, `matches!`
- **Called by:** `logical_text` (edit/v2.rs:802); `check_text_content` (edit/v2.rs:847)
