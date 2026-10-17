# Functions and methods in `src/edit.rs`

132 non-test, 37 test. Part of the [symbol manifest](../README.md).

#### `ResolvedRevisions::is_empty`  (L142-144, pub)
`pub fn is_empty(&self) -> bool`

- **Does:** Whether no change was accepted or rejected.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `is_pure_resolution` (bin/jubarte.rs:1626); `EditReport::to_jsonl` (edit.rs:1029)

#### `default_toc_levels`  (L610-612)
`fn default_toc_levels() -> u8`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `is_default_margins`  (L614-616)
`fn is_default_margins(margins: &Margins) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Margins::default`
- **Called by:** none found in src

#### `resolve_done`  (L618-620)
`fn resolve_done() -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `default_true`  (L622-624)
`fn default_true() -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `ListKind::format`  (L642-649)
`fn format(self) -> crate::markdown::xml::ListFormat`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `emit` (edit/tracked.rs:275) (?); `Transaction::resolve_one` (edit.rs:2528) (?); `Transaction::apply` (edit.rs:4489) (?); `build_paragraph` (edit.rs:6280) (?); `raster_extension` (markdown/from_docx/media.rs:77) (?); `read_picture` (markdown/write.rs:5616) (?)

#### `ListKind::name`  (L651-657)
`fn name(self) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<LineSpacing as Serialize>::serialize`  (L666-668, impl Serialize)
`fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.serialize_f64`, `f64::from`
- **Called by:** `serialize_timeout` (cli.rs:2460) (?)

#### `<LineSpacing as Deserialize>::deserialize`  (L672-680, impl Deserialize)
`fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.round`, `Err`, `Ok`, `Self`, `f64::deserialize`, `serde::de::Error::custom`
- **Macros:** `format!`
- **Called by:** none found in src

#### `<Points as Serialize>::serialize`  (L689-691, impl Serialize)
`fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.serialize_f64`, `f64::from`
- **Called by:** `serialize_timeout` (cli.rs:2460) (?)

#### `<Points as Deserialize>::deserialize`  (L695-703, impl Deserialize)
`fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.round`, `Err`, `Ok`, `Self`, `f64::deserialize`, `serde::de::Error::custom`
- **Macros:** `format!`
- **Called by:** none found in src

#### `Alignment::jc`  (L730-737)
`fn jc(self) -> &'static str`

- **Does:** The `w:jc` value Word writes.
- **Calls:** none
- **Called by:** `Transaction::resolve_one` (edit.rs:2528) (?)

#### `<HalfPoints as Serialize>::serialize`  (L780-782, impl Serialize)
`fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.serialize_f64`, `f64::from`
- **Called by:** `serialize_timeout` (cli.rs:2460) (?)

#### `<HalfPoints as Deserialize>::deserialize`  (L786-794, impl Deserialize)
`fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.round`, `Err`, `Ok`, `Self`, `f64::deserialize`, `serde::de::Error::custom`
- **Macros:** `format!`
- **Called by:** `deeper_boundary_tests::half_points_finite_range_and_rounding_boundaries` (edit.rs:6770)

#### `RunSpec::format`  (L891-899, pub)
`pub fn format(&self) -> RunFormat`

- **Does:** The run's formatting toggles.
- **Calls (std/external/unresolved):** `.clone`, `RunFormat::default`
- **Called by:** `emit` (edit/tracked.rs:275) (?); `Transaction::resolve_one` (edit.rs:2528) (?); `Transaction::apply` (edit.rs:4489) (?); `build_paragraph` (edit.rs:6280) (?); `raster_extension` (markdown/from_docx/media.rs:77) (?); `read_picture` (markdown/write.rs:5616) (?)

#### `RevisionCounts::from_changes`  (L959-972, pub)
`pub fn from_changes<'a>(changes: impl IntoIterator<Item = &'a Change>) -> Self`

- **Does:** The counts of `changes`, one per listed change.
- **Calls (std/external/unresolved):** `Self::default`
- **Called by:** `revision_counts` (edit/tracked.rs:237); `revision_counts` (edit.rs:1755)

#### `EditReport::to_jsonl`  (L1029-1107, pub)
`pub fn to_jsonl(&self) -> String`

- **Does:** JSON lines an agent can append to its own log: `load`, one `op` per operation, `summary`.
- **Calls (crate):** `ResolvedRevisions::is_empty` (edit.rs:142) L1045
- **Calls (std/external/unresolved):** `.as_object_mut`, `.clone`, `.count`, `.enumerate`, `.expect`, `.filter`, `.insert`, `.into`, `.is_empty`, `.iter`, `.push_str`, `.push`, `.to_string`, `String::new`, `push`, `serde_json::to_value`
- **Macros:** `serde_json::json!`
- **Called by:** `run_edit` (bin/jubarte.rs:953) (?); `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (edit.rs:6716) (?)

#### `<EditError as fmt::Display>::fmt`  (L1138-1143, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `open_error`  (L1150-1157)
`fn open_error(error: crate::inspect::InspectError) -> EditError`

- **Does:** An unopenable source: admission refusals keep their own code (`INPUT_LIMIT`, …), anything else is `INVALID_DOCUMENT`.
- **Calls (crate):** `err` (edit.rs:1401) L1153,1155; `.code` (ambiguous, 4 candidates) L1153
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** none found in src

#### `resolve_selected`  (L1161-1231)
`fn resolve_selected( source: &[u8], selection: Option<&ResolveRevisions>, ) -> Result<(Option<Vec<u8>>, ResolvedRevisions), EditError>`

- **Does:** Accept, then reject, the changes `resolve_revisions` selects: the new source (`None` when it selects nothing) and what was resolved.
- **Calls (crate):** `err` (edit.rs:1401) L1170,1175,1176,1181,1204; `list_changes` (changes.rs:287) L1183,1220; `ChangeFilter::normalized` (changes.rs:130) (by name) L1189; `ChangeFilter::check` (changes.rs:159) (by name) L1190; `ChangeFilter::matches` (changes.rs:177) (by name) L1191; `select` (audit.rs:239) (by name) L1201,1202; `accept_changes` (changes.rs:335) L1215; `ChangeFilter::ids` (changes.rs:117) L1215,1226; `reject_changes` (changes.rs:345) L1226
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map_err`, `.map`, `.to_vec`, `Err`, `Ok`, `ResolvedRevisions::default`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `revisions_left` (edit.rs:1238); `settle` (edit.rs:1260); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:6789)

#### `revisions_left`  (L1238-1244, pub)
`pub fn revisions_left(source: &[u8], plan: &EditPlan) -> bool`

- **Does:** Whether `source` still carries a tracked change of its text (body, headers, footers, notes; a style's recorded formatting is not one) once `plan`'s `resolve_revisions` has accepted and rejected what it selects.
- **Calls (crate):** `resolve_selected` (edit.rs:1161) L1239; `list_changes` (changes.rs:287) L1242
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.is_ok_and`, `.iter`, `.ok`, `.starts_with`, `.unwrap_or`
- **Called by:** `editing_mode_conflict` (edit/flags.rs:544)

#### `settle`  (L1260-1297)
`fn settle(source: &[u8], plan: &EditPlan, probe: Opened) -> Result<Settled, EditError>`

- **Does:** Settle `source` (opened as `probe`) as `plan` asks: accept and reject what `resolve_revisions` selects, then apply `existing_revisions` to the changes left.
- **Calls (crate):** `resolve_selected` (edit.rs:1161) L1262; `Opened::open` (inspect.rs:525) L1264,1287; `changes_in` (changes.rs:302) L1269; `err` (edit.rs:1401) L1275,1286; `accept_revisions` (document_comparer.rs:4959) L1283; `reject_revisions` (document_comparer.rs:4976) L1284
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.as_slice`, `.is_some`, `.iter`, `.map_err`, `.starts_with`, `.to_string`, `.to_vec`, `Err`, `Ok`
- **Called by:** `Origin::build` (edit/origin.rs:256); `Transaction::start` (edit.rs:2004); `Select::parse` (markdown/mod.rs:453) (?)

#### `load_stories`  (L1311-1355)
`fn load_stories(opened: &mut Opened) -> Result<Loaded, EditError>`

- **Does:** The body and every header, footer and notes part of `opened`, parsed into its DOM, with every addressable paragraph.
- **Calls (crate):** `Opened::story_parts` (inspect.rs:641) L1318; `err` (edit.rs:1401) L1319; `PartFs::part_string` (opc/mod.rs:498) (by name) L1320; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:447); `invalid` (markdown/from_docx/ooxml.rs:27)) L1323,1324,1329; `validate_xml` (xmllinq/parse.rs:792) L1324; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1325; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L1326; `body_paragraph_nodes` (inspect.rs:747) L1341; `story_paragraph_nodes` (inspect.rs:937) L1343
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.into_iter`, `.into`, `.iter`, `.map_err`, `.ok_or_else`, `.push`, `.to_string`, `Ok`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `Origin::build` (edit/origin.rs:256); `paragraph_ids` (edit.rs:1365); `Transaction::start` (edit.rs:2004)

#### `paragraph_ids`  (L1365-1399, pub)
`pub fn paragraph_ids(docx: &[u8], ids: &[&str]) -> Result<Vec<Result<String, String>>, EditError>`

- **Does:** The paragraph id ([`crate::ids::paragraph_id`]) each of `ids` names in `docx`, as `edit -p` reads it on a document without tracked changes to settle: `p3` and `header1.p1` as given, `header1.p0` → `header1`, `t0.row1.cell2[.p1]` → that cell's paragraph (`p40`).
- **Calls (crate):** `Opened::open` (inspect.rs:525) L1366; `load_stories` (edit.rs:1311) L1371; `Ids::paragraph_id` (edit/origin.rs:48) L1381
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.is_ok`, `.iter`, `.join`, `.map_err`, `.map`, `.parse`, `.rsplit_once`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** none found in src

#### `err`  (L1401-1408)
`fn err(code: &str, operation: Option<&str>, message: impl Into<String>) -> EditError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.map`, `.to_string`, `Vec::new`
- **Called by:** `v2_select` (edit/anchor.rs:182); `v2_select_all` (edit/anchor.rs:238); `Transaction::resolve_fill_control` (edit/controls.rs:72); `Transaction::footnotes_story` (edit/notes.rs:51) (?); `Origin::build` (edit/origin.rs:256); `record_mid_changes` (edit/sections.rs:453); `invalid` (edit/v2.rs:447) (?); `PlanV2::validate` (edit/v2.rs:511) (?); `codec_error` (edit/v2.rs:803) (?); `Transaction::resolve_watermark` (edit/watermark.rs:174); `Transaction::mark_header` (edit/watermark.rs:284); `rewrite` (edit/whole.rs:130); +21 more

#### `EditPlan::from_json`  (L1418-1435, pub)
`pub fn from_json(json: &str) -> Result<Self, EditError>`

- **Does:** Parse and validate a plan's JSON.
- **Calls (crate):** `err` (edit.rs:1401) L1420,1423,1425; `check_operation_keys` (edit.rs:1521) L1421
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Err`, `Ok`, `serde_json::from_str`, `serde_json::from_value`
- **Macros:** `format!`
- **Called by:** `edit_plan` (bin/jubarte.rs:927); `control_source_boundary_tests::plan` (edit/controls.rs:824); `editing_mode_conflict_json` (edit/flags.rs:554); `tests::the_json_entry_point_reads_what_the_cli_schema_serializes` (edit/flags.rs:995); `tests::record_json_reads_the_options_the_bindings_pass` (edit/record.rs:490); `apply_plan_json` (edit.rs:1594); `tests::plan_json_roundtrips_and_rejects_unknown_fields` (edit.rs:6556); `deeper_boundary_tests::plan` (edit.rs:6675)

#### `EditPlan::to_json`  (L1438-1440, pub)
`pub fn to_json(&self) -> String`

- **Does:** Serialize the plan.
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** none found in src

#### `operation_schemas`  (L1446-1474, pub)
`pub fn operation_schemas() -> &'static [serde_json::Value]`

- **Does:** The legacy dispatcher contract, derived from its typed Serde operations.
- **Calls (std/external/unresolved):** `.as_array`, `.clone`, `.expect`, `.for_deserialize`, `.into_generator`, `.into_root_schema_for`, `.with`, `schemars::generate::SchemaSettings::draft2020_12`, `serde_json::to_value`, `std::sync::LazyLock::new`
- **Macros:** `serde_json::json!`
- **Called by:** `profiles` (capabilities/discovery.rs:37); `tests::assert_runtime_dispatch` (capabilities/discovery.rs:324); `dispatched_operation_kinds` (edit.rs:1477); `operation_supports_keep` (edit.rs:1489); `check_operation_keys` (edit.rs:1521)

#### `dispatched_operation_kinds`  (L1477-1487, pub)
`pub fn dispatched_operation_kinds() -> Vec<String>`

- **Does:** Names accepted by the common v1 dispatcher, in the legacy manifest order.
- **Calls (crate):** `operation_schemas` (edit.rs:1446) L1478
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.expect`, `.iter`, `.map`, `.to_string`
- **Called by:** `discovery` (capabilities/discovery.rs:193); `capabilities` (capabilities.rs:355)

#### `operation_supports_keep`  (L1489-1494, pub(crate))
`pub(crate) fn operation_supports_keep(kind: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `operation_schemas` (edit.rs:1446) L1490
- **Calls (std/external/unresolved):** `.find`, `.is_some_and`, `.iter`
- **Called by:** `support` (capabilities/discovery.rs:114); `untracked_kind` (edit/tracked.rs:53)

#### `at_least_one`  (L1496-1503)
`fn at_least_one(fields: &[&str]) -> serde_json::Value`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.map`
- **Macros:** `json!`, `serde_json::json!`
- **Called by:** none found in src

#### `exactly_one`  (L1505-1517)
`fn exactly_one(fields: &[&str]) -> serde_json::Value`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.filter`, `.iter`, `.map`
- **Macros:** `json!`, `serde_json::json!`
- **Called by:** none found in src

#### `check_operation_keys`  (L1521-1586)
`fn check_operation_keys(plan: &serde_json::Value) -> Result<(), EditError>`

- **Does:** Allowed keys per operation kind; serde's `flatten` cannot deny unknown fields itself, and a misspelled key must not silently change meaning.
- **Calls (crate):** `err` (edit.rs:1401) L1528,1539,1555,1562,1569,1577; `operation_schemas` (edit.rs:1446) L1535
- **Calls (std/external/unresolved):** `.and_then`, `.as_array`, `.as_object`, `.as_str`, `.contains_key`, `.contains`, `.enumerate`, `.expect`, `.filter`, `.find`, `.get`, `.iter`, `.keys`, `.unwrap_or`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `EditPlan::from_json` (edit.rs:1418)

#### `apply_plan_json`  (L1594-1596, pub)
`pub fn apply_plan_json(source: &[u8], plan_json: &str) -> Result<EditResult, EditError>`

- **Does:** Apply a plan given as JSON: [`EditPlan::from_json`], then [`apply_plan`].
- **Calls (crate):** `apply_plan` (edit.rs:1621) L1595; `EditPlan::from_json` (edit.rs:1418) L1595
- **Called by:** `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:1422)

#### `apply_plan`  (L1621-1709, pub)
`pub fn apply_plan(source: &[u8], plan: &EditPlan) -> Result<EditResult, EditError>`

- **Does:** Resolve and apply the plan; compare source and copy into a redline.
- **Calls (crate):** `check_update_fields` (edit.rs:1714) L1622; `Transaction::start` (edit.rs:2004) L1623; `Transaction::resolve` (edit.rs:2375) L1624; `check` (edit/tracked.rs:84) L1625; `Transaction::emits_directly` (edit.rs:2125) L1626; `Transaction::apply` (edit.rs:4489) L1627; `Transaction::finish` (edit.rs:5077) L1628; `Transaction::commented_base` (edit.rs:4985) (by name) L1630; `result` (edit/tracked.rs:167) L1632; `Transaction::check_redactions` (edit.rs:4453) L1633,1697; `update_fields` (fields.rs:115) L1639; `err` (edit.rs:1401) L1640,1658,1695; `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1475) L1652; `Transaction::commented_base` (edit.rs:4985) L1655; `compare_documents_with_settings` (document_comparer.rs:6414) L1657; `record_mid_changes` (edit/sections.rs:453) L1664; `rewrite` (edit/whole.rs:130) L1667; `comment_holds_kept_text` (edit.rs:5236) L1682; `Transaction::conflict` (edit.rs:4477) L1684; `apply_settings_to_docx` (settings.rs:424) L1694; `Transaction::report` (edit.rs:2346) L1698; `paragraphs` (inspect.rs:284) L1699; `revision_counts` (edit.rs:1755) L1702
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.find`, `.into`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::new`, `refresh`, `std::borrow::Cow::Borrowed`
- **Macros:** `format!`, `matches!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `control_source_boundary_tests::public_block_controls_refuse_section_loss_and_both_nested_fill_orders` (edit/controls.rs:882); `apply_plan_json` (edit.rs:1594); `preview_plan` (edit.rs:1742); `apply_markdown` (markdown/patch.rs:121)

#### `check_update_fields`  (L1714-1723)
`fn check_update_fields(plan: &EditPlan) -> Result<(), EditError>`

- **Does:** `update_fields` refreshes the clean copy the comparer reads.
- **Calls (crate):** `err` (edit.rs:1401) L1716
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Called by:** `apply_plan` (edit.rs:1621); `resolved` (edit.rs:1732)

#### `check_plan`  (L1728-1730, pub)
`pub fn check_plan(source: &[u8], plan: &EditPlan) -> Result<(), EditError>`

- **Does:** Resolve every operation and run the plan's checks, without applying it: what `add --plan` runs on each recorded operation, so recording a long plan never re-applies it.
- **Calls (crate):** `resolved` (edit.rs:1732) L1729
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `record_plan` (bin/jubarte.rs:1887)

#### `resolved`  (L1732-1739)
`fn resolved<'a>(source: &'a [u8], plan: &'a EditPlan) -> Result<Transaction<'a>, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `check_update_fields` (edit.rs:1714) L1733; `Transaction::start` (edit.rs:2004) L1734; `Transaction::resolve` (edit.rs:2375) L1735; `check` (edit/tracked.rs:84) L1736; `Transaction::emits_directly` (edit.rs:2125) L1737
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `check_plan` (edit.rs:1728); `preview_plan` (edit.rs:1742)

#### `preview_plan`  (L1742-1752, pub)
`pub fn preview_plan(source: &[u8], plan: &EditPlan) -> Result<EditReport, EditError>`

- **Does:** Resolve every operation without producing documents.
- **Calls (crate):** `resolved` (edit.rs:1732) L1743; `Transaction::report` (edit.rs:2346) (by name) L1744; `apply_plan` (edit.rs:1621) L1747
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `control_source_boundary_tests::public_block_controls_refuse_section_loss_and_both_nested_fill_orders` (edit/controls.rs:882); `deeper_boundary_tests::admission_boundaries_and_signature_prefix_are_exact` (edit.rs:6828); `deeper_boundary_tests::resolution_empty_text_and_combined_comment_anchors` (edit.rs:6877); `deeper_boundary_tests::header_refuses_body_only_picture_toc_and_list` (edit.rs:6938)

#### `revision_counts`  (L1755-1757)
`fn revision_counts(redline: &[u8]) -> RevisionCounts`

- **Does:** The comparer's redline holds this edit's changes only: all of them.
- **Calls (crate):** `RevisionCounts::from_changes` (edit.rs:959) L1756; `list_changes` (changes.rs:287) L1756
- **Calls (std/external/unresolved):** `.unwrap_or_default`
- **Called by:** `apply_plan` (edit.rs:1621); `deeper_boundary_tests::invalid_package_has_no_revisions_or_kept_comment_text` (edit.rs:6870); `deeper_boundary_tests::revision_and_comment_scans_ignore_rootless_and_foreign_story_parts` (edit.rs:8016)

#### `ThreadOp::target`  (L1906-1913)
`fn target(&self) -> u32`

- **Does:** The existing comment the operation acts on.
- **Calls:** none
- **Called by:** `parse_json` (cli.rs:3053) (?); `Transaction::resolve_thread` (edit.rs:3342) (?); `Transaction::check_thread_ops` (edit.rs:4326) (?); `run` (update.rs:186) (?)

#### `Transaction::start`  (L2004-2117)
`fn start(source: &[u8], plan: &'p EditPlan) -> Result<Self, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1401) L2006,2016,2023,2032,2083; `source_sha256` (inspect.rs:271) L2012,2045; `Opened::open` (inspect.rs:525) L2025; `PartFs::parts` (opc/mod.rs:532) (by name) L2026; `settle` (edit.rs:1260) L2044; `load_stories` (edit.rs:1311) L2050; `Origin::build` (edit/origin.rs:256) L2055; `project_paragraph` (inspect.rs:1024) L2066; `existing_comment_ids` (edit.rs:5264) L2079; `body_control_nodes` (inspect.rs:756) L2080; `collect_controls` (inspect.rs:764) L2081; `CommentFamily::load` (comments.rs:429) L2082
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.clone`, `.collect`, `.ends_with`, `.filter_map`, `.is_empty`, `.iter`, `.map_err`, `.map_or`, `.map`, `.next`, `.split_whitespace`, `.starts_with`, `.to_uppercase`, `.trim`, `.unwrap_or_else`, `BTreeMap::new`, `Err`, `Ok`, `Some`, `Vec::new`, `std::cell::Cell::new`, `std::collections::BTreeSet::new`, `u64::from`
- **Macros:** `format!`
- **Called by:** `redline` (edit/tracked.rs:185); `apply_plan` (edit.rs:1621); `resolved` (edit.rs:1732); `Transaction::commented_base` (edit.rs:4985); `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (edit.rs:6716); `deeper_boundary_tests::resolution_empty_text_and_combined_comment_anchors` (edit.rs:6877); `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (edit.rs:6974); `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (edit.rs:7001); `deeper_boundary_tests::conflicts` (edit.rs:7056); `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (edit.rs:7466); `deeper_boundary_tests::whole_deletion_builds_no_bookmark_and_keeps_surrounding_text` (edit.rs:7999)

#### `Transaction::emits_directly`  (L2125-2135)
`fn emits_directly(&self) -> Result<bool, EditError>`

- **Does:** Whether the redline is emitted directly, as under keep: the comparer moves a comment's range off text an edit replaces whole, so a plan that does is emitted, unless it holds an operation the emitter cannot track (a footnote, an image).
- **Calls (crate):** `Transaction::edits_a_comment_range` (edit.rs:2139) L2126; `emits_all` (edit/tracked.rs:59) L2126; `err` (edit.rs:1401) L2128
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Called by:** `apply_plan` (edit.rs:1621); `resolved` (edit.rs:1732)

#### `Transaction::edits_a_comment_range`  (L2139-2158)
`fn edits_a_comment_range(&self) -> bool`

- **Does:** Whether the plan names a comment's text (`cN.text`), or one of its text edits replaces exactly the text a comment covers with new text.
- **Calls (crate):** `selector_of` (edit.rs:5334) L2144; `Transaction::comment_ranges` (edit.rs:2162) L2155
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.contains`, `.is_empty`, `.is_some`, `.iter`
- **Called by:** `Transaction::emits_directly` (edit.rs:2125)

#### `Transaction::comment_ranges`  (L2162-2173)
`fn comment_ranges(&self, para: usize) -> Vec<(usize, usize)>`

- **Does:** The `(start, end)` projection offsets of each comment whose range opens and closes in paragraph `para`.
- **Calls (crate):** `Transaction::comment_marks` (edit.rs:2177) L2165
- **Calls (std/external/unresolved):** `.iter`, `.position`, `.push`, `.remove`, `Vec::new`
- **Called by:** `Transaction::edits_a_comment_range` (edit.rs:2139)

#### `Transaction::comment_marks`  (L2177-2206)
`fn comment_marks(&self, para: usize) -> Vec<(String, bool, usize)>`

- **Does:** Each comment range marker of paragraph `para`, in document order: its `w:id`, whether it opens the range, and its projection offset.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L2182; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2193; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2196; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L2201
- **Calls (std/external/unresolved):** `.filter`, `.find_map`, `.iter`, `.name`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::id`
- **Called by:** `Transaction::comment_ranges` (edit.rs:2162); `Transaction::comment_text` (edit.rs:2210)

#### `Transaction::comment_text`  (L2210-2237)
`fn comment_text(&self, n: u32) -> Result<Covered, origin::Miss>`

- **Does:** The text comment `n` covers (`cN.text`), from its range start to its range end.
- **Calls (crate):** `Transaction::comment_marks` (edit.rs:2177) L2214
- **Calls (std/external/unresolved):** `.into`, `.len`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_comment_text` (edit.rs:2242); `Transaction::resolve` (edit.rs:2375); `Transaction::select` (edit.rs:3778)

#### `Transaction::resolve_comment_text`  (L2242-2344)
`fn resolve_comment_text( &self, n: u32, new_text: &str, outcome: &mut EditOutcome, ) -> Result<Vec<Resolved>, (String, String)>`

- **Does:** `cN.text` rewritten as `new_text` (empty: deleted): one tracked substitution inside the comment.
- **Calls (crate):** `Transaction::comment_text` (edit.rs:2210) L2248; `Transaction::paragraph_id` (edit.rs:3876) L2258; `short_paragraph` (ids.rs:435) L2264,2298,2299; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L2264,2298,2299; `check_run_text` (edit.rs:5381) L2268; `Transaction::check_range` (edit.rs:3923) L2273,2303,2305; `context` (edit.rs:5421) L2275,2307; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L2294; `structure` (markdown/from_docx/mod.rs:4190) (by name) L2296
- **Calls (std/external/unresolved):** `.into`, `.len`, `.map_err`, `.push`, `.to_string`, `Err`, `Ok`, `Some`, `String::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `Transaction::resolve_rewrite` (edit.rs:3466)

#### `Transaction::report`  (L2346-2371)
`fn report(&self, ok: bool) -> EditReport`

- **Does:** _no doc comment_
- **Calls (crate):** `Origin::moves` (edit/origin.rs:438) (by name) L2360; `Transaction::paragraph_id` (edit.rs:3876) L2360; `Transaction::body_paragraph_count` (edit.rs:3845) L2363,2364
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.is_some`, `.map`, `.unwrap_or_default`, `RevisionCounts::default`, `Vec::new`
- **Called by:** `result` (edit/tracked.rs:167); `apply_plan` (edit.rs:1621); `preview_plan` (edit.rs:1742) (?); `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (edit.rs:6716) (?)

#### `Transaction::resolve`  (L2375-2526)
`fn resolve(&mut self) -> Result<(), EditError>`

- **Does:** Resolve every operation against the untouched source; the first failure is returned after every operation has been tried.
- **Calls (crate):** `kind_name` (edit.rs:5294) L2379; `selector_of` (edit.rs:5334) L2380; `Transaction::comment_text` (edit.rs:2210) L2383; `Transaction::resolve_rewrite` (edit.rs:3466) L2391,2398; `comment_text_of` (edit.rs:5326) L2397; `Transaction::resolve_list` (edit.rs:3563) L2405; `Transaction::resolve_page_setup` (edit/sections.rs:363) L2427; `err` (edit.rs:1401) L2439; `Transaction::resolve_thread` (edit.rs:3342) L2446; `Transaction::resolve_watermark` (edit/watermark.rs:174) L2454; `Transaction::resolve_fill_control` (edit/controls.rs:72) L2466; `Transaction::resolve_settings` (edit.rs:4399) L2486; `Transaction::resolve_one` (edit.rs:2528) L2491; `Transaction::check_conflicts` (edit.rs:4044) L2524
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.enumerate`, `.extend`, `.filter`, `.get_or_insert`, `.into_iter`, `.into`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.push`, `.set`, `.to_string`, `.unwrap_or_else`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `tests::resolve_keeps_quoted_css_list_intact` (convert/font.rs:4187) (?); `tests::resolve_quoted_cambria_still_finds_cambria` (convert/font.rs:4208) (?); `tests::resolve_dejavu_sans_mono_follows_word_subst_verdana` (convert/font.rs:4221) (?); `tests::resolve_unknown_family_is_cambria_not_calibri` (convert/font.rs:4231) (?); `tests::resolve_wide_latin_stays_calibri_mini_505` (convert/font.rs:4262) (?); `tests::resolve_empty_family_is_times_new_roman` (convert/font.rs:4272) (?); `tests::resolve_prefers_embedded_unknown_family` (convert/font.rs:4798) (?); `tests::resolve_embedded_does_not_steal_unrelated_families` (convert/font.rs:4838) (?); `tests::font_table_scope_is_restored_after_a_panicking_conversion` (convert/font.rs:5231) (?); `tests::nested_font_table_scope_restores_outer_table` (convert/font.rs:5328) (?); `chrome_part_xml` (convert/mod.rs:20750) (?); `hf_pic_trails_line` (convert/mod.rs:21857) (?); +46 more

#### `Transaction::resolve_one`  (L2528-3334)
`fn resolve_one( &self, id: &str, kind: &OperationKind, ) -> Result<(Resolved, EditOutcome), Box<(EditError, EditOutcome)>>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1401) L2547; `Transaction::select` (edit.rs:3778) L2611,2864,3096; `finds_text` (edit.rs:5355) L2611; `Transaction::paragraph_id` (edit.rs:3876) L2618; `comment_text_of` (edit.rs:5326) L2619; `short_paragraph` (ids.rs:435) L2634; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L2634,2876,2884,2885; `check_run_text` (edit.rs:5381) L2694,2783,3105; `check_format` (edit.rs:6434) L2697,2785,3107; `check_comment` (edit.rs:5408) L2701,2789,2855,2902,2940,3111; `Transaction::find_range` (edit.rs:3901) L2703,2730,2751,2800,2806; `context` (edit.rs:5421) L2706,2734,2754,2835,2921,2981...; `hide` (markdown/anchor.rs:26) L2732; `redaction` (edit.rs:5417) L2733; `Transaction::check_insert_position` (edit.rs:4021) L2833; `locate` (edit.rs:5475) L2906; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L2944; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L2951,3236; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L2952,2961; `Transaction::story_container` (edit.rs:3883) L2960; `excerpt` (edit.rs:5435) L2969,3010,3049,3121,3180,3208...; `Transaction::resolve_format_run` (edit/runs.rs:17) L2978; `Transaction::resolve_footnote` (edit/notes.rs:24) L3003; `picture` (edit/images.rs:86) L3037; `.format` (ambiguous: `ListKind::format` (edit.rs:642); `RunSpec::format` (edit.rs:891); `Writer::format` (markdown/write.rs:2950)) L3107; `Transaction::resolve_style` (edit.rs:3716) L3115,3158; `.jc` (ambiguous: `Alignment::jc` (edit.rs:730); `Styles::jc` (markdown/from_docx/mod.rs:1800)) L3168; `check_text` (edit.rs:5368) L3194,3309; `Transaction::merge_partner` (edit.rs:3691) L3195; `check_rows` (edit/structural.rs:28) L3247; `text_width` (edit/structural.rs:70) L3252; `resolve_table_style` (edit/structural.rs:119) L3257; `table_styles` (edit/structural.rs:89) L3258
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.as_str`, `.chars`, `.clone`, `.collect`, `.contains`, `.first`, `.get`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.join`, `.len`, `.map_err`, `.map_or`, `.map`, `.push`, `.replace`, `.rev`, `.take`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Box::new`, `Err`, `Ok`, `Some`, `Spacing::default`, `String::new`, `Vec::new`, `W::p`, `W::sect_pr`, `W::tc`, `fail`, `from`, `u32::try_from`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::resolve_thread`  (L3342-3433)
`fn resolve_thread(&self, id: &str, kind: &OperationKind) -> Resolution<Resolved>`

- **Does:** The paragraph `para` merges with: the next sibling paragraph, with only range markup (bookmarks, comment ranges, ...) between them.
- **Calls (crate):** `err` (edit.rs:1401) L3357; `.target` (ambiguous: `EditOptions::target` (cli.rs:212); `Operation::target` (edit/v2.rs:573); `ThreadOp::target` (edit.rs:1906)) L3381; `Origin::had_comment` (edit/origin.rs:248) (by name) L3384; `check_comment` (edit.rs:5408) L3394; `CommentFamily::with_replies` (comments.rs:563) (by name) L3397; `Transaction::comment_markers` (edit.rs:3437) L3400; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L3402; `Transaction::paragraph_id` (edit.rs:3876) L3416
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.contains`, `.expect`, `.find_map`, `.into_iter`, `.into`, `.is_none`, `.is_some_and`, `.iter`, `.map_err`, `.map`, `.position`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::new`, `W::p`, `fail`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Transaction::resolve` (edit.rs:2375); `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (edit.rs:7466) (?)

#### `Transaction::comment_markers`  (L3437-3462)
`fn comment_markers(&self, ids: &[u32], target: u32) -> (Vec<usize>, Option<NodeId>)`

- **Does:** Stories holding markers of the comments `ids`, and the reference element of comment `target`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L3445; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3446
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.enumerate`, `.is_none`, `.iter`, `.map`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::id`, `W::name`
- **Called by:** `Transaction::resolve_thread` (edit.rs:3342); `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (edit.rs:7466) (?)

#### `Transaction::resolve_rewrite`  (L3466-3559)
`fn resolve_rewrite( &self, id: &str, paragraph: &Selector, new_text: &str, ) -> Resolution<Vec<Resolved>>`

- **Does:** `rewrite`: the smallest word-level edits that make the paragraph read as `new_text`, each checked as a `replace` or an `insert` is.
- **Calls (crate):** `err` (edit.rs:1401) L3486; `comment_text_of` (edit.rs:5326) L3488; `Transaction::resolve_comment_text` (edit.rs:2242) L3489; `Transaction::select` (edit.rs:3778) L3494; `Transaction::paragraph_id` (edit.rs:3876) L3501; `check_run_text` (edit.rs:5381) L3504; `ChangeFilter::matches` (changes.rs:177) (by name) L3507; `rewrite_ranges` (edit/rewrite.rs:72) L3519; `Transaction::check_insert_position` (edit.rs:4021) L3536,3538; `Transaction::check_range` (edit.rs:3923) L3543; `excerpt` (edit.rs:5435) L3557
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.count`, `.into`, `.is_empty`, `.is_some_and`, `.len`, `.map_err`, `.map`, `.next_back`, `.or_else`, `.push`, `.replace`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::with_capacity`, `fail`, `symbols`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::resolve_list`  (L3563-3689)
`fn resolve_list( &self, id: &str, selectors: &[Selector], kind: ListKind, level: u32, restart: bool, ) -> Resolution<Resolved>`

- **Does:** `list`: every selector matches one body paragraph, once; `restart: false` finds the list to continue.
- **Calls (crate):** `err` (edit.rs:1401) L3585; `Transaction::select` (edit.rs:3778) L3603; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L3613,3622; `Transaction::paragraph_id` (edit.rs:3876) L3633; `direct_num_id` (edit/structural.rs:422) L3643; `Transaction::paragraph_styles` (edit.rs:3735) L3657; `excerpt` (edit.rs:5435) L3676
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.eq_ignore_ascii_case`, `.find_map`, `.find`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_or`, `.map`, `.min`, `.name`, `.or_else`, `.push`, `.rev`, `.to_string`, `.unwrap_or`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::with_capacity`, `fail`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::merge_partner`  (L3691-3713)
`fn merge_partner(&self, para: usize) -> Result<usize, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::next_element` (xmllinq/mod.rs:855) (by name) L3694,3696; `is_range_markup` (edit.rs:5280) L3695; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L3699; `Dom::element` (xmllinq/mod.rs:655) (by name) L3708,3709
- **Calls (std/external/unresolved):** `.filter`, `.into`, `.is_some_and`, `.is_some`, `.iter`, `.ok_or_else`, `.position`, `.to_string`, `Err`, `Ok`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `Transaction::resolve_style`  (L3716-3732)
`fn resolve_style(&self, requested: &str) -> Result<String, String>`

- **Does:** A paragraph style id from its id or its name (case-insensitive).
- **Calls (crate):** `Transaction::paragraph_styles` (edit.rs:3735) L3717
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.eq_ignore_ascii_case`, `.find`, `.iter`, `.join`, `.map`, `.take`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `Transaction::paragraph_styles`  (L3735-3760)
`fn paragraph_styles(&self) -> Vec<(String, String)>`

- **Does:** `(styleId, name)` of every paragraph style in the styles part.
- **Calls (crate):** `Opened::related` (inspect.rs:669) L3736; `PartFs::part_string` (opc/mod.rs:498) L3739; `Dom::new` (xmllinq/mod.rs:387) L3742; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3743; `Dom::root` (xmllinq/mod.rs:822) L3744; `Dom::elements` (xmllinq/mod.rs:640) L3747; `Dom::attribute` (xmllinq/mod.rs:874) L3749,3751,3754; `Dom::element` (xmllinq/mod.rs:655) L3752
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.next`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::val`
- **Called by:** `Transaction::resolve_list` (edit.rs:3563); `Transaction::resolve_style` (edit.rs:3716); `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (edit.rs:6974) (?)

#### `Transaction::ids`  (L3763-3771)
`fn ids(&self) -> origin::Ids<'_>`

- **Does:** What this plan's own ids resolve against.
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `validate_matches` (cli.rs:2566) (?); `Transaction::cell_control` (edit/controls.rs:298) (?); `Transaction::select` (edit.rs:3778) (?)

#### `Transaction::select`  (L3778-3843)
`fn select(&self, selector: &Selector, merged_ok: bool) -> Result<usize, origin::Miss>`

- **Does:** The paragraph `selector` names.
- **Calls (crate):** `comment_text_of` (edit.rs:5326) L3779; `Transaction::comment_text` (edit.rs:2210) L3780; `.ids` (ambiguous: `ChangeFilter::ids` (changes.rs:117); `Origin::ids` (edit/origin.rs:397); `Transaction::ids` (edit.rs:3763)) L3783; `Origin::follow` (edit/origin.rs:410) (by name) L3785; `Transaction::paragraph_id` (edit.rs:3876) L3785; `Transaction::ids` (edit.rs:3763) L3788,3790; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L3836
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.position`, `.starts_with`, `Err`, `Ok`
- **Macros:** `format!`, `unreachable!`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_rewrite` (edit.rs:3466); `Transaction::resolve_list` (edit.rs:3563)

#### `Transaction::body_paragraph_count`  (L3845-3847)
`fn body_paragraph_count(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`
- **Called by:** `Transaction::report` (edit.rs:2346)

#### `Transaction::touched_stories`  (L3850-3873)
`fn touched_stories(&self) -> std::collections::BTreeSet<usize>`

- **Does:** Stories an operation of the plan edits.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.flat_map`, `.iter`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `Transaction::paragraph_id`  (L3876-3879)
`fn paragraph_id(&self, para: usize) -> String`

- **Does:** The id of a paragraph ([`crate::ids::paragraph_id`]).
- **Calls (crate):** `paragraph_id` (ids.rs:410) L3878
- **Called by:** `Package::heading_skip` (audit.rs:532) (?); `Package::literal_bullet` (audit.rs:636) (?); `Package::empty_spacers` (audit.rs:661) (?); `Transaction::cell_control` (edit/controls.rs:298) (?); `Transaction::resolve_comment_text` (edit.rs:2242); `Transaction::report` (edit.rs:2346); `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_thread` (edit.rs:3342); `Transaction::resolve_rewrite` (edit.rs:3466); `Transaction::resolve_list` (edit.rs:3563) (?); `Transaction::select` (edit.rs:3778)

#### `Transaction::story_container`  (L3883-3896)
`fn story_container(&self, node: NodeId) -> (NodeId, &'static str)`

- **Does:** The body, header, footer or note a paragraph belongs to, for the "keeps one paragraph" checks.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L3891
- **Calls (std/external/unresolved):** `.first`, `Some`, `W::name`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528); `Transaction::check_deletions_leave_valid_containers` (edit.rs:4289)

#### `Transaction::find_range`  (L3901-3918)
`fn find_range( &self, projection: &Projection, find: &str, occurrence: Option<usize>, outcome: &mut EditOutcome, ) -> Result<(usize, usize), (String, String)>`

- **Does:** The unique occurrence of `find`, or its `occurrence`-th hit (1-based) when given (overlapping occurrences count), checked to lie within editable direct text.
- **Calls (crate):** `locate` (edit.rs:5475) L3908; `Transaction::check_range` (edit.rs:3923) L3915
- **Calls (std/external/unresolved):** `.get`, `.map_err`, `.to_string`, `Ok`
- **Called by:** `Transaction::resolve_footnote` (edit/notes.rs:24); `Transaction::resolve_format_run` (edit/runs.rs:17); `Transaction::resolve_one` (edit.rs:2528)

#### `Transaction::check_range`  (L3923-3954)
`fn check_range(&self, projection: &Projection, start: usize, end: usize) -> Result<(), String>`

- **Does:** Every byte of `[start, end)` must come from a direct run: its text, or a tab, break or non-breaking hyphen, which an edit over it removes.
- **Calls (crate):** `Transaction::opaque_structure` (edit.rs:3959) (by name) L3932; `run_of` (edit.rs:6008) L3932; `is_symbol` (edit.rs:5583) L3935
- **Calls (std/external/unresolved):** `.any`, `.into`, `.iter`, `.max`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_comment_text` (edit.rs:2242); `Transaction::resolve_rewrite` (edit.rs:3466); `Transaction::find_range` (edit.rs:3901); `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (edit.rs:7001) (?)

#### `Transaction::opaque_structure`  (L3959-4019)
`fn opaque_structure(&self, run: NodeId) -> String`

- **Does:** What holds `run`, a run no edit may touch, and the way around it: a revision by id and author first (settling it frees the text), else the innermost hyperlink, field or content control.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L3961; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L3969,3978,3989,3993,4001; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3972,4005; `ChangeId::short` (ids.rs:155) (by name) L3979; `ChangeId::new` (ids.rs:142) L3979; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:523)) L3979,3982,3996; `Dom::element` (xmllinq/mod.rs:655) (by name) L4003,4004
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.contains`, `.find`, `.into`, `.is_some`, `.iter`, `.map_or`, `.map`, `.or_else`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `Some`, `String::new`, `W::name`, `W::p`, `W::val`, `property`
- **Macros:** `format!`
- **Called by:** `Transaction::check_range` (edit.rs:3923) (?); `Transaction::check_insert_position` (edit.rs:4021) (?)

#### `Transaction::check_insert_position`  (L4021-4042)
`fn check_insert_position( &self, projection: &Projection, pos: usize, attach_before: bool, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `attach_segment` (edit.rs:5588) L4030; `Transaction::opaque_structure` (edit.rs:3959) (by name) L4035; `run_of` (edit.rs:6008) L4035; `is_symbol` (edit.rs:5583) L4038
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.ok_or_else`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_footnote` (edit/notes.rs:24); `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_rewrite` (edit.rs:3466); `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (edit.rs:7001) (?)

#### `Transaction::check_conflicts`  (L4044-4204)
`fn check_conflicts(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::conflict` (edit.rs:4477) L4049,4067,4074,4081,4087,4093...; `Transaction::check_deletions_leave_valid_containers` (edit.rs:4289) L4054; `Transaction::check_paragraph_ops` (edit.rs:4208) L4055; `Transaction::check_control_conflicts` (edit/controls.rs:420) L4056; `Transaction::check_comment_ids_fit` (edit.rs:4359) L4057; `Transaction::check_thread_ops` (edit.rs:4326) L4058; `format_overlaps_edit` (edit/runs.rs:39) L4165,4196; `targets_overlap` (edit/sections.rs:525) L4182
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.collect`, `.contains`, `.entry`, `.enumerate`, `.filter_map`, `.flatten`, `.get`, `.into_iter`, `.iter`, `.or_default`, `.push`, `.sort_by_key`, `.values_mut`, `BTreeMap::new`, `Err`, `Ok`, `Some`, `Vec::new`
- **Called by:** `Transaction::resolve` (edit.rs:2375); `deeper_boundary_tests::conflicts` (edit.rs:7056) (?)

#### `Transaction::check_paragraph_ops`  (L4208-4285)
`fn check_paragraph_ops(&self, deleted: &[usize]) -> Result<(), EditError>`

- **Does:** Formatting and merging, checked against each other, against deletions and against paragraph insertions.
- **Calls (crate):** `Transaction::conflict` (edit.rs:4477) L4216,4220,4227,4231,4244,4252...
- **Calls (std/external/unresolved):** `.contains`, `.push`, `Err`, `Ok`, `Vec::new`
- **Called by:** `Transaction::check_conflicts` (edit.rs:4044)

#### `Transaction::check_deletions_leave_valid_containers`  (L4289-4322)
`fn check_deletions_leave_valid_containers(&self, deleted: &[usize]) -> Result<(), EditError>`

- **Does:** Deletions, taken together, must leave every table cell they touch ending in a paragraph, and the body with a paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L4297; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L4298,4309; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L4301; `Transaction::conflict` (edit.rs:4477) L4302,4314; `Transaction::story_container` (edit.rs:3883) L4308
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.copied`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `.iter`, `.last`, `.map`, `.rev`, `Err`, `Ok`, `Some`, `W::p`, `W::tc`
- **Macros:** `format!`
- **Called by:** `Transaction::check_conflicts` (edit.rs:4044)

#### `Transaction::check_thread_ops`  (L4326-4355)
`fn check_thread_ops(&self) -> Result<(), EditError>`

- **Does:** Thread operations against each other: nothing may act on a comment another operation deletes, and a comment is edited once.
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:563) (by name) L4335; `Transaction::conflict` (edit.rs:4477) L4338,4351; `.target` (ambiguous: `EditOptions::target` (cli.rs:212); `Operation::target` (edit/v2.rs:573); `ThreadOp::target` (edit.rs:1906)) L4349
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.contains`, `.expect`, `.iter`, `.push`, `Err`, `Ok`, `Vec::new`
- **Called by:** `Transaction::check_conflicts` (edit.rs:4044)

#### `Transaction::check_comment_ids_fit`  (L4359-4369)
`fn check_comment_ids_fit(&self) -> Result<(), EditError>`

- **Does:** New comments take ids after the source's highest; refuse a plan whose comments would not fit in `w:id`'s 32 bits.
- **Calls (crate):** `Transaction::comments_planned` (edit.rs:4374) L4360; `err` (edit.rs:1401) L4362
- **Calls (std/external/unresolved):** `Err`, `Ok`, `u64::from`
- **Called by:** `Transaction::check_conflicts` (edit.rs:4044)

#### `Transaction::comments_planned`  (L4374-4395)
`fn comments_planned(&self) -> usize`

- **Does:** How many comments the resolved operations write: a comment on a text edit, an inserted or deleted paragraph, a comment operation and a reply each write one.
- **Calls (crate):** `Transaction::deletion_comment` (edit.rs:4974) L4383
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_some`, `.iter`
- **Macros:** `matches!`
- **Called by:** `Transaction::check_comment_ids_fit` (edit.rs:4359)

#### `Transaction::resolve_settings`  (L4399-4448)
`fn resolve_settings( &self, id: &str, request: &crate::settings::SettingsRequest, ) -> Result<EditOutcome, Box<(EditError, EditOutcome)>>`

- **Does:** Check a `settings` operation: something to write, no password, and one per plan.
- **Calls (crate):** `err` (edit.rs:1401) L4418; `SettingsRequest::describe` (settings.rs:216) L4446
- **Calls (std/external/unresolved):** `.as_ref`, `.is_some_and`, `.is_some`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `crate::settings::SettingsRequest::default`, `fail`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::check_redactions`  (L4453-4475)
`fn check_redactions(&self, outputs: &[&[u8]]) -> Result<(), EditError>`

- **Does:** Refuse the plan when the text of a redaction still occurs in one of `outputs` (a comment on it, another paragraph, a header, the document properties).
- **Calls (crate):** `leaks` (scrub.rs:396) L4460; `Transaction::conflict` (edit.rs:4477) L4469
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.flat_map`, `.into_iter`, `.into`, `.is_empty`, `.iter`, `.join`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `apply_plan` (edit.rs:1621)

#### `Transaction::conflict`  (L4477-4486)
`fn conflict(&self, op_index: usize, message: &str) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1401) L4479
- **Calls (std/external/unresolved):** `.clone`, `.into`, `.to_string`, `Some`
- **Called by:** `Transaction::check_control_conflicts` (edit/controls.rs:420); `check` (edit/tracked.rs:84); `apply_plan` (edit.rs:1621); `Transaction::check_conflicts` (edit.rs:4044); `Transaction::check_paragraph_ops` (edit.rs:4208); `Transaction::check_deletions_leave_valid_containers` (edit.rs:4289); `Transaction::check_thread_ops` (edit.rs:4326); `Transaction::check_redactions` (edit.rs:4453)

#### `Transaction::apply`  (L4489-4954)
`fn apply(&mut self) -> Result<(), EditError>`

- **Does:** Mutate the copy.
- **Calls (crate):** `Transaction::deletion_comment` (edit.rs:4974) L4506,4523; `Transaction::reserve_comment_id` (edit.rs:4962) L4525; `Transaction::new_comment` (edit.rs:4956) L4529; `Transaction::style_defined` (edit/notes.rs:83) L4549,4564; `Transaction::footnotes_story` (edit/notes.rs:51) L4561; `next_footnote_id` (edit/notes.rs:102) L4565; `append_footnote` (edit/notes.rs:132) L4567; `project_paragraph` (inspect.rs:1024) L4637,4823; `apply_text_edit` (edit.rs:5611) L4638; `new_position` (edit.rs:5448) L4650,4662,4663,4674,4690,4708...; `format_range` (edit.rs:6016) L4651,4664; `insert_reference` (edit/notes.rs:168) L4675; `whole::mark` (ambiguous: `mark` (edit/whole.rs:53); `tests::mark` (edit/whole.rs:458)) L4694; `anchor_comment` (edit.rs:5809) L4719,4824; `anchor_span` (edit.rs:5865) L4725; `Transaction::apply_fills` (edit/controls.rs:483) L4734; `toc_paragraphs` (edit.rs:6236) L4769; `build_paragraph` (edit.rs:6280) L4770; `build_table` (edit/structural.rs:153) L4788; `Transaction::image_paragraph` (edit/images.rs:143) L4805; `Dom::add_after_self` (xmllinq/mod.rs:1131) L4815; `Dom::add_before_self` (xmllinq/mod.rs:1114) L4818; `format_paragraph` (edit.rs:6064) L4836; `next_numbering_ids` (edit/structural.rs:350) L4846; `abstract_num` (markdown/xml.rs:2109) L4863; `.format` (ambiguous: `ListKind::format` (edit.rs:642); `RunSpec::format` (edit.rs:891); `Writer::format` (markdown/write.rs:2950)) L4865; `num` (markdown/xml.rs:2154) L4867; `number_paragraph` (edit/structural.rs:435) L4874; `merge_into` (edit.rs:6184) L4901; `Dom::remove` (xmllinq/mod.rs:1102) L4911; `place_reply_markers` (edit.rs:5894) L4922; `CommentFamily::with_replies` (comments.rs:563) (by name) L4926; `remove_comment_markers` (edit.rs:5932) L4931; `separate` (edit/structural.rs:242) L4939; `Transaction::apply_page_setup` (edit/sections.rs:394) L4951
- **Calls (std/external/unresolved):** `.as_deref`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.copied`, `.entry`, `.expect`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.len`, `.map`, `.or_default`, `.push`, `.remove`, `.rev`, `.saturating_add`, `.sort_by_key`, `.then_some`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `BTreeMap::new`, `Ok`, `Some`, `Vec::new`, `crate::markdown::max_drawing_id`, `std::collections::HashSet::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `apply_plan` (edit.rs:1621); `Transaction::commented_base` (edit.rs:4985)

#### `Transaction::new_comment`  (L4956-4960)
`fn new_comment(&mut self, text: String) -> u32`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::reserve_comment_id` (edit.rs:4962) L4957
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `Transaction::reserve_comment_id`  (L4962-4971)
`fn reserve_comment_id(&mut self) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.pop`, `.unwrap_or`, `u32::try_from`
- **Called by:** `Transaction::apply` (edit.rs:4489); `Transaction::new_comment` (edit.rs:4956)

#### `Transaction::deletion_comment`  (L4974-4979)
`fn deletion_comment(&self, op: usize) -> Option<&str>`

- **Does:** The comment of operation `op` when it deletes a paragraph.
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `redline` (edit/tracked.rs:185); `Transaction::comments_planned` (edit.rs:4374); `Transaction::apply` (edit.rs:4489)

#### `Transaction::commented_base`  (L4985-5073)
`fn commented_base(&self) -> Result<std::borrow::Cow<'_, [u8]>, EditError>`

- **Does:** The copy the comparer reads as the original: the base, with each deleted paragraph's comment anchored on its whole text.
- **Calls (crate):** `Transaction::start` (edit.rs:2004) L5062; `Transaction::resolve` (edit.rs:2375) L5069; `Transaction::apply` (edit.rs:4489) L5070; `Transaction::finish` (edit.rs:5077) L5071
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.extend`, `.filter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.rev`, `ExistingRevisions::default`, `Ok`, `Some`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Macros:** `matches!`, `unreachable!`
- **Called by:** `apply_plan` (edit.rs:1621) (?)

#### `Transaction::finish`  (L5077-5150)
`fn finish(&mut self) -> Result<(Vec<u8>, Option<Vec<u8>>), EditError>`

- **Does:** The clean copy, and the copy the comparer reads when `whole` replacements carry helper bookmarks (the clean copy never does).
- **Calls (crate):** `Transaction::write_comments_part` (edit.rs:5190) L5083; `add_styles` (edit/structural.rs:267) L5100; `splice_numbering` (edit/structural.rs:377) L5104; `Transaction::touched_stories` (edit.rs:3850) L5108; `Transaction::apply_watermark` (edit/watermark.rs:254) L5110; `apply_settings` (settings.rs:347) L5113; `err` (edit.rs:1401) L5114,5125,5148; `Transaction::removes_comment_reference` (edit.rs:5155) L5116; `Transaction::write_stories` (edit.rs:5179) L5120,5131; `PartFs::to_zip` (opc/mod.rs:548) (by name) L5121,5144; `strip` (edit/whole.rs:89) L5127; `revision_bearing_parts` (revision_processor.rs:3403) L5137
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cloned`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.to_string`, `Ok`, `Some`, `crate::revision_processor::prune_orphan_comments`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `redline` (edit/tracked.rs:185); `apply_plan` (edit.rs:1621); `Transaction::commented_base` (edit.rs:4985)

#### `Transaction::removes_comment_reference`  (L5155-5177)
`fn removes_comment_reference(&self, stories: &std::collections::BTreeSet<usize>) -> bool`

- **Does:** Whether the plan took a comment reference out of a story it writes (a deleted paragraph's, or deleted text's), read against the story as the package still holds it.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L5156; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L5161; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L5163; `PartFs::part_string` (opc/mod.rs:498) L5168; `Dom::new` (xmllinq/mod.rs:387) L5171; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5172; `Dom::root` (xmllinq/mod.rs:822) L5173; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2434) (by name) L5174
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter_map`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_subset`, `.iter`, `.map`, `Some`, `W::id`, `W::name`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `Transaction::write_stories`  (L5179-5185)
`fn write_stories(&mut self, stories: &std::collections::BTreeSet<usize>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::serialize_document` (xmllinq/mod.rs:1322) L5182; `PartFs::set_part` (opc/mod.rs:510) L5183
- **Calls (std/external/unresolved):** `.into_bytes`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `Transaction::write_comments_part`  (L5190-5232)
`fn write_comments_part(&mut self) -> Result<(), EditError>`

- **Does:** Write the comment part family: the plan's new comments and replies, then its edits, resolutions and deletions; every part consistent (see [`crate::comments`]).
- **Calls (crate):** `CommentFamily::load` (comments.rs:429) L5194; `err` (edit.rs:1401) L5195; `.add` (ambiguous, 5 candidates) L5198; `CommentFamily::set_text` (comments.rs:737) (by name) L5217; `CommentFamily::set_done` (comments.rs:774) (by name) L5222; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L5230
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.copied`, `.filter_map`, `.get`, `.iter`, `.map_err`, `.remove`, `.take`, `Ok`, `Some`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `comment_holds_kept_text`  (L5236-5262)
`fn comment_holds_kept_text(redline: &[u8], id: u32) -> bool`

- **Does:** Whether comment `id`'s range in `redline` holds text that is not deleted.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L5237; `Opened::story_parts` (inspect.rs:641) (by name) L5242; `parse_part` (inspect.rs:693) L5244; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L5248; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L5249; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L5252; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5253
- **Calls (std/external/unresolved):** `.as_str`, `.chain`, `.clone`, `.filter`, `.into_iter`, `.map`, `.name`, `.to_string`, `Some`, `W::id`, `is_id`, `std::iter::once`
- **Called by:** `apply_plan` (edit.rs:1621); `deeper_boundary_tests::invalid_package_has_no_revisions_or_kept_comment_text` (edit.rs:6870); `deeper_boundary_tests::revision_and_comment_scans_ignore_rootless_and_foreign_story_parts` (edit.rs:8016)

#### `existing_comment_ids`  (L5264-5277)
`fn existing_comment_ids(opened: &Opened) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::related` (inspect.rs:669) L5265; `PartFs::part_string` (opc/mod.rs:498) L5266; `Dom::new` (xmllinq/mod.rs:387) L5267; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5268; `Dom::root` (xmllinq/mod.rs:822) L5269; `Dom::descendants` (xmllinq/mod.rs:668) L5270; `Dom::attribute` (xmllinq/mod.rs:874) L5273
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.into_iter`, `.max`, `.next`, `.ok`, `.parse`, `Some`, `W::id`, `W::name`
- **Called by:** `Transaction::start` (edit.rs:2004)

#### `is_range_markup`  (L5280-5292)
`fn is_range_markup(dom: &Dom, node: NodeId) -> bool`

- **Does:** Markup that may sit between two block paragraphs and also inside one.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L5290; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L5291; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5291
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`
- **Called by:** `separate` (edit/structural.rs:242); `Transaction::merge_partner` (edit.rs:3691); `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (edit.rs:7762)

#### `kind_name`  (L5294-5321, pub(crate))
`pub(crate) fn kind_name(kind: &OperationKind) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::assert_runtime_dispatch` (capabilities/discovery.rs:324); `untracked_kind` (edit/tracked.rs:53) (?); `Transaction::resolve` (edit.rs:2375)

#### `comment_text_of`  (L5326-5331)
`fn comment_text_of(selector: &Selector) -> Option<u32>`

- **Does:** The operation edits or comments text it finds in its paragraph, so it may follow a paragraph into the one it merged with.
- **Calls (crate):** `comment_text_id` (ids.rs:301) L5328
- **Called by:** `Transaction::resolve` (edit.rs:2375); `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_rewrite` (edit.rs:3466); `Transaction::select` (edit.rs:3778)

#### `selector_of`  (L5334-5353)
`fn selector_of(kind: &OperationKind) -> Option<&Selector>`

- **Does:** The paragraph an operation names, when it names one.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Transaction::edits_a_comment_range` (edit.rs:2139); `Transaction::resolve` (edit.rs:2375)

#### `finds_text`  (L5355-5365)
`fn finds_text(kind: &OperationKind) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `check_text`  (L5368-5376)
`fn check_text(text: &str) -> Result<(), String>`

- **Does:** Plain text only: no control characters (tabs and breaks are not run text).
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.into`, `.is_control`, `Err`, `Ok`
- **Macros:** `matches!`
- **Called by:** `Transaction::fill_value` (edit/controls.rs:313); `picture` (edit/images.rs:86); `Transaction::resolve_footnote` (edit/notes.rs:24); `check_rows` (edit/structural.rs:28); `spec` (edit/watermark.rs:43); `Transaction::resolve_one` (edit.rs:2528); `tests::check_text_rejects_controls` (edit.rs:6624)

#### `check_run_text`  (L5381-5397)
`fn check_run_text(text: &str) -> Result<(), String>`

- **Does:** Run text: plain text in which `\t` writes a tab and `\n` a line break, as the agent view prints them; any other control character, and the view's symbol placeholder U+FFFC, is refused by name.
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.into`, `.is_control`, `Err`, `Ok`, `from`
- **Macros:** `format!`, `matches!`
- **Called by:** `Transaction::resolve_comment_text` (edit.rs:2242); `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_rewrite` (edit.rs:3466)

#### `blank_is_none`  (L5401-5404)
`fn blank_is_none<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<String>, D::Error>`

- **Does:** An optional comment as a plan gives it: blank text (`""`, spaces, line breaks) is no comment, as an agent with nothing to say writes it.
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.trim`, `Ok`, `serde::Deserialize::deserialize`
- **Called by:** none found in src

#### `check_comment`  (L5408-5413)
`fn check_comment(text: &str) -> Result<(), String>`

- **Does:** Comment text: nonempty, and each line (`\n` starts a new comment paragraph) plain text that `comments.xml` can carry.
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.split`, `.trim`, `.try_for_each`, `Err`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_thread` (edit.rs:3342)

#### `redaction`  (L5417-5419)
`fn redaction(find: &str) -> String`

- **Does:** `before {mark} after` with up to 20 chars of context on either side.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.repeat`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `context`  (L5421-5433)
`fn context(text: &str, start: usize, end: usize, mark: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.into_iter`, `.rev`, `.take`
- **Macros:** `format!`
- **Called by:** `public_option_source_ownership_tests::assert_source_projection` (comparer/cross_para.rs:2664) (?); `coverage_real_revision_merge_matrix_tests::compact_source_difference` (comparer/finalize.rs:24897) (?); `finalizer_remaining_source_boundary_tests::assert_body` (comparer/finalize.rs:30744) (?); `finalizer_merge_source_guard_tests::assert_body` (comparer/finalize.rs:31486) (?); `finalizer_reached_predicate_boundary_tests::assert_body` (comparer/finalize.rs:31923) (?); `existing_mark_layout_ownership_tests::assert_body` (comparer/finalize.rs:32777) (?); `interleave_terminal_source_ownership_tests::remains_source_owned` (comparer/finalize.rs:33122) (?); `whitespace_source_mark_carrier_tests::check_complete` (comparer/finalize.rs:33330) (?); `fragmented_source_label_boundary_tests::assert_complete` (comparer/finalize.rs:33500) (?); `row_pair_uses_word_table_mesh_context` (comparer/lcs_table.rs:530) (?); `Transaction::resolve_comment_text` (edit.rs:2242); `Transaction::resolve_one` (edit.rs:2528); +1 more

#### `excerpt`  (L5435-5441)
`fn excerpt(text: &str, max: usize) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.push`, `.take`
- **Called by:** `Transaction::resolve_fill_control` (edit/controls.rs:72); `Transaction::resolve_one` (edit.rs:2528); `Transaction::resolve_rewrite` (edit.rs:3466); `Transaction::resolve_list` (edit.rs:3563); `tests::context_windows_and_excerpts` (edit.rs:6578); `describe` (markdown/patch.rs:284)

#### `new_position`  (L5448-5470)
`fn new_position(edits: &[ScheduledEdit], pos: usize, inclusive: bool, own: Option<usize>) -> usize`

- **Does:** Position `pos` of the source projection after `edits` were applied.
- **Calls (std/external/unresolved):** `.is_none_or`, `.len`, `.max`, `Some`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `tests::new_position_accounts_for_earlier_edits_and_insertions` (edit.rs:6588)

#### `locate`  (L5475-5580)
`fn locate( projection: &Projection, find: &str, occurrence: Option<usize>, outcome: &mut EditOutcome, window: Option<(u32, usize, usize)>, ) -> Result<(usize, usize), (String, String)>`

- **Does:** The unique occurrence of `find` in the paragraph, or its `occurrence`-th hit (1-based) when given (overlapping occurrences count), whatever structure it crosses.
- **Calls (crate):** `inside` (ambiguous: `inside` (comparer/finalize.rs:14934); `inside` (markdown/critic.rs:375)) L5509,5529,5531; `reading_hits` (edit/anchor.rs:79) L5529,5531; `shown` (edit/anchor.rs:95) L5543
- **Calls (std/external/unresolved):** `.as_slice`, `.char_indices`, `.collect`, `.copied`, `.filter`, `.find`, `.get`, `.into_iter`, `.into`, `.is_empty`, `.iter`, `.len`, `.map`, `.push`, `.set`, `.starts_with`, `.then_some`, `.to_string`, `Err`, `Ok`, `Some`, `Vec::new`, `crate::markdown::plain_anchor`, `exact`, `hits_of`, `std::cell::Cell::new`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528); `Transaction::find_range` (edit.rs:3901)

#### `is_symbol`  (L5583-5585)
`fn is_symbol(projection: &Projection, seg: &crate::inspect::Segment) -> bool`

- **Does:** `seg` is a `w:sym`, projected as U+FFFC.
- **Macros:** `matches!`
- **Called by:** `Transaction::check_range` (edit.rs:3923); `Transaction::check_insert_position` (edit.rs:4021)

#### `attach_segment`  (L5588-5605)
`fn attach_segment( projection: &Projection, pos: usize, attach_before: bool, ) -> Option<&crate::inspect::Segment>`

- **Does:** The segment an insertion at `pos` attaches to.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.or_else`, `.rev`
- **Called by:** `insert_reference` (edit/notes.rs:168); `emit_text` (edit/tracked.rs:732); `Transaction::check_insert_position` (edit.rs:4021); `apply_text_edit` (edit.rs:5611); `deeper_boundary_tests::text_edit_glyphs_and_segment_attachment_are_exact` (edit.rs:7550)

#### `apply_text_edit`  (L5611-5669)
`fn apply_text_edit( dom: &mut Dom, projection: &Projection, start: usize, end: usize, replacement: &str, attach_before: bool, )`

- **Does:** Replace `[start, end)` of the paragraph's projection with `replacement`.
- **Calls (crate):** `attach_segment` (edit.rs:5588) L5620; `Dom::value` (xmllinq/mod.rs:1169) L5623,5650; `write_text_new_in` (edit.rs:5736) L5626,5655; `Dom::new_element` (xmllinq/mod.rs:478) L5631,5660; `Dom::add_after_self` (xmllinq/mod.rs:1131) L5633; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5635,5661; `write_text` (edit.rs:5729) L5637,5662; `inner_comment_references` (edit.rs:5674) L5642; `Dom::remove` (xmllinq/mod.rs:1102) L5664; `remove_comment_references` (edit.rs:5708) L5668
- **Calls (std/external/unresolved):** `.expect`, `.filter`, `.is_empty`, `.len`, `.min`, `.saturating_sub`, `.take`, `.unwrap_or_default`, `Some`, `W::t`
- **Macros:** `format!`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::text_edit_glyphs_and_segment_attachment_are_exact` (edit.rs:7550)

#### `inner_comment_references`  (L5674-5705)
`fn inner_comment_references( dom: &Dom, projection: &Projection, start: usize, end: usize, ) -> Vec<NodeId>`

- **Does:** The comment references between the first and the last character of `[start, end)`: a selection over them takes their mark along, and Word then drops the comment (rule R1).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L5693; `Dom::descendants` (xmllinq/mod.rs:668) L5696,5701; `at` (markdown/from_docx/drawing.rs:1134) (by name) L5698,5703
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.first`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.next_back`, `.next`, `.position`, `Some`, `Vec::new`, `W::name`, `W::p`
- **Called by:** `apply_text_edit` (edit.rs:5611)

#### `remove_comment_references`  (L5708-5719)
`fn remove_comment_references(dom: &mut Dom, references: &[NodeId])`

- **Does:** Remove comment references, with the run when it holds nothing else.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L5710; `Dom::name_is` (xmllinq/mod.rs:535) L5711,5715; `Dom::elements` (xmllinq/mod.rs:640) L5712; `Dom::remove` (xmllinq/mod.rs:1102) L5717
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.into_iter`, `.unwrap_or`, `W::r_pr`, `W::r`
- **Called by:** `apply_text_edit` (edit.rs:5611)

#### `set_text`  (L5721-5724)
`fn set_text(dom: &mut Dom, t: NodeId, value: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::set_value` (xmllinq/mod.rs:1203) L5722; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5723; `XNamespace::xml` (xmllinq/mod.rs:128) L5723
- **Calls (std/external/unresolved):** `.name`, `Some`
- **Called by:** `emit` (edit/tracked.rs:275); `emit_text` (edit/tracked.rs:732); `write_text_new_in` (edit.rs:5736); `split_run_at` (edit.rs:5771); `merge_into` (edit.rs:6184)

#### `write_text`  (L5729-5731)
`fn write_text(dom: &mut Dom, t: NodeId, value: &str)`

- **Does:** Set the text of `t`, a `w:t` in a run, writing each `\t` as a `w:tab` and each `\n` as a `w:br` of that run, with the text between them in `w:t`s of its own; `t` itself goes when no text is left for it.
- **Calls (crate):** `write_text_new_in` (edit.rs:5736) L5730
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `apply_text_edit` (edit.rs:5611); `build_paragraph` (edit.rs:6280)

#### `write_text_new_in`  (L5736-5767)
`fn write_text_new_in(dom: &mut Dom, t: NodeId, value: &str, new: std::ops::Range<usize>)`

- **Does:** [`write_text`] for `value` whose new text is `new` alone: the text kept around it stays as the source wrote it, a tab typed in `w:t` included, so an edit never restates what it did not touch.
- **Calls (crate):** `set_text` (edit.rs:5721) L5742,5745,5760; `Dom::new_element` (xmllinq/mod.rs:478) L5753,5758; `Dom::add_after_self` (xmllinq/mod.rs:1131) L5754,5759; `Dom::remove` (xmllinq/mod.rs:1102) L5765
- **Calls (std/external/unresolved):** `.as_bytes`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.first`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.match_indices`, `.unwrap_or`, `W::name`, `W::t`
- **Called by:** `apply_text_edit` (edit.rs:5611); `write_text` (edit.rs:5729)

#### `split_run_at`  (L5771-5805)
`fn split_run_at(dom: &mut Dom, seg: &crate::inspect::Segment, at: usize)`

- **Does:** Split the run owning `seg` so that a run boundary falls at projection byte `at` (inside `seg`).
- **Calls (crate):** `Dom::value` (xmllinq/mod.rs:1169) L5778; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5783; `Dom::elements` (xmllinq/mod.rs:640) L5784,5785; `Dom::name_is` (xmllinq/mod.rs:535) L5791; `Dom::remove` (xmllinq/mod.rs:1102) L5793,5798; `set_text` (edit.rs:5721) L5801,5803; `Dom::add_after_self` (xmllinq/mod.rs:1131) L5804
- **Calls (std/external/unresolved):** `.enumerate`, `.expect`, `.iter`, `.position`, `.to_string`, `W::r_pr`
- **Called by:** `insert_reference` (edit/notes.rs:168); `emit_text` (edit/tracked.rs:732); `isolate_range` (edit/tracked.rs:826); `wrap_range` (edit.rs:5960); `runs_in_range` (edit.rs:6035); `deeper_boundary_tests::splitting_multichild_run_preserves_order_and_properties` (edit.rs:7597)

#### `anchor_comment`  (L5809-5826)
`fn anchor_comment(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, id: u32)`

- **Does:** Wrap the projection range `[start, end)` of `paragraph` in comment markers for comment `id`, splitting runs at the boundaries as needed.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L5811,5813,5815,5816; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5812,5814,5817; `Dom::add` (xmllinq/mod.rs:1058) L5818,5822,5823,5824; `wrap_range` (edit.rs:5960) L5819; `place_reference` (edit.rs:5834) L5820
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::id`, `W::name`, `W::r`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:7631)

#### `place_reference`  (L5834-5847)
`fn place_reference(dom: &mut Dom, range_start: NodeId, range_end: NodeId, reference_run: NodeId)`

- **Does:** Put a comment's reference run right after its end marker, or, when the range starts outside the tracked insertion holding that marker, after the insertion: rejecting it would take a reference inside it, and the comment with it, though only part of the commented text was inserted.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L5836,5841; `Dom::name_is` (xmllinq/mod.rs:535) L5837,5840; `Dom::add_after_self` (xmllinq/mod.rs:1131) L5846
- **Calls (std/external/unresolved):** `.contains`, `W::ins`, `W::name`, `W::p`
- **Called by:** `anchor_comment` (edit.rs:5809); `anchor_span` (edit.rs:5865)

#### `comment_marker`  (L5849-5853)
`fn comment_marker(dom: &mut Dom, local: &str, id: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L5850; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5851
- **Calls (std/external/unresolved):** `Some`, `W::id`, `W::name`
- **Called by:** `comment_reference_run` (edit.rs:5855); `anchor_span` (edit.rs:5865); `place_reply_markers` (edit.rs:5894); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:7631)

#### `comment_reference_run`  (L5855-5860)
`fn comment_reference_run(dom: &mut Dom, id: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L5856; `comment_marker` (edit.rs:5849) L5857; `Dom::add` (xmllinq/mod.rs:1058) L5858
- **Calls (std/external/unresolved):** `W::r`
- **Called by:** `anchor_span` (edit.rs:5865); `place_reply_markers` (edit.rs:5894)

#### `anchor_span`  (L5865-5889)
`fn anchor_span(dom: &mut Dom, first: NodeId, last: NodeId, id: u32)`

- **Does:** Comment `id` from the start of paragraph `first` to the end of `last`: the start marker before `first`'s first run, the end marker and the reference run after `last`'s last run (appended to an empty paragraph).
- **Calls (crate):** `comment_marker` (edit.rs:5849) L5867,5868; `comment_reference_run` (edit.rs:5855) L5869; `project_paragraph` (inspect.rs:1024) L5870,5880; `Dom::new_element` (xmllinq/mod.rs:478) L5871,5881; `wrap_range` (edit.rs:5960) L5872,5882; `Dom::remove` (xmllinq/mod.rs:1102) L5873,5883; `Dom::element` (xmllinq/mod.rs:655) L5875; `Dom::add_after_self` (xmllinq/mod.rs:1131) L5876; `Dom::add_first` (xmllinq/mod.rs:1085) L5877; `place_reference` (edit.rs:5834) L5884; `Dom::add` (xmllinq/mod.rs:1058) L5886,5887
- **Calls (std/external/unresolved):** `.len`, `.to_string`, `W::name`, `W::p_pr`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:7631)

#### `place_reply_markers`  (L5894-5928)
`fn place_reply_markers(dom: &mut Dom, roots: &[NodeId], parent: u32, id: u32)`

- **Does:** Markers of reply `id` beside those of comment `parent`, as Word writes them: the start after the parent's start, the end after the parent's end, the reference run after the parent's reference run.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L5899; `Dom::attribute` (xmllinq/mod.rs:874) L5901; `find` (ambiguous: `find` (markdown/pages.rs:451); `find` (markdown/unified.rs:464)) L5905,5906,5907; `Dom::parent` (xmllinq/mod.rs:597) L5912; `Dom::name_is` (xmllinq/mod.rs:535) L5914; `comment_reference_run` (edit.rs:5855) L5916; `Dom::add_after_self` (xmllinq/mod.rs:1131) L5917,5920,5925; `comment_marker` (edit.rs:5849) L5918,5923; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5921,5926
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.find_map`, `.find`, `.into_iter`, `.iter`, `.to_string`, `.unwrap_or`, `Some`, `W::id`, `W::name`, `W::r`
- **Called by:** `thread_markers` (edit/tracked.rs:701); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:7631)

#### `remove_comment_markers`  (L5932-5955)
`fn remove_comment_markers(dom: &mut Dom, roots: &[NodeId], ids: &[String])`

- **Does:** Remove the range markers and references of the comments `ids`; a run left holding nothing but its properties goes too.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L5935; `Dom::attribute` (xmllinq/mod.rs:874) L5936; `Dom::parent` (xmllinq/mod.rs:597) L5942; `Dom::name_is` (xmllinq/mod.rs:535) L5942,5948; `Dom::remove` (xmllinq/mod.rs:1102) L5943,5950; `Dom::elements` (xmllinq/mod.rs:640) L5945
- **Calls (std/external/unresolved):** `.all`, `.any`, `.filter`, `.is_some_and`, `.iter`, `Some`, `W::id`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `thread_markers` (edit/tracked.rs:701); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:7631)

#### `wrap_range`  (L5960-6006)
`fn wrap_range( dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, open: NodeId, close: NodeId, ) -> bool`

- **Does:** Put `open` right before the run holding projection offset `start` and `close` right after the run ending at `end`, splitting runs at both offsets first.
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L5968,5977,5986; `split_run_at` (edit.rs:5771) L5975,5984; `run_of` (edit.rs:6008) L5991,5997; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6000; `Dom::add_after_self` (xmllinq/mod.rs:1131) L6001
- **Calls (std/external/unresolved):** `.cloned`, `.find`, `.iter`, `.map`, `.max`, `.rev`
- **Called by:** `mark` (edit/whole.rs:53); `anchor_comment` (edit.rs:5809); `anchor_span` (edit.rs:5865); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:7631)

#### `run_of`  (L6008-6012)
`fn run_of(piece: &Piece) -> NodeId`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `insert_reference` (edit/notes.rs:168); `Transaction::check_range` (edit.rs:3923); `Transaction::check_insert_position` (edit.rs:4021); `wrap_range` (edit.rs:5960); `runs_in_range` (edit.rs:6035)

#### `format_range`  (L6016-6031)
`fn format_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, format: &RunFormat)`

- **Does:** Apply `format` to the runs holding projection range `[start, end)` of `paragraph`, splitting runs at the boundaries first.
- **Calls (crate):** `runs_in_range` (edit.rs:6035) L6017; `Dom::element` (xmllinq/mod.rs:655) L6018; `Dom::new_element` (xmllinq/mod.rs:478) L6021; `Dom::add_first` (xmllinq/mod.rs:1085) L6022; `apply_run_format` (edit.rs:6355) L6026; `Dom::elements` (xmllinq/mod.rs:640) L6027; `Dom::remove` (xmllinq/mod.rs:1102) L6028
- **Calls (std/external/unresolved):** `.is_empty`, `W::r_pr`
- **Called by:** `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::format_range_deduplicates_runs_and_removes_empty_properties` (edit.rs:7682)

#### `runs_in_range`  (L6035-6061)
`fn runs_in_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize) -> Vec<NodeId>`

- **Does:** The runs holding projection range `[start, end)` of `paragraph`, in order, split at the boundaries first; none for an empty range.
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L6040,6050; `split_run_at` (edit.rs:5771) L6047; `run_of` (edit.rs:6008) L6054
- **Calls (std/external/unresolved):** `.cloned`, `.contains`, `.find`, `.iter`, `.push`, `Vec::new`
- **Called by:** `track_run_format` (edit/tracked.rs:669); `format_range` (edit.rs:6016)

#### `format_paragraph`  (L6064-6120)
`fn format_paragraph( dom: &mut Dom, paragraph: NodeId, style: Option<&str>, alignment: Option<Alignment>, spacing: Spacing, )`

- **Does:** Set the paragraph style, alignment and spacing in `paragraph`'s `w:pPr`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6071,6080,6089,6098; `Dom::new_element` (xmllinq/mod.rs:478) L6074,6083,6092,6101; `Dom::add_first` (xmllinq/mod.rs:1085) L6075,6085; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6081,6084,6090,6093,6107,6108...; `insert_ppr_child` (edit.rs:6163) L6094,6102
- **Calls (std/external/unresolved):** `.jc`, `.to_string`, `Some`, `Spacing::default`, `W::name`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::paragraph_format_partial_spacing_preserves_other_axes` (edit.rs:7704)

#### `insert_ppr_child`  (L6163-6179)
`fn insert_ppr_child(dom: &mut Dom, ppr: NodeId, child: NodeId)`

- **Does:** Insert `child` into `ppr` at its schema position.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L6170,6172; `rank` (edit.rs:6517) L6170,6173; `XName::local_name` (xmllinq/mod.rs:208) (by name) L6170,6173; `Dom::elements` (xmllinq/mod.rs:640) L6171; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L6173; `Dom::add_after_self` (xmllinq/mod.rs:1131) L6176; `Dom::add_first` (xmllinq/mod.rs:1085) L6177
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.position`, `.rev`, `.unwrap_or`
- **Called by:** `number_paragraph` (edit/structural.rs:435); `track_properties` (edit/tracked.rs:633); `mark` (edit/tracked.rs:1012); `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (edit/tracked.rs:1237); `format_paragraph` (edit.rs:6064); `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (edit.rs:7762)

#### `merge_into`  (L6184-6232)
`fn merge_into(dom: &mut Dom, head: NodeId, next: NodeId, separator: &str)`

- **Does:** Move `head`'s content, then `separator` as a run, then any range markup between the two, to the start of `next`, and drop `head`.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L6185,6218; `Dom::name_is` (xmllinq/mod.rs:535) L6188,6199,6200,6201,6221; `Dom::new_element` (xmllinq/mod.rs:478) L6191,6208; `Dom::elements` (xmllinq/mod.rs:640) L6192,6198; `Dom::element` (xmllinq/mod.rs:655) L6195; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L6197; `Dom::remove` (xmllinq/mod.rs:1102) L6203,6224,6231; `Dom::add` (xmllinq/mod.rs:1058) L6206,6210,6228; `set_text` (edit.rs:5721) L6209; `Dom::next_element` (xmllinq/mod.rs:855) L6213,6216; `Dom::parent` (xmllinq/mod.rs:597) L6223; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6227
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some`, `.last`, `.push`, `Some`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::merge_separator_copies_properties_without_revision_children` (edit.rs:7806)

#### `toc_paragraphs`  (L6236-6275)
`fn toc_paragraphs(dom: &mut Dom, toc: &TocSpec) -> Vec<NodeId>`

- **Does:** `insert_toc`'s paragraphs: the optional `TOCHeading` title, then a paragraph holding an empty `TOC` field.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L6239,6240,6241,6245,6246,6253...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6242,6247,6257,6264; `Dom::add` (xmllinq/mod.rs:1058) L6243,6244,6249,6250,6258,6259...; `XNamespace::xml` (xmllinq/mod.rs:128) L6247,6264; `Dom::add_text` (xmllinq/mod.rs:1071) L6248,6265; `mark` (ambiguous, 5 candidates) L6261,6271,6272
- **Calls (std/external/unresolved):** `.name`, `.push`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p_style`, `W::p`, `W::r`, `W::t`, `W::val`
- **Macros:** `format!`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489)

#### `build_paragraph`  (L6280-6352)
`fn build_paragraph(dom: &mut Dom, anchor: NodeId, runs: &[RunSpec], style: Option<&str>) -> NodeId`

- **Does:** A new paragraph modeled on `anchor`: its `pPr` minus section break and revision marks, runs formatted like the anchor's first run plus the requested toggles.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L6281,6296,6303,6304,6327,6330...; `Dom::element` (xmllinq/mod.rs:655) L6282,6293,6322; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L6283,6329; `Dom::elements` (xmllinq/mod.rs:640) L6284,6312,6317,6332,6341; `Dom::name_is` (xmllinq/mod.rs:535) L6285,6286,6287,6333,6334,6335; `Dom::remove` (xmllinq/mod.rs:1102) L6289,6337,6342; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6294,6297,6305; `Dom::add_first` (xmllinq/mod.rs:1085) L6298; `Dom::add` (xmllinq/mod.rs:1058) L6301,6306,6307,6344,6347,6349; `Dom::value` (xmllinq/mod.rs:1169) L6319; `apply_run_format` (edit.rs:6355) L6340; `.format` (ambiguous: `ListKind::format` (edit.rs:642); `RunSpec::format` (edit.rs:891); `Writer::format` (markdown/write.rs:2950)) L6340; `write_text` (edit.rs:5729) L6348
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.count`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.max_by_key`, `.rev`, `.sum`, `Some`, `W::del`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::r_pr_change`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `W::val`
- **Called by:** `cell_paragraph` (edit/structural.rs:220); `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `deeper_boundary_tests::new_paragraph_style_and_revision_cleanup_truth_table` (edit.rs:7839)

#### `apply_run_format`  (L6355-6410)
`fn apply_run_format(dom: &mut Dom, rpr: NodeId, format: &RunFormat)`

- **Does:** Set or clear the requested properties in `rpr`.
- **Calls (crate):** `toggle` (edit.rs:6538) L6356,6357,6358,6359,6380,6381; `Dom::element` (xmllinq/mod.rs:655) L6361,6371,6383,6393,6402; `Dom::remove` (xmllinq/mod.rs:1102) L6362,6372,6384,6394,6403; `Dom::new_element` (xmllinq/mod.rs:478) L6365,6375,6386,6396,6405; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6366,6376,6388,6397,6406; `insert_rpr_child` (edit.rs:6525) L6367,6377,6390,6398,6407
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::name`, `W::val`
- **Called by:** `track_run_format` (edit/tracked.rs:669); `emit_text` (edit/tracked.rs:732); `format_range` (edit.rs:6016); `build_paragraph` (edit.rs:6280); `deeper_boundary_tests::replacing_and_clearing_existing_run_properties` (edit.rs:7886)

#### `check_format`  (L6434-6472)
`fn check_format(format: &RunFormat, text: &str) -> Result<(), String>`

- **Does:** A format must name a real highlight colour and apply to some text.
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.bytes`, `.chars`, `.contains`, `.count`, `.into`, `.is_ascii_hexdigit`, `.is_empty`, `.join`, `.len`, `.trim`, `Err`, `Ok`, `RunFormat::default`, `from`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_format_run` (edit/runs.rs:17); `Transaction::resolve_one` (edit.rs:2528); `deeper_boundary_tests::format_validation_unicode_length_and_color_truth_tables` (edit.rs:7941)

#### `rank`  (L6517-6522)
`fn rank(local: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.position`, `.unwrap_or`
- **Called by:** `insert_ppr_child` (edit.rs:6163); `insert_rpr_child` (edit.rs:6525)

#### `insert_rpr_child`  (L6525-6535)
`fn insert_rpr_child(dom: &mut Dom, rpr: NodeId, child: NodeId)`

- **Does:** Insert `child` into `rpr` at its schema position.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L6526,6528; `rank` (edit.rs:6517) L6526,6529; `XName::local_name` (xmllinq/mod.rs:208) (by name) L6526,6529; `Dom::elements` (xmllinq/mod.rs:640) L6527; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L6529; `Dom::add_after_self` (xmllinq/mod.rs:1131) L6532; `Dom::add_first` (xmllinq/mod.rs:1085) L6533
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.map_or`, `.rev`
- **Called by:** `set_glyph_font` (edit/controls.rs:647); `apply_run_format` (edit.rs:6355); `toggle` (edit.rs:6538); `tests::rpr_children_are_inserted_in_schema_order` (edit.rs:6632); `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (edit.rs:7762)

#### `toggle`  (L6538-6548)
`fn toggle(dom: &mut Dom, rpr: NodeId, local: &str, value: Option<bool>)`

- **Does:** Set or clear a boolean run property (`w:b`, `w:i`, ...).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6540; `Dom::remove` (xmllinq/mod.rs:1102) L6541; `Dom::new_element` (xmllinq/mod.rs:478) L6543; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6545; `insert_rpr_child` (edit.rs:6525) L6547
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::val`
- **Called by:** `apply_run_format` (edit.rs:6355); `tests::rpr_children_are_inserted_in_schema_order` (edit.rs:6632); `definition` (markdown/view_styles.rs:734) (?)

## Test functions (37)

- `tests::plan_json_roundtrips_and_rejects_unknown_fields` (L6556): _no doc comment_
- `tests::context_windows_and_excerpts` (L6578): _no doc comment_
- `tests::new_position_accounts_for_earlier_edits_and_insertions` (L6588): _no doc comment_
- `tests::check_text_rejects_controls` (L6624): _no doc comment_
- `tests::rpr_children_are_inserted_in_schema_order` (L6632): _no doc comment_
- `tests::error_display_names_the_operation` (L6656): _no doc comment_
- `deeper_boundary_tests::plan` (L6675): _no doc comment_
- `deeper_boundary_tests::dom` (L6681): _no doc comment_
- `deeper_boundary_tests::names` (L6691): _no doc comment_
- `deeper_boundary_tests::outcome` (L6698): _no doc comment_
- `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (L6716): Baseline gaps: 113, 916, 933, 936, 942, 945, 958. _(from a code comment)_
- `deeper_boundary_tests::half_points_finite_range_and_rounding_boundaries` (L6770): 702's two disjuncts, including values JSON cannot represent. _(from a code comment)_
- `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (L6789): 1055, 1069, 1073: no selection, empty filters, reject-only selection. _(from a code comment)_
- `deeper_boundary_tests::admission_boundaries_and_signature_prefix_are_exact` (L6828): 1652, 1669, 1677; validation precedes package parsing. _(from a code comment)_
- `deeper_boundary_tests::invalid_package_has_no_revisions_or_kept_comment_text` (L6870): 1376, 1389, 4519: invalid byte input is handled without panic. _(from a code comment)_
- `deeper_boundary_tests::resolution_empty_text_and_combined_comment_anchors` (L6877): 2259, 2312, 2339, 2503, 2703, 3173. _(from a code comment)_
- `deeper_boundary_tests::header_refuses_body_only_picture_toc_and_list` (L6938): Body-only operations with a real in-memory header story: 2432, 2688, 2987. _(from a code comment)_
- `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (L6974): 3109, 3114: a related style part can disappear/become rootless. _(from a code comment)_
- `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (L7001): 3336, 3342, 3368: end coverage, field-boundary equality, and glyph attachment. _(from a code comment)_
- `deeper_boundary_tests::text_range` (L7044): _no doc comment_
- `deeper_boundary_tests::conflicts` (L7056): _no doc comment_
- `deeper_boundary_tests::overlapping_range_and_comment_edge_truth_tables` (L7249): 3460/3461 and 3477: same-start replacements vs insertions, and either comment edge. _(from a code comment)_
- `deeper_boundary_tests::deleted_paragraph_rejects_each_anchor_owner` (L7306): All deletion conflict arms use explicit functional error assertions, not a call sweep. _(from a code comment)_
- `deeper_boundary_tests::table_insertion_sides_at_merge_boundaries` (L7426): 3586's Before/tail arm, with a table rather than the existing paragraph integration case. _(from a code comment)_
- `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (L7466): 2804, 2842, 2851, 3667, 3680. _(from a code comment)_
- `deeper_boundary_tests::text_edit_glyphs_and_segment_attachment_are_exact` (L7550): 4697, 4713, 4726: insertion affinity and glyphs never turn into text nodes. _(from a code comment)_
- `deeper_boundary_tests::splitting_multichild_run_preserves_order_and_properties` (L7597): 4751, 4754, 4771, 4776: both edges, a glyph, and children on either side of t. _(from a code comment)_
- `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (L7631): 4798, 4830, 4840, 4867, 4903, 4957. _(from a code comment)_
- `deeper_boundary_tests::format_range_deduplicates_runs_and_removes_empty_properties` (L7682): 4975, 4992, 4994, 5009: no-op ranges, empty t, and two segments in one run. _(from a code comment)_
- `deeper_boundary_tests::paragraph_format_partial_spacing_preserves_other_axes` (L7704): 5032, 5041, 5058, 5063, 5067; only the requested spacing axis changes. _(from a code comment)_
- `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (L7762): 5125, 5482: higher-ranked and foreign children do not supply the insertion anchor. _(from a code comment)_
- `deeper_boundary_tests::merge_separator_copies_properties_without_revision_children` (L7806): 5142, 5148, 5151-5153: separator inherits clean properties only. _(from a code comment)_
- `deeper_boundary_tests::new_paragraph_style_and_revision_cleanup_truth_table` (L7839): 5238, 5244, 5245, 5276, 5285-5287: property cloning and empty run specs. _(from a code comment)_
- `deeper_boundary_tests::replacing_and_clearing_existing_run_properties` (L7886): 5314, 5317, 5324, 5327, 5336, 5346, 5355. _(from a code comment)_
- `deeper_boundary_tests::format_validation_unicode_length_and_color_truth_tables` (L7941): 5398, 5399, 5406, 5407, 5421. _(from a code comment)_
- `deeper_boundary_tests::whole_deletion_builds_no_bookmark_and_keeps_surrounding_text` (L7999): 4009: whole deletion skips replacement bookmarks. _(from a code comment)_
- `deeper_boundary_tests::revision_and_comment_scans_ignore_rootless_and_foreign_story_parts` (L8016): 1402, 1414, 4526: tolerate an unreadable story and ignore foreign revisions. _(from a code comment)_
