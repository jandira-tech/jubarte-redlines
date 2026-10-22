# Functions and methods in `src/edit.rs`

205 non-test, 44 test. Part of the [symbol manifest](../README.md).

#### `RemovedAnchorComments::is_remove`  (L162-164, pub(crate))
`pub(crate) fn is_remove(&self) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<ResolveRevisions as From>::from`  (L210-221, impl From)
`fn from(wire: ResolveRevisionsWire) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `.is_some_and`, `.map`
- **Called by:** none found in src

#### `<ResolveRevisionsWire as From>::from`  (L225-235, impl From)
`fn from(selection: ResolveRevisions) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `ResolvedRevisions::is_empty`  (L271-273, pub)
`pub fn is_empty(&self) -> bool`

- **Does:** Whether no change was accepted or rejected.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `is_pure_resolution` (bin/jubarte.rs:2209); `apply_bound` (edit/v2_apply.rs:121); `EditReport::to_jsonl` (edit.rs:1246)

#### `default_toc_levels`  (L790-792)
`fn default_toc_levels() -> u8`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `is_default_margins`  (L794-796)
`fn is_default_margins(margins: &Margins) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Margins::default`
- **Called by:** none found in src

#### `resolve_done`  (L798-800)
`fn resolve_done() -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `default_true`  (L802-804)
`fn default_true() -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `ListKind::format`  (L822-829)
`fn format(self) -> crate::markdown::xml::ListFormat`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `emit` (edit/tracked.rs:452) (?); `Transaction::resolve_one_unreported` (edit.rs:3545) (?); `Transaction::apply_lists` (edit.rs:6303) (?); `build_paragraph` (edit.rs:8299) (?); `raster_extension` (markdown/from_docx/media.rs:77) (?); `read_picture` (markdown/write.rs:6479) (?)

#### `ListKind::name`  (L831-837)
`fn name(self) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<LineSpacing as Serialize>::serialize`  (L846-848, impl Serialize)
`fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.serialize_f64`, `f64::from`
- **Called by:** `serialize_timeout` (cli.rs:2885) (?)

#### `<LineSpacing as Deserialize>::deserialize`  (L852-860, impl Deserialize)
`fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.round`, `Err`, `Ok`, `Self`, `f64::deserialize`, `serde::de::Error::custom`
- **Macros:** `format!`
- **Called by:** none found in src

#### `<Points as Serialize>::serialize`  (L869-871, impl Serialize)
`fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.serialize_f64`, `f64::from`
- **Called by:** `serialize_timeout` (cli.rs:2885) (?)

#### `<Points as Deserialize>::deserialize`  (L875-883, impl Deserialize)
`fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.round`, `Err`, `Ok`, `Self`, `f64::deserialize`, `serde::de::Error::custom`
- **Macros:** `format!`
- **Called by:** none found in src

#### `Alignment::jc`  (L910-917)
`fn jc(self) -> &'static str`

- **Does:** The `w:jc` value Word writes.
- **Calls:** none
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545) (?)

#### `<HalfPoints as Serialize>::serialize`  (L960-962, impl Serialize)
`fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.serialize_f64`, `f64::from`
- **Called by:** `serialize_timeout` (cli.rs:2885) (?)

#### `<HalfPoints as Deserialize>::deserialize`  (L966-974, impl Deserialize)
`fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.round`, `Err`, `Ok`, `Self`, `f64::deserialize`, `serde::de::Error::custom`
- **Macros:** `format!`
- **Called by:** `deeper_boundary_tests::half_points_finite_range_and_rounding_boundaries` (edit.rs:8895)

#### `RunSpec::format`  (L1073-1081, pub)
`pub fn format(&self) -> RunFormat`

- **Does:** The run's formatting toggles.
- **Calls (std/external/unresolved):** `.clone`, `RunFormat::default`
- **Called by:** `emit` (edit/tracked.rs:452) (?); `Transaction::resolve_one_unreported` (edit.rs:3545) (?); `Transaction::apply_lists` (edit.rs:6303) (?); `build_paragraph` (edit.rs:8299) (?); `raster_extension` (markdown/from_docx/media.rs:77) (?); `read_picture` (markdown/write.rs:6479) (?)

#### `RevisionCounts::from_changes`  (L1149-1163, pub(crate))
`pub(crate) fn from_changes<'a>(changes: impl IntoIterator<Item = &'a Change>) -> Self`

- **Does:** The counts of `changes`, one per listed change.
- **Calls (std/external/unresolved):** `Self::default`
- **Called by:** `revision_counts` (edit/tracked.rs:404); `revision_counts` (edit.rs:2396)

#### `is_zero`  (L1239-1241)
`fn is_zero(n: &usize) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `EditReport::to_jsonl`  (L1246-1356, pub)
`pub fn to_jsonl(&self) -> String`

- **Does:** JSON lines an agent can append to its own log: `load`, one `op` per operation, `summary`.
- **Calls (crate):** `ResolvedRevisions::is_empty` (edit.rs:271) L1262
- **Calls (std/external/unresolved):** `.as_object_mut`, `.clone`, `.count`, `.enumerate`, `.expect`, `.filter`, `.insert`, `.into`, `.is_empty`, `.iter`, `.push_str`, `.push`, `.to_string`, `String::new`, `push`, `serde_json::to_value`
- **Macros:** `serde_json::json!`
- **Called by:** `run_edit` (bin/jubarte.rs:1341) (?); `publish_edit` (bin/jubarte.rs:1402) (?); `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (edit.rs:8841) (?)

#### `<EditError as fmt::Display>::fmt`  (L1404-1409, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `open_error`  (L1416-1423)
`fn open_error(error: crate::inspect::InspectError) -> EditError`

- **Does:** An unopenable source: admission refusals keep their own code (`INPUT_LIMIT`, …), anything else is `INVALID_DOCUMENT`.
- **Calls (crate):** `err` (edit.rs:1770) L1419,1421; `.code` (ambiguous, 4 candidates) L1419
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** none found in src

#### `selection_error`  (L1426-1446)
`fn selection_error(e: ChangeError) -> EditError`

- **Does:** A [`ChangeError`] of a `resolve_revisions` selection as the plan's error.
- **Calls (crate):** `err` (edit.rs:1770) L1428,1433,1434,1439,1444
- **Macros:** `format!`
- **Called by:** none found in src

#### `selected_ids`  (L1450-1468)
`fn selected_ids( listed: &[Change], filter: Option<&ChangeFilter>, ) -> Result<Vec<String>, EditError>`

- **Does:** The ids of `listed` a `resolve_revisions` side selects, in document order, a move's other side included.
- **Calls (std/external/unresolved):** `.any`, `.check`, `.clone`, `.collect`, `.filter`, `.is_some`, `.iter`, `.map_err`, `.map`, `.matches`, `.normalized`, `Ok`, `Vec::new`
- **Called by:** `lower` (edit/visible.rs:95); `resolve_selected` (edit.rs:1485)

#### `named_ids`  (L1472-1481)
`fn named_ids(listed: &[Change], filter: Option<&ChangeFilter>) -> Vec<String>`

- **Does:** The ids `filter` itself selects in `listed`, before [`selected_ids`] adds each move's other side: the ones a receipt names as resolved.
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:349) (by name) L1473
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.filter`, `.iter`, `.map`, `.matches`, `.ok`, `Vec::new`
- **Called by:** `resolve_selected` (edit.rs:1485)

#### `resolve_selected`  (L1485-1564)
`fn resolve_selected( source: &[u8], selection: Option<&ResolveRevisions>, policy: RemovedAnchorComments, ) -> Result<(Option<Vec<u8>>, ResolvedRevisions, AnchorChanges), EditError>`

- **Does:** Accept, then reject, the changes `resolve_revisions` selects: the new source (`None` when it selects nothing) and what was resolved.
- **Calls (crate):** `check_whole_changes` (changes.rs:596) L1502; `list_changes` (changes.rs:564) L1504,1541; `selected_ids` (edit.rs:1450) L1505,1510; `err` (edit.rs:1770) L1513; `resolve_for_edit` (changes.rs:909) L1524,1547; `ChangeFilter::ids` (changes.rs:330) L1526,1549; `AnchorChanges::extend` (edit/comment_settlement.rs:58) L1531,1554; `named_ids` (edit.rs:1472) L1535,1558
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.filter`, `.find`, `.flatten`, `.into_iter`, `.is_empty`, `.iter`, `.map_err`, `.to_vec`, `AnchorChanges::default`, `Err`, `Ok`, `ResolvedRevisions::default`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121); `revisions_left` (edit.rs:1571); `settle` (edit.rs:1617); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:8914)

#### `revisions_left`  (L1571-1581, pub(crate))
`pub(crate) fn revisions_left(source: &[u8], plan: &EditPlan) -> bool`

- **Does:** Whether `source` still carries a tracked change of its text (body, headers, footers, notes; a style's recorded formatting is not one) once `plan`'s `resolve_revisions` has accepted and rejected what it selects.
- **Calls (crate):** `resolve_selected` (edit.rs:1485) L1572; `list_changes` (changes.rs:564) L1579
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.is_ok_and`, `.iter`, `.ok`, `.starts_with`, `.unwrap_or`
- **Called by:** `editing_mode_conflict` (edit/flags.rs:650)

#### `replies_on_deleted_paragraphs`  (L1588-1600, pub)
`pub fn replies_on_deleted_paragraphs(source: &[u8], plan: &EditPlan) -> Vec<(u32, Option<String>)>`

- **Does:** The replies `plan` makes to a comment on a paragraph it deletes, each as the parent's id and the deleted paragraph's id.
- **Calls (crate):** `lower` (edit/visible.rs:95) L1589; `resolved` (edit.rs:2358) L1593; `Transaction::reply_on_deleted` (edit.rs:6483) (by name) L1597
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.collect`, `.filter_map`, `.len`, `.map`, `.unwrap_or`, `Vec::new`
- **Called by:** `editing_mode_conflict` (edit/flags.rs:650)

#### `settle`  (L1617-1666)
`fn settle(source: &[u8], plan: &EditPlan, probe: Opened) -> Result<Settled, EditError>`

- **Does:** Settle `source` (opened as `probe`) as `plan` asks: accept and reject what `resolve_revisions` selects, then apply `existing_revisions` to the changes left.
- **Calls (crate):** `resolve_selected` (edit.rs:1485) L1618; `Opened::open` (inspect.rs:735) L1624,1655; `changes_in` (changes.rs:643) L1629; `err` (edit.rs:1770) L1635,1653; `resolve_for_edit` (changes.rs:909) L1647
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.as_slice`, `.extend`, `.is_some`, `.iter`, `.map_err`, `.starts_with`, `.to_string`, `.to_vec`, `ChangeFilter::default`, `Err`, `Ok`
- **Called by:** `for_edit` (edit/for_edit.rs:1581) (?); `Origin::build` (edit/origin.rs:260); `Transaction::start` (edit.rs:2680); `Select::parse` (markdown/mod.rs:461) (?)

#### `load_stories`  (L1680-1724)
`fn load_stories(opened: &mut Opened) -> Result<Loaded, EditError>`

- **Does:** The body and every header, footer and notes part of `opened`, parsed into its DOM, with every addressable paragraph.
- **Calls (crate):** `Opened::story_parts` (inspect.rs:861) L1687; `err` (edit.rs:1770) L1688; `PartFs::part_string` (opc/mod.rs:636) (by name) L1689; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:458); `invalid` (markdown/from_docx/ooxml.rs:31)) L1692,1693,1698; `validate_xml` (xmllinq/parse.rs:1052) L1693; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1694; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L1695; `body_paragraph_nodes` (inspect.rs:988) L1710; `story_paragraph_nodes` (inspect.rs:1178) L1712
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.into_iter`, `.into`, `.iter`, `.map_err`, `.ok_or_else`, `.push`, `.to_string`, `Ok`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581) (?); `Origin::build` (edit/origin.rs:260); `Paragraphs::open` (edit/origin.rs:585); `written_places` (edit/origin.rs:649); `tests::split_correspondence_is_refused_by_origin` (edit/origin.rs:911); `SourceModel::new` (edit/v2_apply.rs:814) (?); `paragraph_ids` (edit.rs:1734); `Transaction::start` (edit.rs:2680)

#### `paragraph_ids`  (L1734-1768, pub)
`pub fn paragraph_ids(docx: &[u8], ids: &[&str]) -> Result<Vec<Result<String, String>>, EditError>`

- **Does:** The paragraph id ([`crate::ids::paragraph_id`]) each of `ids` names in `docx`, as `edit -p` reads it on a document without tracked changes to settle: `p3` and `header1.p1` as given, `header1.p0` → `header1`, `t0.row1.cell2[.p1]` → that cell's paragraph (`p40`).
- **Calls (crate):** `Opened::open` (inspect.rs:735) L1735; `load_stories` (edit.rs:1680) L1740; `Ids::paragraph_id` (edit/origin.rs:48) L1750
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.is_ok`, `.iter`, `.join`, `.map_err`, `.map`, `.parse`, `.rsplit_once`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** none found in src

#### `err`  (L1770-1779)
`fn err(code: &str, operation: Option<&str>, message: impl Into<String>) -> EditError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.map`, `.to_string`, `Box::default`, `Vec::new`
- **Called by:** `missing` (edit/anchor.rs:217); `v2_select` (edit/anchor.rs:296); `v2_select_all` (edit/anchor.rs:345); `AnchorChanges::records` (edit/comment_settlement.rs:78) (?); `Transaction::resolve_fill_control` (edit/controls.rs:73); `cursor_offset` (edit/for_edit.rs:503) (?); `Selection::build` (edit/for_edit.rs:547) (?); `text_selection` (edit/for_edit.rs:663) (?); `changed_selection` (edit/for_edit.rs:703) (?); `for_edit` (edit/for_edit.rs:1581) (?); `Transaction::footnotes_story` (edit/notes.rs:51) (?); `Origin::build` (edit/origin.rs:260); +67 more

#### `EditPlan::from_json`  (L1790-1792, pub)
`pub fn from_json(json: &str) -> Result<Self, EditError>`

- **Does:** Parse and validate a plan's JSON.
- **Calls (crate):** `EditPlan::from_json_with_notes` (edit.rs:1805) L1791
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `request` (bin/jubarte/edit_typed.rs:294); `control_source_boundary_tests::plan` (edit/controls.rs:827); `editing_mode_conflict_json` (edit/flags.rs:670); `tests::the_json_entry_point_reads_what_the_cli_schema_serializes` (edit/flags.rs:1150); `tests::settles_all_names_one_command_only_when_every_revision_goes_one_way` (edit/origin.rs:1172); `tests::verification_reports_only_completed_candidate_checks` (edit/prepared.rs:1172); `tests::redline_only_defects_name_the_story_and_paragraph` (edit/prepared.rs:1190); `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (edit/prepared.rs:1292); `tests::record_json_reads_the_options_the_bindings_pass` (edit/record.rs:504); `tests::thread_only_and_unchanged_plans_do_not_scan_comment_anchors` (edit/v2_apply.rs:3005); `tests::existing_comment_anchors_scan_each_paragraph_once_per_apply` (edit/v2_apply.rs:3033); `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051); +9 more

#### `EditPlan::from_json_with_notes`  (L1805-1843, pub)
`pub fn from_json_with_notes(json: &str) -> Result<(Self, Vec<String>), EditError>`

- **Does:** Parse and validate a plan's JSON, with a note for each field read as another: `anchor` as `find` on `replace`, `delete` and `comment`, and `comment` as `text` on `comment` and `reply_comment`.
- **Calls (crate):** `without_byte_order_mark` (edit.rs:1966) L1806; `plan_value` (edit.rs:1971) L1807; `err` (edit.rs:1770) L1809,1816,1831,1839; `check_plan_keys` (edit.rs:2023) L1822; `check_change_filters` (edit.rs:2045) L1823; `check_operations` (edit.rs:2072) L1824; `operation_label` (edit.rs:2016) L1836
- **Calls (std/external/unresolved):** `.as_str`, `.as_u64`, `.collect`, `.enumerate`, `.filter`, `.get_mut`, `.get`, `.into_iter`, `.map_err`, `.map`, `.to_string`, `.unwrap_or_default`, `Err`, `Ok`, `Some`, `Vec::new`, `serde_json::from_value`, `std::mem::take`, `u64::from`
- **Macros:** `format!`
- **Called by:** `request` (bin/jubarte/edit_typed.rs:294); `edit_plan` (bin/jubarte.rs:1304); `EditPlan::from_json` (edit.rs:1790); `apply_plan_json_with` (edit.rs:2284)

#### `EditPlan::to_json`  (L1846-1848, pub)
`pub fn to_json(&self) -> String`

- **Does:** Serialize the plan.
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** `prepare` (edit/prepared.rs:211)

#### `operation_schemas`  (L1854-1888, pub(crate))
`pub(crate) fn operation_schemas() -> &'static [serde_json::Value]`

- **Does:** The legacy dispatcher contract, derived from its typed Serde operations.
- **Calls (std/external/unresolved):** `.as_array`, `.clone`, `.expect`, `.for_deserialize`, `.into_generator`, `.into_root_schema_for`, `.with`, `schemars::generate::SchemaSettings::draft2020_12`, `serde_json::to_value`, `std::sync::LazyLock::new`
- **Macros:** `serde_json::json!`
- **Called by:** `profiles` (capabilities/discovery.rs:38); `tests::assert_runtime_dispatch` (capabilities/discovery.rs:359); `dispatched_operation_kinds` (edit.rs:1891); `operation_needs_keep` (edit.rs:1905); `operation_supports_keep` (edit.rs:1912); `check_operations` (edit.rs:2072)

#### `dispatched_operation_kinds`  (L1891-1901, pub)
`pub fn dispatched_operation_kinds() -> Vec<String>`

- **Does:** Names accepted by the common v1 dispatcher, in the legacy manifest order.
- **Calls (crate):** `operation_schemas` (edit.rs:1854) L1892
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.expect`, `.iter`, `.map`, `.to_string`
- **Called by:** `discovery` (capabilities/discovery.rs:225); `capabilities` (capabilities.rs:399); `check_operations` (edit.rs:2072)

#### `operation_needs_keep`  (L1905-1910, pub(crate))
`pub(crate) fn operation_needs_keep(kind: &str) -> bool`

- **Does:** The operation answers another author's revision beside it, so it runs only when the plan keeps revisions (`strike`, `restore`).
- **Calls (crate):** `operation_schemas` (edit.rs:1854) L1906
- **Calls (std/external/unresolved):** `.find`, `.is_some_and`, `.iter`
- **Called by:** `support` (capabilities/discovery.rs:117)

#### `operation_supports_keep`  (L1912-1917, pub(crate))
`pub(crate) fn operation_supports_keep(kind: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `operation_schemas` (edit.rs:1854) L1913
- **Calls (std/external/unresolved):** `.find`, `.is_some_and`, `.iter`
- **Called by:** `support` (capabilities/discovery.rs:117); `keep_note` (edit/skip.rs:335); `untracked_kind` (edit/tracked.rs:79)

#### `at_least_one`  (L1919-1926)
`fn at_least_one(fields: &[&str]) -> serde_json::Value`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.map`
- **Macros:** `json!`, `serde_json::json!`
- **Called by:** none found in src

#### `exactly_one`  (L1928-1940)
`fn exactly_one(fields: &[&str]) -> serde_json::Value`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.filter`, `.iter`, `.map`
- **Macros:** `json!`, `serde_json::json!`
- **Called by:** none found in src

#### `without_byte_order_mark`  (L1966-1968, pub(crate))
`pub(crate) fn without_byte_order_mark(json: &str) -> &str`

- **Does:** `json` without a leading byte order mark, which RFC 8259 (§8.1) lets a parser ignore: editors that save UTF-8 with one write it first.
- **Calls (std/external/unresolved):** `.strip_prefix`, `.unwrap_or`
- **Called by:** `PlanV2::from_json` (edit/v2.rs:514); `EditPlan::from_json_with_notes` (edit.rs:1805)

#### `plan_value`  (L1971-2000)
`fn plan_value(json: &str) -> Result<serde_json::Value, EditError>`

- **Does:** The plan's JSON: one value, an object, with nothing after it.
- **Calls (crate):** `.line` (ambiguous: `Seg::line` (debug/diff.rs:1226); `Output::line` (markdown/diff.rs:389)) L1981; `err` (edit.rs:1770) L1987,1990; `json_kind` (edit.rs:2003) L1995
- **Calls (std/external/unresolved):** `.column`, `.into_iter`, `.is_object`, `.is_ok`, `.is_some_and`, `.map_err`, `.next`, `.to_string`, `Err`, `Ok`, `serde_json::Deserializer::from_str`, `serde_json::from_str`
- **Macros:** `format!`
- **Called by:** `EditPlan::from_json_with_notes` (edit.rs:1805)

#### `json_kind`  (L2003-2012)
`fn json_kind(value: &serde_json::Value) -> &'static str`

- **Does:** What a JSON value is, for a message.
- **Calls:** none
- **Called by:** `plan_value` (edit.rs:1971); `check_operations` (edit.rs:2072)

#### `operation_label`  (L2016-2020)
`fn operation_label(i: usize, op: &serde_json::Value) -> String`

- **Does:** The name an error gives operation `i`: its `id`, else `op-N` as the report names it.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map_or_else`
- **Macros:** `format!`
- **Called by:** `EditPlan::from_json_with_notes` (edit.rs:1805); `check_operations` (edit.rs:2072)

#### `check_plan_keys`  (L2023-2033)
`fn check_plan_keys(plan: &serde_json::Value) -> Result<(), EditError>`

- **Does:** The plan's own fields: an unknown one names the fields and the nearest.
- **Calls (crate):** `err` (edit.rs:1770) L2026; `unknown_field` (edit.rs:2036) L2029
- **Calls (std/external/unresolved):** `.as_object`, `.as_str`, `.contains`, `.expect`, `.find`, `.keys`, `Err`, `Ok`
- **Called by:** `EditPlan::from_json_with_notes` (edit.rs:1805)

#### `unknown_field`  (L2036-2041)
`fn unknown_field(key: &str, fields: &[&str], meant: Option<&str>) -> String`

- **Does:** `unknown field "key"; did you mean "nearest"?
- **Calls (crate):** `nearest` (edit.rs:2233) L2037
- **Calls (std/external/unresolved):** `.join`, `.map`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `check_plan_keys` (edit.rs:2023)

#### `check_change_filters`  (L2045-2064)
`fn check_change_filters(plan: &serde_json::Value) -> Result<(), EditError>`

- **Does:** `resolve_revisions.accept.ids` (and `authors`, `kinds`) are arrays; a single string is the slip worth naming.
- **Calls (crate):** `err` (edit.rs:1770) L2052
- **Calls (std/external/unresolved):** `.and_then`, `.pointer`, `Err`, `Ok`
- **Macros:** `format!`, `json!`
- **Called by:** `EditPlan::from_json_with_notes` (edit.rs:1805)

#### `check_operations`  (L2072-2207)
`fn check_operations(plan: &mut serde_json::Value) -> Result<Vec<String>, EditError>`

- **Does:** Each operation's fields against its kind's schema (the table `capabilities` lists): an error names the operation, its kind, the fields the kind takes and the nearest one.
- **Calls (crate):** `dispatched_operation_kinds` (edit.rs:1891) L2081; `operation_label` (edit.rs:2016) L2084; `err` (edit.rs:1770) L2085; `json_kind` (edit.rs:2003) L2086,2099,2114; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:458); `invalid` (markdown/from_docx/ooxml.rs:31)) L2088,2094,2099,2106,2112,2126...; `operation_schemas` (edit.rs:1854) L2119; `nearest` (edit.rs:2233) L2123,2178; `meant_field` (edit.rs:2222) L2178; `index_selector` (edit.rs:2211) L2190,2201
- **Calls (std/external/unresolved):** `.and_then`, `.as_object_mut`, `.as_object`, `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.enumerate`, `.expect`, `.filter`, `.find`, `.get_mut`, `.get`, `.insert`, `.is_number`, `.is_string`, `.iter_mut`, `.iter`, `.join`, `.keys`, `.map`, `.push`, `.remove`, `.to_string`, `.unwrap_or_default`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `EditPlan::from_json_with_notes` (edit.rs:1805)

#### `index_selector`  (L2211-2216)
`fn index_selector(n: &serde_json::Value) -> String`

- **Does:** The selector a bare paragraph number meant: `{"index": n}` for a whole number from 0, else what an index is.
- **Calls (std/external/unresolved):** `.as_u64`, `.map_or_else`, `.to_string`
- **Macros:** `format!`
- **Called by:** `check_operations` (edit.rs:2072)

#### `meant_field`  (L2222-2229)
`fn meant_field(kind: &str, key: &str) -> Option<&'static str>`

- **Does:** The field a plan gives another one's name: `text` for a replacement, `comment` for a comment's text, `anchor` for `find`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `check_operations` (edit.rs:2072)

#### `nearest`  (L2233-2244)
`fn nearest<'a>(key: &str, names: &[&'a str], meant: Option<&str>) -> Option<&'a str>`

- **Does:** The name a slip most likely meant: `meant` when it is one of `names`, else the closest by edit distance within a third of the slip's length.
- **Calls (crate):** `edit_distance` (edit.rs:2247) L2240
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.copied`, `.count`, `.filter`, `.find`, `.iter`, `.map`, `.max`, `.min_by_key`, `Some`
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:98); `unknown_field` (edit.rs:2036); `check_operations` (edit.rs:2072); `tests::nearest_names_the_closest_within_a_third` (edit.rs:8663); `operations` (markdown/patch.rs:502)

#### `edit_distance`  (L2247-2262)
`fn edit_distance(a: &str, b: &str) -> usize`

- **Does:** Levenshtein distance between two names, by characters.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.enumerate`, `.iter`, `.len`, `.min`, `usize::from`
- **Called by:** `nearest` (edit.rs:2233); `tests::nearest_names_the_closest_within_a_third` (edit.rs:8663)

#### `apply_plan_json`  (L2271-2273, pub)
`pub fn apply_plan_json(source: &[u8], plan_json: &str) -> Result<EditResult, EditError>`

- **Does:** Apply a plan given as JSON: [`EditPlan::from_json_with_notes`], then [`apply_plan`]; the report's `notes` name each field read as another.
- **Calls (crate):** `apply_plan_json_with` (edit.rs:2284) L2272; `<PrepareOptions as Default>::default` (edit/prepared.rs:158) L2272
- **Called by:** `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:2174)

#### `apply_plan_json_with`  (L2284-2294, pub)
`pub fn apply_plan_json_with( source: &[u8], plan_json: &str, options: &PrepareOptions, ) -> Result<EditResult, EditError>`

- **Does:** [`apply_plan_json`] with the shared preparation's options: `require_change` and `exact_text` reach the JSON callers the CLI already exposes.
- **Calls (crate):** `EditPlan::from_json_with_notes` (edit.rs:1805) L2289; `prepare` (edit/prepared.rs:211) L2290; `Output::append` (markdown/diff.rs:402) (by name) L2291
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `apply_plan_json` (edit.rs:2271)

#### `apply_plan`  (L2322-2324, pub)
`pub fn apply_plan(source: &[u8], plan: &EditPlan) -> Result<EditResult, EditError>`

- **Does:** Resolve and apply the plan; compare source and copy into a redline.
- **Calls (crate):** `prepare` (edit/prepared.rs:211) L2323; `<PrepareOptions as Default>::default` (edit/prepared.rs:158) L2323
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `control_source_boundary_tests::public_block_controls_refuse_section_loss_and_both_nested_fill_orders` (edit/controls.rs:885); `apply_markdown` (markdown/patch.rs:121)

#### `check_update_fields`  (L2329-2338)
`fn check_update_fields(plan: &EditPlan) -> Result<(), EditError>`

- **Does:** `update_fields` refreshes the clean copy the comparer reads.
- **Calls (crate):** `err` (edit.rs:1770) L2331
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Called by:** `prepare` (edit/prepared.rs:211) (?); `resolved` (edit.rs:2358)

#### `check_plan`  (L2343-2345, pub)
`pub fn check_plan(source: &[u8], plan: &EditPlan) -> Result<(), EditError>`

- **Does:** Resolve every operation and run the plan's checks, without applying it: what `add --plan` runs on each recorded operation, so recording a long plan never re-applies it.
- **Calls (crate):** `check_plan_quoting` (edit.rs:2349) L2344
- **Called by:** none found in src

#### `check_plan_quoting`  (L2349-2356, pub)
`pub fn check_plan_quoting( source: &[u8], plan: &EditPlan, syntax: PlanSyntax, ) -> Result<(), EditError>`

- **Does:** [`check_plan`], its refusals quoting their routes in `syntax`: the flags `add --plan` records come back as flags.
- **Calls (crate):** `lower` (edit/visible.rs:95) L2354; `resolved` (edit.rs:2358) L2355
- **Calls (std/external/unresolved):** `.as_ref`, `.map`, `.unwrap_or`
- **Called by:** `record_plan` (bin/jubarte.rs:2625); `check_plan` (edit.rs:2343)

#### `resolved`  (L2358-2370)
`fn resolved<'a>( source: &'a [u8], plan: &'a EditPlan, syntax: PlanSyntax, ) -> Result<Transaction<'a>, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `check_update_fields` (edit.rs:2329) L2363; `Transaction::start` (edit.rs:2680) L2364; `Transaction::resolve` (edit.rs:3267) L2366; `check` (edit/tracked.rs:107) L2367; `Transaction::emits_directly` (edit.rs:2828) L2368
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `replies_on_deleted_paragraphs` (edit.rs:1588); `check_plan_quoting` (edit.rs:2349)

#### `preview_plan`  (L2373-2375, pub)
`pub fn preview_plan(source: &[u8], plan: &EditPlan) -> Result<EditReport, EditError>`

- **Does:** Resolve every operation without producing documents.
- **Calls (crate):** `preview_plan_with` (edit.rs:2380) L2374; `<PrepareOptions as Default>::default` (edit/prepared.rs:158) L2374
- **Called by:** `control_source_boundary_tests::public_block_controls_refuse_section_loss_and_both_nested_fill_orders` (edit/controls.rs:885); `deeper_boundary_tests::admission_boundaries_and_signature_prefix_are_exact` (edit.rs:8956); `deeper_boundary_tests::resolution_empty_text_and_combined_comment_anchors` (edit.rs:9005); `deeper_boundary_tests::header_refuses_body_only_picture_toc_and_list` (edit.rs:9066)

#### `preview_plan_with`  (L2380-2393, pub)
`pub fn preview_plan_with( source: &[u8], plan: &EditPlan, options: &PrepareOptions, ) -> Result<EditReport, EditError>`

- **Does:** [`preview_plan`] with the shared preparation's options: the dry run prepares and checks the same serialized candidate as an apply and stops before publication.
- **Calls (crate):** `prepare` (edit/prepared.rs:211) L2390
- **Calls (std/external/unresolved):** `Ok`, `Vec::new`
- **Called by:** `run_edit` (bin/jubarte.rs:1341); `preview_plan` (edit.rs:2373)

#### `revision_counts`  (L2396-2398)
`fn revision_counts(redline: &[u8]) -> RevisionCounts`

- **Does:** The comparer's redline holds this edit's changes only: all of them.
- **Calls (crate):** `RevisionCounts::from_changes` (edit.rs:1149) L2397; `list_logical_changes` (changes.rs:579) L2397
- **Calls (std/external/unresolved):** `.unwrap_or_default`
- **Called by:** `produce` (edit/prepared.rs:928) (?); `deeper_boundary_tests::invalid_package_has_no_revisions_or_kept_comment_text` (edit.rs:8998); `deeper_boundary_tests::revision_and_comment_scans_ignore_rootless_and_foreign_story_parts` (edit.rs:10145)

#### `ThreadOp::target`  (L2567-2574)
`fn target(&self) -> u32`

- **Does:** The existing comment the operation acts on.
- **Calls:** none
- **Called by:** `parse_subcommand_json` (cli.rs:3779) (?); `with_operation_targets` (edit/v2_apply.rs:97) (?); `compose_threads` (edit/v2_apply.rs:1821) (?); `Transaction::resolve_thread` (edit.rs:4379) (?); `Transaction::check_thread_ops` (edit.rs:5715) (?); `run` (update.rs:225) (?)

#### `Transaction::start`  (L2680-2812)
`fn start(source: &[u8], plan: &'p EditPlan) -> Result<Self, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1770) L2682,2692,2699,2709,2771; `source_sha256` (inspect.rs:299) L2688,2729; `check_keep` (edit/visible.rs:71) L2701; `Opened::open` (inspect.rs:735) L2702,2726; `PartFs::parts` (opc/mod.rs:733) (by name) L2703; `existing_comment_ids` (edit.rs:6960) L2716,2765; `settle` (edit.rs:1617) L2723; `normalize_permission_ids` (edit/ranges.rs:374) L2724; `AnchorChanges::records` (edit/comment_settlement.rs:78) (by name) L2728; `load_stories` (edit.rs:1680) L2734; `Origin::build` (edit/origin.rs:260) L2739; `project_paragraph` (inspect.rs:1265) L2750; `body_control_nodes` (inspect.rs:997) L2768; `collect_controls` (inspect.rs:1005) L2769; `CommentFamily::load` (comments.rs:457) L2770
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.clone`, `.collect`, `.ends_with`, `.filter_map`, `.is_empty`, `.iter`, `.map_err`, `.map_or`, `.map`, `.max`, `.next`, `.split_whitespace`, `.starts_with`, `.to_uppercase`, `.trim`, `.unwrap_or_else`, `BTreeMap::new`, `Err`, `Ok`, `PlanSyntax::default`, `Some`, `Vec::new`, `std::cell::Cell::new`, `std::cell::RefCell::new`, `std::collections::BTreeSet::new`, `u64::from`
- **Macros:** `format!`
- **Called by:** `prepare` (edit/prepared.rs:211); `tests::redline_only_defects_name_the_story_and_paragraph` (edit/prepared.rs:1190); `redline` (edit/tracked.rs:283); `apply_bound` (edit/v2_apply.rs:121); `tests::thread_only_and_unchanged_plans_do_not_scan_comment_anchors` (edit/v2_apply.rs:3005); `tests::existing_comment_anchors_scan_each_paragraph_once_per_apply` (edit/v2_apply.rs:3033); `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051); `tests::comment_inventory_preserves_first_complete_ranges_and_offsets` (edit/v2_apply.rs:3072); `resolved` (edit.rs:2358); `Transaction::commented_base` (edit.rs:6518); `deeper_boundary_tests::a_retried_insertion_point_leaves_no_refusal` (edit.rs:8818); `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (edit.rs:8841); +6 more

#### `Transaction::emits_directly`  (L2828-2862)
`fn emits_directly(&self) -> Result<bool, EditError>`

- **Does:** Whether the redline is emitted directly, as under keep: the comparer moves a comment's range off text an edit replaces whole, so a plan that does is emitted, unless it holds an operation the emitter cannot track (a footnote, an image).
- **Calls (crate):** `emits_all` (edit/tracked.rs:85) L2829; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L2835; `in_text_box` (inspect.rs:962) L2841; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L2842; `Transaction::edits_a_comment_range` (edit.rs:2913) L2851; `err` (edit.rs:1770) L2853; `Transaction::edits_beside_a_range_marker` (edit.rs:2872) L2861
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `Err`, `Ok`, `Some`, `W::tbl`, `crate::namespaces::MC::name`
- **Called by:** `prepare` (edit/prepared.rs:211); `resolved` (edit.rs:2358)

#### `Transaction::edits_beside_a_range_marker`  (L2872-2893)
`fn edits_beside_a_range_marker(&self) -> bool`

- **Does:** Whether a text edit writes new words beside a range marker: one in a word the edit touches (the comparer rewrites whole words), or one at body level next to the paragraph's first or last character.
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493) L2873; `word_window` (edit.rs:7375) L2883; `Transaction::range_marks` (edit.rs:2970) L2884; `Transaction::body_level_marker_beside` (edit.rs:2897) L2888,2889
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.is_empty`, `.iter`, `.len`, `.starts_with`
- **Called by:** `prepare` (edit/prepared.rs:211); `Transaction::emits_directly` (edit.rs:2828)

#### `Transaction::body_level_marker_beside`  (L2897-2909)
`fn body_level_marker_beside(&self, para: usize, before: bool) -> bool`

- **Does:** Whether a range marker sits at body level right before (or after) paragraph `para`.
- **Calls (crate):** `Dom::nodes_before_self` (xmllinq/mod.rs:846) (by name) L2901; `Dom::is_element` (xmllinq/mod.rs:502) (by name) L2904; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L2906; `range_marker` (edit.rs:7393) L2908
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.is_some`, `.rev`
- **Called by:** `Transaction::edits_beside_a_range_marker` (edit.rs:2872)

#### `Transaction::edits_a_comment_range`  (L2913-2932)
`fn edits_a_comment_range(&self) -> bool`

- **Does:** Whether the plan names a comment's text (`cN.text`), or one of its text edits replaces exactly the text a comment covers with new text.
- **Calls (crate):** `selector_of` (edit.rs:7032) L2918; `Transaction::comment_ranges` (edit.rs:2936) L2929
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.contains`, `.is_empty`, `.is_some`, `.iter`
- **Called by:** `Transaction::emits_directly` (edit.rs:2828)

#### `Transaction::comment_ranges`  (L2936-2947)
`fn comment_ranges(&self, para: usize) -> Vec<(usize, usize)>`

- **Does:** The `(start, end)` projection offsets of each comment whose range opens and closes in paragraph `para`.
- **Calls (crate):** `Transaction::comment_marks` (edit.rs:2951) L2939
- **Calls (std/external/unresolved):** `.iter`, `.position`, `.push`, `.remove`, `Vec::new`
- **Called by:** `Transaction::edits_a_comment_range` (edit.rs:2913)

#### `Transaction::comment_marks`  (L2951-2965)
`fn comment_marks(&self, para: usize) -> Vec<(String, bool, usize)>`

- **Does:** Each comment range marker of paragraph `para`, in document order: its `w:id`, whether it opens the range, and its projection offset.
- **Calls (crate):** `Transaction::range_marks` (edit.rs:2970) L2954
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.get`, `.into_iter`, `.set`, `.with`, `Some`
- **Called by:** `Transaction::comment_ranges` (edit.rs:2936); `Transaction::comment_anchors` (edit.rs:2997); `Transaction::comment_text` (edit.rs:3025)

#### `Transaction::range_marks`  (L2970-2993)
`fn range_marks(&self, para: usize) -> Vec<(&'static str, Option<String>, usize)>`

- **Does:** Each range marker of paragraph `para` (see [`range_marker`]), in document order: its local name, its `w:id`, and its projection offset.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L2982; `range_marker` (edit.rs:7393) L2987; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L2988
- **Calls (std/external/unresolved):** `.entry`, `.get`, `.map`, `.or_insert`, `.push`, `BTreeMap::new`, `Vec::new`, `W::id`
- **Called by:** `Transaction::edits_beside_a_range_marker` (edit.rs:2872); `Transaction::comment_marks` (edit.rs:2951)

#### `Transaction::comment_anchors`  (L2997-3021)
`fn comment_anchors(&self) -> BTreeMap<String, Covered>`

- **Does:** Every complete range in one document pass.
- **Calls (crate):** `Transaction::comment_marks` (edit.rs:2951) L3001
- **Calls (std/external/unresolved):** `.contains_key`, `.entry`, `.insert`, `.len`, `.or_insert`, `.remove`, `BTreeMap::new`
- **Called by:** `check_comment_anchors` (edit/v2_apply.rs:1913); `tests::comment_inventory_preserves_first_complete_ranges_and_offsets` (edit/v2_apply.rs:3072) (?)

#### `Transaction::comment_text`  (L3025-3052)
`fn comment_text(&self, n: u32) -> Result<Covered, origin::Miss>`

- **Does:** The text comment `n` covers (`cN.text`), from its range start to its range end.
- **Calls (crate):** `Transaction::comment_marks` (edit.rs:2951) L3029
- **Calls (std/external/unresolved):** `.into`, `.len`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::comment_inventory_preserves_first_complete_ranges_and_offsets` (edit/v2_apply.rs:3072) (?); `Transaction::resolve_comment_text` (edit.rs:3057); `Transaction::resolve` (edit.rs:3267); `Transaction::select` (edit.rs:4980)

#### `Transaction::resolve_comment_text`  (L3057-3159)
`fn resolve_comment_text( &self, n: u32, new_text: &str, outcome: &mut EditOutcome, ) -> Result<Vec<Resolved>, (String, String)>`

- **Does:** `cN.text` rewritten as `new_text` (empty: deleted): one tracked substitution inside the comment.
- **Calls (crate):** `Transaction::comment_text` (edit.rs:3025) L3063; `Transaction::paragraph_id` (edit.rs:5085) L3073; `short_paragraph` (ids.rs:435) L3079,3113,3114; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:5085)) L3079,3113,3114; `check_run_text` (edit.rs:7107) L3083; `Transaction::check_range` (edit.rs:5160) L3088,3118,3120; `context` (edit.rs:7147) L3090,3122; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L3109; `structure` (markdown/from_docx/mod.rs:4460) (by name) L3111
- **Calls (std/external/unresolved):** `.into`, `.len`, `.map_err`, `.push`, `.to_string`, `Err`, `Ok`, `Some`, `String::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `Transaction::resolve_rewrite` (edit.rs:4608)

#### `Transaction::report`  (L3161-3209)
`fn report(&self, ok: bool) -> EditReport`

- **Does:** _no doc comment_
- **Calls (crate):** `Origin::moves` (edit/origin.rs:484) (by name) L3194; `Transaction::paragraph_id` (edit.rs:5085) L3194; `Transaction::body_paragraph_count` (edit.rs:5052) L3198,3199; `Transaction::rewrite_notes` (edit.rs:3223) L3206
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter_map`, `.is_some`, `.iter`, `.map`, `.ok`, `.parse`, `.push`, `.strip_prefix`, `.unwrap_or_default`, `RevisionCounts::default`, `Vec::new`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `produce` (edit/prepared.rs:928); `result_v2_with_paragraphs` (edit/tracked.rs:231); `result_with_order` (edit/tracked.rs:252); `apply_bound` (edit/v2_apply.rs:121); `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (edit.rs:8841) (?)

#### `Transaction::skip_comment_ids_of`  (L3213-3219)
`fn skip_comment_ids_of(&mut self, source: &[u8]) -> Result<(), EditError>`

- **Does:** New comments take ids above every comment `source` defines, as [`Self::start`] does when it settles the source itself.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L3214; `existing_comment_ids` (edit.rs:6960) L3215
- **Calls (std/external/unresolved):** `.map_err`, `.max`, `Ok`, `u64::from`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `Transaction::rewrite_notes`  (L3223-3238)
`fn rewrite_notes(&self) -> Vec<String>`

- **Does:** One note per comment of another author an `edit_comment` rewrote under `rewrite_others_comments`: their name now carries our words.
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.get`, `.is_empty`, `.iter`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::report` (edit.rs:3161)

#### `Transaction::note_rewrites`  (L3243-3255, pub(crate))
`pub(crate) fn note_rewrites(&mut self, outcome: usize, resolved: &[Resolved])`

- **Does:** Record each `edit_comment` among `resolved` (the operation whose outcome will be `outcomes[outcome]`) that changes a comment not the plan author's, for [`Self::rewrite_notes`]: v1 and v2 alike.
- **Calls (crate):** `Transaction::not_own_comment` (edit.rs:3260) L3250
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `resolve` (edit/v2_apply.rs:1656); `Transaction::resolve` (edit.rs:3267)

#### `Transaction::not_own_comment`  (L3260-3263)
`fn not_own_comment(&self, family: &crate::comments::CommentFamily, id: u32) -> Option<String>`

- **Does:** The author of comment `id` when it is not the plan author's, `""` for one that names no author: only a comment that names the plan's author is its own (#1264).
- **Calls (crate):** `CommentFamily::author` (comments.rs:562) L3261
- **Calls (std/external/unresolved):** `.then`, `.to_string`, `.unwrap_or_default`
- **Called by:** `Transaction::note_rewrites` (edit.rs:3243); `Transaction::resolve_thread` (edit.rs:4379)

#### `Transaction::resolve`  (L3267-3477)
`fn resolve(&mut self) -> Result<(), EditError>`

- **Does:** Resolve every operation against the untouched source; the first failure is returned after every operation has been tried.
- **Calls (crate):** `kind_name` (edit.rs:6990) L3272; `selector_of` (edit.rs:7032) L3273; `Transaction::comment_text` (edit.rs:3025) L3276; `Transaction::resolve_rewrite` (edit.rs:4608) L3288,3294; `comment_text_of` (edit.rs:7024) L3293; `Transaction::resolve_list` (edit.rs:4730) L3301; `Transaction::resolve_page_setup` (edit/sections.rs:363) L3324; `err` (edit.rs:1770) L3336; `Transaction::resolve_thread` (edit.rs:4379) L3343; `Transaction::resolve_watermark` (edit/watermark.rs:174) L3351; `Transaction::resolve_fill_control` (edit/controls.rs:73) L3363; `Transaction::resolve_settings` (edit.rs:5791) L3383; `Transaction::resolve_visible` (edit/visible.rs:348) L3394; `Transaction::resolve_one` (edit.rs:3479) L3396; `Transaction::replace_pieces` (edit.rs:4549) L3409; `Transaction::note_rewrites` (edit.rs:3243) L3441; `Transaction::attach_revision_refusal` (edit/refusal.rs:195) L3448; `Transaction::check_conflicts` (edit.rs:5402) L3475
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.chain`, `.clone`, `.collect`, `.enumerate`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.into_vec`, `.into`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.push`, `.set`, `.take`, `.to_string`, `.unwrap_or_else`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`, `vec!`
- **Called by:** `tests::resolve_keeps_quoted_css_list_intact` (convert/font.rs:4187) (?); `tests::resolve_quoted_cambria_still_finds_cambria` (convert/font.rs:4208) (?); `tests::resolve_dejavu_sans_mono_follows_word_subst_verdana` (convert/font.rs:4221) (?); `tests::resolve_unknown_family_is_cambria_not_calibri` (convert/font.rs:4231) (?); `tests::resolve_wide_latin_stays_calibri_mini_505` (convert/font.rs:4262) (?); `tests::resolve_empty_family_is_times_new_roman` (convert/font.rs:4272) (?); `tests::resolve_prefers_embedded_unknown_family` (convert/font.rs:4798) (?); `tests::resolve_embedded_does_not_steal_unrelated_families` (convert/font.rs:4838) (?); `tests::font_table_scope_is_restored_after_a_panicking_conversion` (convert/font.rs:5231) (?); `tests::nested_font_table_scope_restores_outer_table` (convert/font.rs:5328) (?); `chrome_part_xml` (convert/mod.rs:20786) (?); `hf_pic_trails_line` (convert/mod.rs:21893) (?); +48 more

#### `Transaction::resolve_one`  (L3479-3543)
`fn resolve_one( &self, id: &str, kind: &OperationKind, ) -> Result<(Resolved, EditOutcome), Box<(EditError, EditOutcome)>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::resolve_one_unreported` (edit.rs:3545) L3486,3541; `redaction` (edit.rs:7143) L3497; `scrub` (scrub.rs:109) L3509,3536; `short_paragraph` (ids.rs:435) L3514; `err` (edit.rs:1770) L3535
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.is_some`, `.map_or_else`, `.map`, `.take`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `resolve_operation` (edit/v2_apply.rs:2084); `tests::existing_comment_anchors_scan_each_paragraph_once_per_apply` (edit/v2_apply.rs:3033) (?); `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051) (?); `Transaction::resolve` (edit.rs:3267)

#### `Transaction::resolve_one_unreported`  (L3545-4369)
`fn resolve_one_unreported( &self, id: &str, kind: &OperationKind, ) -> Result<(Resolved, EditOutcome), Box<(EditError, EditOutcome)>>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1770) L3565; `Transaction::select` (edit.rs:4980) L3636,3894,4131; `finds_text` (edit.rs:7081) L3636; `Transaction::paragraph_id` (edit.rs:5085) L3643; `comment_text_of` (edit.rs:7024) L3644; `short_paragraph` (ids.rs:435) L3659; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:5085)) L3659,3906,3914,3915; `check_run_text` (edit.rs:7107) L3720,3813,4140; `check_format` (edit.rs:8453) L3723,3815,4142; `check_comment` (edit.rs:7134) L3727,3779,3819,3885,3932,3970...; `Transaction::find_range` (edit.rs:5130) L3729,3754,3781,3830,3836; `context` (edit.rs:7147) L3732,3784,3865,3951,4011,4036; `redaction` (edit.rs:7143) L3757; `Transaction::check_insert_position` (edit.rs:5381) L3863; `locate` (edit.rs:7203) L3936; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L3974; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L3981,4271; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L3982,3991; `Transaction::story_container` (edit.rs:5112) L3990; `excerpt` (edit.rs:7161) L3999,4040,4079,4156,4215,4243...; `Transaction::resolve_format_run` (edit/runs.rs:17) L4008; `Transaction::resolve_footnote` (edit/notes.rs:24) L4033; `picture` (edit/images.rs:86) L4067; `.format` (ambiguous: `ListKind::format` (edit.rs:822); `RunSpec::format` (edit.rs:1073); `Writer::format` (markdown/write.rs:3254)) L4142; `Transaction::resolve_style` (edit.rs:4884) L4150,4193; `.jc` (ambiguous: `Alignment::jc` (edit.rs:910); `Styles::jc` (markdown/from_docx/mod.rs:1883)) L4203; `check_text` (edit.rs:7094) L4229,4344; `Transaction::merge_partner` (edit.rs:4859) L4230; `check_rows` (edit/structural.rs:28) L4282; `text_width` (edit/structural.rs:70) L4287; `resolve_table_style` (edit/structural.rs:119) L4292; `table_styles` (edit/structural.rs:89) L4293
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.as_str`, `.chars`, `.clone`, `.collect`, `.contains`, `.first`, `.get`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.join`, `.len`, `.map_err`, `.map_or`, `.map`, `.push`, `.rev`, `.take`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Box::new`, `Err`, `Ok`, `Some`, `Spacing::default`, `String::new`, `Vec::new`, `W::p`, `W::sect_pr`, `W::tc`, `fail`, `from`, `u32::try_from`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Transaction::resolve_one` (edit.rs:3479)

#### `Transaction::resolve_thread`  (L4379-4511)
`fn resolve_thread(&self, id: &str, kind: &OperationKind) -> Resolution<Resolved>`

- **Does:** The paragraph `para` merges with: the next sibling paragraph, with only range markup (bookmarks, comment ranges, ...) between them.
- **Calls (crate):** `err` (edit.rs:1770) L4395; `.target` (ambiguous: `EditOptions::target` (cli.rs:358); `Operation::target` (edit/v2.rs:585); `ThreadOp::target` (edit.rs:2567)) L4419; `Origin::had_comment` (edit/origin.rs:252) (by name) L4422; `check_comment` (edit.rs:7134) L4446; `Transaction::not_own_comment` (edit.rs:3260) L4451; `CommentFamily::with_replies` (comments.rs:597) (by name) L4475; `Transaction::comment_markers` (edit.rs:4515) L4478; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L4480; `Transaction::paragraph_id` (edit.rs:5085) L4494
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.contains`, `.expect`, `.find_map`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.join`, `.map_err`, `.map`, `.position`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::new`, `W::p`, `fail`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `resolve_operation` (edit/v2_apply.rs:2084); `Transaction::resolve` (edit.rs:3267); `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (edit.rs:9595) (?)

#### `Transaction::comment_markers`  (L4515-4540)
`fn comment_markers(&self, ids: &[u32], target: u32) -> (Vec<usize>, Option<NodeId>)`

- **Does:** Stories holding markers of the comments `ids`, and the reference element of comment `target`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L4523; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L4524
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.enumerate`, `.is_none`, `.iter`, `.map`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::id`, `W::name`
- **Called by:** `Transaction::resolve_thread` (edit.rs:4379); `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (edit.rs:9595) (?)

#### `Transaction::replace_pieces`  (L4549-4604)
`fn replace_pieces(&self, resolved: Resolved, outcome: &mut EditOutcome) -> Vec<Resolved>`

- **Does:** A `replace` as the word-level edits that make its matched text read as the replacement (plan 12): kept words stay in their runs, with their formatting and the document's characters (a curly quote the anchor typed straight), and only the words that change are written, a new one formatted like the text before it.
- **Calls (crate):** `replace_ranges` (edit/rewrite.rs:84) L4565; `Transaction::check_insert_either` (edit.rs:5364) L4572; `Transaction::check_range` (edit.rs:5160) L4574
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.into`, `.is_empty`, `.is_err`, `.is_some_and`, `.next_back`, `.push`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Transaction::resolve` (edit.rs:3267)

#### `Transaction::resolve_rewrite`  (L4608-4726)
`fn resolve_rewrite( &self, id: &str, paragraph: &Selector, new_text: &str, comment: Option<&str>, ) -> Resolution<Vec<Resolved>>`

- **Does:** `rewrite`: the smallest word-level edits that make the paragraph read as `new_text`, each checked as a `replace` or an `insert` is.
- **Calls (crate):** `err` (edit.rs:1770) L4630; `check_comment` (edit.rs:7134) L4633; `comment_text_of` (edit.rs:7024) L4635; `Transaction::resolve_comment_text` (edit.rs:3057) L4645; `Transaction::select` (edit.rs:4980) L4650; `Transaction::paragraph_id` (edit.rs:5085) L4657; `check_run_text` (edit.rs:7107) L4667; `ChangeFilter::matches` (changes.rs:405) (by name) L4670; `rewrite_ranges` (edit/rewrite.rs:77) L4682; `Transaction::check_insert_either` (edit.rs:5364) L4699; `Transaction::check_range` (edit.rs:5160) L4701; `excerpt` (edit.rs:7161) L4724
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.count`, `.into`, `.is_empty`, `.is_some_and`, `.is_some`, `.len`, `.map_err`, `.next_back`, `.push`, `.replace`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::with_capacity`, `fail`, `symbols`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve` (edit.rs:3267)

#### `Transaction::resolve_list`  (L4730-4857)
`fn resolve_list( &self, id: &str, selectors: &[Selector], kind: ListKind, level: u32, restart: bool, ) -> Resolution<Resolved>`

- **Does:** `list`: every selector matches one body paragraph, once; `restart: false` finds the list to continue.
- **Calls (crate):** `err` (edit.rs:1770) L4753; `Transaction::select` (edit.rs:4980) L4771; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:5085)) L4781,4790; `Transaction::paragraph_id` (edit.rs:5085) L4801; `direct_num_id` (edit/structural.rs:422) L4811; `Transaction::paragraph_styles` (edit.rs:4903) L4825; `excerpt` (edit.rs:7161) L4844
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.eq_ignore_ascii_case`, `.find_map`, `.find`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_or`, `.map`, `.min`, `.name`, `.or_else`, `.push`, `.rev`, `.to_string`, `.unwrap_or`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::with_capacity`, `fail`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve` (edit.rs:3267)

#### `Transaction::merge_partner`  (L4859-4881)
`fn merge_partner(&self, para: usize) -> Result<usize, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::next_element` (xmllinq/mod.rs:855) (by name) L4862,4864; `is_range_markup` (edit.rs:6976) L4863; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L4867; `Dom::element` (xmllinq/mod.rs:655) (by name) L4876,4877
- **Calls (std/external/unresolved):** `.filter`, `.into`, `.is_some_and`, `.is_some`, `.iter`, `.ok_or_else`, `.position`, `.to_string`, `Err`, `Ok`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545)

#### `Transaction::resolve_style`  (L4884-4900)
`fn resolve_style(&self, requested: &str) -> Result<String, String>`

- **Does:** A paragraph style id from its id or its name (case-insensitive).
- **Calls (crate):** `Transaction::paragraph_styles` (edit.rs:4903) L4885
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.eq_ignore_ascii_case`, `.find`, `.iter`, `.join`, `.map`, `.take`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545)

#### `Transaction::paragraph_styles`  (L4903-4928)
`fn paragraph_styles(&self) -> Vec<(String, String)>`

- **Does:** `(styleId, name)` of every paragraph style in the styles part.
- **Calls (crate):** `Opened::related` (inspect.rs:902) L4904; `PartFs::part_string` (opc/mod.rs:636) L4907; `Dom::new` (xmllinq/mod.rs:387) L4910; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L4911; `Dom::root` (xmllinq/mod.rs:822) L4912; `Dom::elements` (xmllinq/mod.rs:640) L4915; `Dom::attribute` (xmllinq/mod.rs:874) L4917,4919,4922; `Dom::element` (xmllinq/mod.rs:655) L4920
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.next`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::val`
- **Called by:** `Transaction::resolve_list` (edit.rs:4730); `Transaction::resolve_style` (edit.rs:4884); `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (edit.rs:9102) (?)

#### `Transaction::ids`  (L4931-4939)
`fn ids(&self) -> origin::Ids<'_>`

- **Does:** What this plan's own ids resolve against.
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `Transaction::cell_control` (edit/controls.rs:303); `Transaction::select` (edit.rs:4980)

#### `Transaction::text_selector`  (L4945-4973)
`fn text_selector(&self, global: usize) -> Option<String>`

- **Does:** A plan's selector that reaches paragraph `global` by its text: its first words, or all of it, as `{"contains": …}` (with its story outside the body), when no other paragraph of its story holds them.
- **Calls (crate):** `unique` (edit/v2.rs:474) (by name) L4966
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.clone`, `.collect`, `.contains`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.len`, `.ok`, `.rfind`, `.take`, `.then`, `.to_string`, `.trim_end`, `.trim`, `serde_json::to_string`
- **Called by:** `Transaction::select` (edit.rs:4980)

#### `Transaction::select`  (L4980-5050)
`fn select(&self, selector: &Selector, merged_ok: bool) -> Result<usize, origin::Miss>`

- **Does:** The paragraph `selector` names.
- **Calls (crate):** `comment_text_of` (edit.rs:7024) L4981; `Transaction::comment_text` (edit.rs:3025) L4982; `.ids` (ambiguous, 6 candidates) L4985; `Origin::follow` (edit/origin.rs:418) (by name) L4987; `Transaction::paragraph_id` (edit.rs:5085) L4990; `Transaction::text_selector` (edit.rs:4945) L4991; `Transaction::ids` (edit.rs:4931) L4995,4997; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:5085)) L5043
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.position`, `.starts_with`, `Err`, `Ok`
- **Macros:** `format!`, `unreachable!`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::resolve_rewrite` (edit.rs:4608); `Transaction::resolve_list` (edit.rs:4730)

#### `Transaction::body_paragraph_count`  (L5052-5054)
`fn body_paragraph_count(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`
- **Called by:** `Transaction::report` (edit.rs:3161)

#### `Transaction::touched_stories`  (L5057-5082)
`fn touched_stories(&self) -> std::collections::BTreeSet<usize>`

- **Does:** Stories an operation of the plan edits.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.flat_map`, `.iter`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Transaction::finish` (edit.rs:6645)

#### `Transaction::paragraph_id`  (L5085-5088)
`fn paragraph_id(&self, para: usize) -> String`

- **Does:** The id of a paragraph ([`crate::ids::paragraph_id`]).
- **Calls (crate):** `paragraph_id` (ids.rs:410) L5087
- **Called by:** `Package::heading_skip` (audit.rs:532) (?); `Package::literal_bullet` (audit.rs:636) (?); `Package::empty_spacers` (audit.rs:661) (?); `Transaction::cell_control` (edit/controls.rs:303) (?); `resolve` (edit/v2_apply.rs:1656); `resolve_operation` (edit/v2_apply.rs:2084); `Transaction::resolve_visible` (edit/visible.rs:348); `Transaction::resolve_comment_text` (edit.rs:3057); `Transaction::report` (edit.rs:3161); `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::resolve_thread` (edit.rs:4379); `Transaction::resolve_rewrite` (edit.rs:4608); +4 more

#### `Transaction::paragraph_walk`  (L5097-5108, pub(super))
`pub(super) fn paragraph_walk(&self) -> Vec<NodeId>`

- **Does:** Every paragraph of the transaction's current DOM, in the fixed projection order the expectations checker reopens bytes in: the body's paragraphs in body order, then each story's own, story by story (the same collectors `load_stories` numbers paragraphs with).
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L5102; `story_paragraph_nodes` (inspect.rs:1178) L5104
- **Calls (std/external/unresolved):** `.enumerate`, `.extend`, `.iter`, `Vec::new`
- **Called by:** `redline` (edit/tracked.rs:283); `apply_bound` (edit/v2_apply.rs:121)

#### `Transaction::story_container`  (L5112-5125)
`fn story_container(&self, node: NodeId) -> (NodeId, &'static str)`

- **Does:** The body, header, footer or note a paragraph belongs to, for the "keeps one paragraph" checks.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L5120
- **Calls (std/external/unresolved):** `.first`, `Some`, `W::name`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::check_deletions_leave_valid_containers` (edit.rs:5678)

#### `Transaction::find_range`  (L5130-5155)
`fn find_range( &self, projection: &Projection, find: &str, occurrence: Option<usize>, outcome: &mut EditOutcome, ) -> Result<(usize, usize), (String, String)>`

- **Does:** The unique occurrence of `find`, or its `occurrence`-th hit (1-based) when given (overlapping occurrences count), checked to lie within editable direct text.
- **Calls (crate):** `locate` (edit.rs:7203) L5137; `Transaction::removed_anchor` (edit.rs:5344) L5145; `Transaction::check_range` (edit.rs:5160) L5152
- **Calls (std/external/unresolved):** `.get`, `.into`, `.map_err`, `.to_string`, `Err`, `Ok`
- **Called by:** `Transaction::resolve_footnote` (edit/notes.rs:24); `Transaction::resolve_format_run` (edit/runs.rs:17); `Transaction::resolve_one_unreported` (edit.rs:3545)

#### `Transaction::check_range`  (L5160-5188)
`fn check_range(&self, projection: &Projection, start: usize, end: usize) -> Result<(), String>`

- **Does:** Every byte of `[start, end)` must come from a direct run or an editable insertion: text, a tab, break or non-breaking hyphen, which an edit over it removes.
- **Calls (crate):** `editable_insertion` (edit.rs:7316) L5166; `Transaction::refuse_opaque` (edit.rs:5196) L5167; `run_of` (edit.rs:8027) L5167; `is_symbol` (edit.rs:7343) L5169
- **Calls (std/external/unresolved):** `.any`, `.into`, `.is_none`, `.iter`, `.max`, `Err`, `Ok`
- **Called by:** `text_component` (edit/v2_apply.rs:2269); `Transaction::resolve_comment_text` (edit.rs:3057); `Transaction::replace_pieces` (edit.rs:4549); `Transaction::resolve_rewrite` (edit.rs:4608); `Transaction::find_range` (edit.rs:5130); `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (edit.rs:9129) (?)

#### `Transaction::refuse_opaque`  (L5196-5207)
`fn refuse_opaque(&self, run: NodeId, subject: &str) -> String`

- **Does:** Why `subject` (the text, the insertion point) in `run` cannot be edited, and the way around it: another author's move or deletion by id, author and text, with each route around it (#1108); else the innermost hyperlink, field or content control.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L5198; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L5201; `Held::of` (edit/hint.rs:58) L5202; `Transaction::revision_hint` (edit.rs:5259) L5204; `Transaction::opaque_structure` (edit.rs:5211) (by name) L5205
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.take_while`, `W::p`
- **Macros:** `format!`
- **Called by:** `Transaction::check_range` (edit.rs:5160); `Transaction::check_insert_position` (edit.rs:5381)

#### `Transaction::opaque_structure`  (L5211-5253)
`fn opaque_structure(&self, run: NodeId) -> String`

- **Does:** The innermost hyperlink, field or content control that holds `run`, and the way around it.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L5213; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L5216,5223,5227,5235; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L5222,5239; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:641)) L5230; `Dom::element` (xmllinq/mod.rs:655) (by name) L5237,5238
- **Calls (std/external/unresolved):** `.and_then`, `.into`, `.is_some`, `.map`, `.or_else`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `Some`, `String::new`, `W::name`, `W::p`, `W::val`, `property`
- **Macros:** `format!`
- **Called by:** `Transaction::refuse_opaque` (edit.rs:5196) (?)

#### `Transaction::revision_hint`  (L5259-5339)
`fn revision_hint(&self, revision: NodeId, lead: &str) -> String`

- **Does:** The refusal for an edit `revision` holds (a `w:ins`, `w:del`, `w:moveTo`, `w:moveFrom`, `w:pPrChange` or `w:rPrChange`): `lead` says what was refused up to the revision's id, then the revision and its routes in the plan's syntax.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L5261; `RevisionRefusal::new` (edit/refusal.rs:51) L5267; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L5269,5302,5311; `ChangeId::short` (ids.rs:155) (by name) L5271,5333; `ChangeId::new` (ids.rs:142) L5271; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5272; `revision_long_id` (edit/v2_apply.rs:780) L5273; `Held::of` (edit/hint.rs:58) L5279; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L5286,5294,5299,5312; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L5291,5307; `Transaction::paragraph_id` (edit.rs:5085) L5301; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L5312; `restorable` (edit/visible.rs:257) L5315; `parse_change_ids` (ids.rs:232) L5332; `restored_here_hint` (edit/hint.rs:251) L5336; `settle_hint` (edit/hint.rs:211) L5338
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_str`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_ok_and`, `.is_some_and`, `.is_some`, `.iter`, `.map_or_else`, `.map_or`, `.map`, `.name`, `.position`, `.replace`, `.to_string`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `.value`, `Some`, `W::author`, `W::del_text`, `W::id`, `W::name`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`
- **Called by:** `check` (edit/tracked.rs:107); `Transaction::refuse_opaque` (edit.rs:5196); `Transaction::removed_anchor_in` (edit.rs:5354)

#### `Transaction::removed_anchor`  (L5344-5351)
`fn removed_anchor(&self, projection: &Projection, find: &str) -> Option<String>`

- **Does:** When `find` misses `projection`'s visible text but reaches into text another author deleted or moved away, the refusal that names that revision; the anchor is otherwise just not there.
- **Calls (crate):** `Transaction::removed_anchor_in` (edit.rs:5354) L5350
- **Calls (std/external/unresolved):** `.iter`, `.position`, `std::ptr::eq`
- **Called by:** `Transaction::find_range` (edit.rs:5130)

#### `Transaction::removed_anchor_in`  (L5354-5358)
`fn removed_anchor_in(&self, para: usize, find: &str) -> Option<String>`

- **Does:** [`Self::removed_anchor`] for paragraph `para`.
- **Calls (crate):** `removed_holder` (edit/hint.rs:144) L5356; `Transaction::revision_hint` (edit.rs:5259) L5357
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `resolve_operation` (edit/v2_apply.rs:2084); `Transaction::removed_anchor` (edit.rs:5344)

#### `Transaction::check_insert_either`  (L5364-5379)
`fn check_insert_either( &self, projection: &Projection, pos: usize, attach_before: &mut bool, ) -> Result<(), String>`

- **Does:** [`Self::check_insert_position`] on the side `attach_before` names, else on the other, which `attach_before` then names.
- **Calls (crate):** `Transaction::check_insert_position` (edit.rs:5381) L5370,5372
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `.or_else`, `.take`
- **Called by:** `Transaction::replace_pieces` (edit.rs:4549); `Transaction::resolve_rewrite` (edit.rs:4608); `deeper_boundary_tests::a_retried_insertion_point_leaves_no_refusal` (edit.rs:8818) (?)

#### `Transaction::check_insert_position`  (L5381-5400)
`fn check_insert_position( &self, projection: &Projection, pos: usize, attach_before: bool, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `attach_segment` (edit.rs:7348) L5391; `editable_insertion` (edit.rs:7316) L5393; `Transaction::refuse_opaque` (edit.rs:5196) L5394; `run_of` (edit.rs:8027) L5394; `is_symbol` (edit.rs:7343) L5396
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.is_none`, `.ok_or_else`, `.to_string`, `Err`, `Ok`
- **Called by:** `Transaction::resolve_footnote` (edit/notes.rs:24); `text_component` (edit/v2_apply.rs:2269); `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::check_insert_either` (edit.rs:5364); `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (edit.rs:9129) (?)

#### `Transaction::check_conflicts`  (L5402-5404)
`fn check_conflicts(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::check_conflicts_with_comments` (edit.rs:5410) L5403
- **Called by:** `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051) (?); `Transaction::resolve` (edit.rs:3267); `deeper_boundary_tests::conflicts` (edit.rs:9184) (?)

#### `Transaction::check_conflicts_v2`  (L5406-5408)
`fn check_conflicts_v2(&self) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::check_conflicts_with_comments` (edit.rs:5410) L5407
- **Called by:** `resolve` (edit/v2_apply.rs:1656); `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051) (?)

#### `Transaction::check_conflicts_with_comments`  (L5410-5593)
`fn check_conflicts_with_comments(&self, v2: bool) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::conflict` (edit.rs:5870) L5412,5423,5436,5458,5465,5471...; `Transaction::check_deletions_leave_valid_containers` (edit.rs:5678) L5428; `Transaction::check_paragraph_ops` (edit.rs:5597) L5429; `Transaction::check_control_conflicts` (edit/controls.rs:425) L5430; `Transaction::check_comment_ids_fit` (edit.rs:5748) L5431; `Transaction::check_thread_ops` (edit.rs:5715) L5432; `format_overlaps_edit` (edit/runs.rs:39) L5554,5585; `targets_overlap` (edit/sections.rs:525) L5571
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.entry`, `.enumerate`, `.filter_map`, `.flatten`, `.get`, `.into_iter`, `.into`, `.iter`, `.or_default`, `.push`, `.sort_by_key`, `.values_mut`, `BTreeMap::new`, `Err`, `Ok`, `Some`, `Vec::new`, `comment_conflict`
- **Called by:** `Transaction::check_conflicts` (edit.rs:5402); `Transaction::check_conflicts_v2` (edit.rs:5406)

#### `Transaction::check_paragraph_ops`  (L5597-5674)
`fn check_paragraph_ops(&self, deleted: &[usize]) -> Result<(), EditError>`

- **Does:** Formatting and merging, checked against each other, against deletions and against paragraph insertions.
- **Calls (crate):** `Transaction::conflict` (edit.rs:5870) L5605,5609,5616,5620,5633,5641...
- **Calls (std/external/unresolved):** `.contains`, `.push`, `Err`, `Ok`, `Vec::new`
- **Called by:** `Transaction::check_conflicts_with_comments` (edit.rs:5410)

#### `Transaction::check_deletions_leave_valid_containers`  (L5678-5711)
`fn check_deletions_leave_valid_containers(&self, deleted: &[usize]) -> Result<(), EditError>`

- **Does:** Deletions, taken together, must leave every table cell they touch ending in a paragraph, and the body with a paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L5686; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L5687,5698; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L5690; `Transaction::conflict` (edit.rs:5870) L5691,5703; `Transaction::story_container` (edit.rs:5112) L5697
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.copied`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `.iter`, `.last`, `.map`, `.rev`, `Err`, `Ok`, `Some`, `W::p`, `W::tc`
- **Macros:** `format!`
- **Called by:** `Transaction::check_conflicts_with_comments` (edit.rs:5410)

#### `Transaction::check_thread_ops`  (L5715-5744)
`fn check_thread_ops(&self) -> Result<(), EditError>`

- **Does:** Thread operations against each other: nothing may act on a comment another operation deletes, and a comment is edited once.
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:597) (by name) L5724; `Transaction::conflict` (edit.rs:5870) L5727,5740; `.target` (ambiguous: `EditOptions::target` (cli.rs:358); `Operation::target` (edit/v2.rs:585); `ThreadOp::target` (edit.rs:2567)) L5738
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.contains`, `.expect`, `.iter`, `.push`, `Err`, `Ok`, `Vec::new`
- **Called by:** `Transaction::check_conflicts_with_comments` (edit.rs:5410)

#### `Transaction::check_comment_ids_fit`  (L5748-5758)
`fn check_comment_ids_fit(&self) -> Result<(), EditError>`

- **Does:** New comments take ids after the source's highest; refuse a plan whose comments would not fit in `w:id`'s 32 bits.
- **Calls (crate):** `Transaction::comments_planned` (edit.rs:5763) L5749; `err` (edit.rs:1770) L5751
- **Calls (std/external/unresolved):** `Err`, `Ok`, `u64::from`
- **Called by:** `Transaction::check_conflicts_with_comments` (edit.rs:5410)

#### `Transaction::comments_planned`  (L5763-5787)
`fn comments_planned(&self) -> usize`

- **Does:** How many comments the resolved operations write: a comment on a text edit, an inserted or deleted paragraph, a comment operation and a reply each write one.
- **Calls (crate):** `Transaction::deletion_comment` (edit.rs:6465) L5773
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_some`, `.iter`
- **Macros:** `matches!`
- **Called by:** `Transaction::check_comment_ids_fit` (edit.rs:5748)

#### `Transaction::resolve_settings`  (L5791-5841)
`fn resolve_settings( &self, id: &str, request: &crate::settings::SettingsRequest, ) -> Result<EditOutcome, Box<(EditError, EditOutcome)>>`

- **Does:** Check a `settings` operation: something to write, no password, and one per plan.
- **Calls (crate):** `err` (edit.rs:1770) L5811; `SettingsRequest::describe` (settings.rs:216) L5839
- **Calls (std/external/unresolved):** `.as_ref`, `.is_some_and`, `.is_some`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `crate::settings::SettingsRequest::default`, `fail`
- **Called by:** `Transaction::resolve` (edit.rs:3267)

#### `Transaction::check_redactions`  (L5846-5868)
`fn check_redactions(&self, outputs: &[&[u8]]) -> Result<(), EditError>`

- **Does:** Refuse the plan when the text of a redaction still occurs in one of `outputs` (a comment on it, another paragraph, a header, the document properties).
- **Calls (crate):** `leaks` (scrub.rs:404) L5853; `Transaction::conflict` (edit.rs:5870) L5862
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.flat_map`, `.into_iter`, `.into`, `.is_empty`, `.iter`, `.join`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `produce` (edit/prepared.rs:928)

#### `Transaction::conflict`  (L5870-5879)
`fn conflict(&self, op_index: usize, message: &str) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1770) L5872
- **Calls (std/external/unresolved):** `.clone`, `.into`, `.to_string`, `Some`
- **Called by:** `Transaction::check_control_conflicts` (edit/controls.rs:425); `produce` (edit/prepared.rs:928); `check` (edit/tracked.rs:107); `comment_error` (edit/v2_apply.rs:1898); `Transaction::check_conflicts_with_comments` (edit.rs:5410); `Transaction::check_paragraph_ops` (edit.rs:5597); `Transaction::check_deletions_leave_valid_containers` (edit.rs:5678); `Transaction::check_thread_ops` (edit.rs:5715); `Transaction::check_redactions` (edit.rs:5846)

#### `Transaction::apply_comments`  (L5882-5938)
`fn apply_comments(&mut self) -> BTreeMap<usize, u32>`

- **Does:** Mutate the copy.
- **Calls (crate):** `Transaction::deletion_comment` (edit.rs:6465) L5899,5917; `Transaction::reply_in_deleted` (edit.rs:6477) L5917; `Transaction::reserve_comment_id` (edit.rs:6452) L5919; `Transaction::new_comment` (edit.rs:6446) L5923
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.insert`, `.is_some`, `.iter`, `.map`, `.push`, `BTreeMap::new`, `Some`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_text_edits`  (L5940-6170)
`fn apply_text_edits(&mut self, ids: &BTreeMap<usize, u32>) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::style_defined` (edit/notes.rs:83) L5947,5962; `Transaction::footnotes_story` (edit/notes.rs:51) L5959; `next_footnote_id` (edit/notes.rs:102) L5963; `append_footnote` (edit/notes.rs:132) L5965; `project_paragraph` (inspect.rs:1265) L6032,6039; `sort_edits` (edit.rs:7741) L6036; `run_of` (edit.rs:8027) L6049; `Dom::name_is` (xmllinq/mod.rs:535) L6050; `mark_run` (edit.rs:7679) L6056; `.place` (ambiguous, 4 candidates) L6057; `write_mark_run_text` (edit.rs:7697) L6058; `apply_text_edit` (edit.rs:7450) L6061; `remove_empty_insertion` (edit/tracked.rs:1501) L6071; `new_position` (edit.rs:7174) L6077,6089,6090,6101,6118,6136...; `format_range` (edit.rs:8035) L6078,6091; `insert_reference` (edit/notes.rs:168) L6102; `whole::mark` (ambiguous: `mark` (edit/whole.rs:53); `tests::mark` (edit/whole.rs:489)) L6122; `anchor_comment` (edit.rs:7828) L6154; `restore_plain` (edit/visible.rs:311) L6164; `comment_around` (edit/visible.rs:299) L6166
- **Calls (std/external/unresolved):** `.as_mut`, `.chain`, `.clone`, `.collect`, `.copied`, `.dedup`, `.entry`, `.filter_map`, `.filter`, `.get`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.len`, `.map`, `.or_default`, `.parent`, `.push`, `.remove`, `.rev`, `.saturating_add`, `.sort_by_key`, `.sum`, `.then_some`, `.then`, `.to_string`, `.unwrap_or_default`, `BTreeMap::new`, `Ok`, `Some`, `Vec::new`, `W::ins`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_comment_spans`  (L6172-6184)
`fn apply_comment_spans(&mut self, ids: &BTreeMap<usize, u32>)`

- **Does:** _no doc comment_
- **Calls (crate):** `anchor_span` (edit.rs:7884) L6176
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_insertions`  (L6186-6280)
`fn apply_insertions(&mut self, ids: &BTreeMap<usize, u32>) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `toc_paragraphs` (edit.rs:8255) L6221; `build_paragraph` (edit.rs:8299) L6222; `build_table` (edit/structural.rs:153) L6240; `Transaction::image_paragraph` (edit/images.rs:143) L6257; `Dom::add_after_self` (xmllinq/mod.rs:1147) L6267; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6270; `project_paragraph` (inspect.rs:1265) L6275; `anchor_comment` (edit.rs:7828) L6276
- **Calls (std/external/unresolved):** `.as_deref`, `.cloned`, `.collect`, `.copied`, `.filter`, `.get`, `.insert`, `.is_some`, `.iter`, `.len`, `.push`, `.saturating_add`, `.unwrap_or`, `BTreeMap::new`, `Vec::new`, `crate::markdown::max_drawing_id`, `std::collections::HashSet::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_paragraph_formatting`  (L6282-6301)
`fn apply_paragraph_formatting(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `format_paragraph` (edit.rs:8083) L6292
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_lists`  (L6303-6345)
`fn apply_lists(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `next_numbering_ids` (edit/structural.rs:350) L6305; `abstract_num` (markdown/xml.rs:2553) L6322; `.format` (ambiguous: `ListKind::format` (edit.rs:822); `RunSpec::format` (edit.rs:1073); `Writer::format` (markdown/write.rs:3254)) L6324; `num` (markdown/xml.rs:2598) L6326; `number_paragraph` (edit/structural.rs:435) L6333
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.push`, `.to_string`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_merges`  (L6347-6370)
`fn apply_merges(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `merge_into` (edit.rs:8203) L6363
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.iter`, `.sort_by_key`, `Some`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_paragraph_deletions`  (L6372-6379)
`fn apply_paragraph_deletions(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::remove` (xmllinq/mod.rs:1118) L6376
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_reply_markers`  (L6381-6404)
`fn apply_reply_markers(&mut self, ids: &BTreeMap<usize, u32>)`

- **Does:** _no doc comment_
- **Calls (crate):** `place_reply_markers` (edit.rs:7913) L6390; `CommentFamily::with_replies` (comments.rs:597) (by name) L6394; `remove_comment_markers` (edit.rs:7951) L6399
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.expect`, `.iter`, `.map`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_table_separations`  (L6406-6412)
`fn apply_table_separations(&mut self, tables: Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `separate` (edit/structural.rs:242) L6410
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply_page_setups`  (L6414-6427)
`fn apply_page_setups(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::apply_page_setup` (edit/sections.rs:394) L6425
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.iter`, `Some`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `Transaction::apply`  (L6430-6445)
`fn apply(&mut self) -> Result<(), EditError>`

- **Does:** Mutate the copy.
- **Calls (crate):** `Transaction::apply_comments` (edit.rs:5882) L6431; `Transaction::apply_text_edits` (edit.rs:5940) L6432; `Transaction::apply_comment_spans` (edit.rs:6172) L6433; `Transaction::apply_fills` (edit/controls.rs:488) L6435; `Transaction::apply_insertions` (edit.rs:6186) L6436; `Transaction::apply_paragraph_formatting` (edit.rs:6282) L6437; `Transaction::apply_lists` (edit.rs:6303) L6438; `Transaction::apply_merges` (edit.rs:6347) L6439; `Transaction::apply_paragraph_deletions` (edit.rs:6372) L6440; `Transaction::apply_reply_markers` (edit.rs:6381) L6441; `Transaction::apply_table_separations` (edit.rs:6406) L6442; `Transaction::apply_page_setups` (edit.rs:6414) L6443
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `prepare` (edit/prepared.rs:211); `apply_bound` (edit/v2_apply.rs:121); `Transaction::commented_base` (edit.rs:6518)

#### `Transaction::new_comment`  (L6446-6450)
`fn new_comment(&mut self, text: String) -> u32`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::reserve_comment_id` (edit.rs:6452) L6447
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `Transaction::apply_comments` (edit.rs:5882)

#### `Transaction::reserve_comment_id`  (L6452-6461)
`fn reserve_comment_id(&mut self) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.pop`, `.unwrap_or`, `u32::try_from`
- **Called by:** `Transaction::apply_comments` (edit.rs:5882); `Transaction::new_comment` (edit.rs:6446)

#### `Transaction::deletion_comment`  (L6465-6472)
`fn deletion_comment(&self, op: usize) -> Option<&str>`

- **Does:** The comment of operation `op` when it deletes text or a paragraph, or strikes an insertion: on deleted text, so in the redline only.
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `Transaction::comments_planned` (edit.rs:5763); `Transaction::apply_comments` (edit.rs:5882); `Transaction::deleted_text_comment` (edit.rs:6503)

#### `Transaction::reply_in_deleted`  (L6477-6479)
`fn reply_in_deleted(&self, op: usize) -> bool`

- **Does:** Whether operation `op` replies to a comment whose reference sits in a paragraph this plan deletes: the copy loses the reference, so the reply goes where the parent's stays, on the deleted text (#1159).
- **Calls (crate):** `Transaction::reply_on_deleted` (edit.rs:6483) L6478
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Transaction::apply_comments` (edit.rs:5882); `Transaction::deleted_text_comment` (edit.rs:6503)

#### `Transaction::reply_on_deleted`  (L6483-6492)
`fn reply_on_deleted(&self, op: usize) -> Option<(u32, usize)>`

- **Does:** The parent of reply `op` and the operation that deletes the paragraph the parent is on, when the plan deletes it.
- **Calls (crate):** `Transaction::deletes_paragraph` (edit.rs:6495) L6489
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.map`
- **Called by:** `replies_on_deleted_paragraphs` (edit.rs:1588) (?); `Transaction::reply_in_deleted` (edit.rs:6477)

#### `Transaction::deletes_paragraph`  (L6495-6499)
`fn deletes_paragraph(&self, para: usize) -> Option<usize>`

- **Does:** The operation that deletes paragraph `para`, if one does.
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.then_some`
- **Macros:** `matches!`
- **Called by:** `Transaction::reply_on_deleted` (edit.rs:6483)

#### `Transaction::deleted_text_comment`  (L6503-6511)
`fn deleted_text_comment(&self, op: usize) -> Option<&str>`

- **Does:** The text of [`Self::deletion_comment`], or of a reply [`Self::reply_in_deleted`] names: what goes on the deleted text.
- **Calls (crate):** `Transaction::deletion_comment` (edit.rs:6465) L6504; `Transaction::reply_in_deleted` (edit.rs:6477) L6506
- **Calls (std/external/unresolved):** `.as_str`, `.or_else`, `Some`
- **Called by:** `redline` (edit/tracked.rs:283)

#### `Transaction::commented_base`  (L6518-6641)
`fn commented_base(&self) -> Result<std::borrow::Cow<'_, [u8]>, EditError>`

- **Does:** The copy the comparer reads as the original: the base, with each deleted paragraph's comment anchored on its whole text, and each reply to a comment in a deleted paragraph beside its parent.
- **Calls (crate):** `comment_text_of` (edit.rs:7024) L6578; `Transaction::paragraph_id` (edit.rs:5085) L6587; `Transaction::start` (edit.rs:2680) L6630; `Transaction::resolve` (edit.rs:3267) L6637; `Transaction::apply` (edit.rs:6430) L6638; `Transaction::finish` (edit.rs:6645) L6639
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.rev`, `.unwrap_or_else`, `ExistingRevisions::default`, `Ok`, `RemovedAnchorComments::default`, `Selector::Name`, `Some`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Macros:** `matches!`
- **Called by:** `produce` (edit/prepared.rs:928) (?)

#### `Transaction::finish`  (L6645-6721)
`fn finish(&mut self) -> Result<(Vec<u8>, Option<Vec<u8>>), EditError>`

- **Does:** The clean copy, and the copy the comparer reads when `whole` replacements carry helper bookmarks (the clean copy never does).
- **Calls (crate):** `Transaction::write_comments_part` (edit.rs:6761) L6651; `add_styles` (edit/structural.rs:267) L6668; `splice_numbering` (edit/structural.rs:377) L6672; `Transaction::touched_stories` (edit.rs:5057) L6676; `Transaction::apply_watermark` (edit/watermark.rs:255) L6678; `apply_settings` (settings.rs:347) L6681; `err` (edit.rs:1770) L6682,6693,6717; `Transaction::removes_comment_reference` (edit.rs:6726) L6684; `Transaction::write_stories` (edit.rs:6750) L6688,6699; `PartFs::to_zip` (opc/mod.rs:749) (by name) L6689,6713; `strip` (edit/whole.rs:89) L6695; `revision_bearing_parts` (revision_processor.rs:3716) L6706; `tests::serialized_candidate` (edit/prepared.rs:1287) L6719
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cloned`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.to_string`, `Ok`, `Some`, `crate::revision_processor::prune_orphan_comments`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `produce` (edit/prepared.rs:928); `redline` (edit/tracked.rs:283); `apply_bound` (edit/v2_apply.rs:121); `Transaction::commented_base` (edit.rs:6518)

#### `Transaction::removes_comment_reference`  (L6726-6748)
`fn removes_comment_reference(&self, stories: &std::collections::BTreeSet<usize>) -> bool`

- **Does:** Whether the plan took a comment reference out of a story it writes (a deleted paragraph's, or deleted text's), read against the story as the package still holds it.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L6727; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L6732; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L6734; `PartFs::part_string` (opc/mod.rs:636) L6739; `Dom::new` (xmllinq/mod.rs:387) L6742; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6743; `Dom::root` (xmllinq/mod.rs:822) L6744; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2433) (by name) L6745
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter_map`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_subset`, `.iter`, `.map`, `Some`, `W::id`, `W::name`
- **Called by:** `Transaction::finish` (edit.rs:6645)

#### `Transaction::write_stories`  (L6750-6756)
`fn write_stories(&mut self, stories: &std::collections::BTreeSet<usize>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::serialize_document` (xmllinq/mod.rs:1338) L6753; `PartFs::set_part` (opc/mod.rs:661) L6754
- **Calls (std/external/unresolved):** `.into_bytes`
- **Called by:** `Transaction::finish` (edit.rs:6645)

#### `Transaction::write_comments_part`  (L6761-6813)
`fn write_comments_part(&mut self) -> Result<(), EditError>`

- **Does:** Write the comment part family: the plan's new comments and replies, then its edits, resolutions and deletions; every part consistent (see [`crate::comments`]).
- **Calls (crate):** `CommentFamily::load` (comments.rs:457) L6765; `err` (edit.rs:1770) L6766; `.add` (ambiguous, 5 candidates) L6769; `CommentFamily::set_text` (comments.rs:793) (by name) L6788; `CommentFamily::set_done` (comments.rs:830) (by name) L6793; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L6811
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.iter`, `.map_err`, `.map`, `.remove`, `.retain`, `.take`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::finish` (edit.rs:6645)

#### `lift_out_of_deletion`  (L6823-6864)
`fn lift_out_of_deletion(dom: &mut Dom, scope: NodeId, id: u32, next_id: &mut u64)`

- **Does:** Comment `id`'s markers under `scope`, anchored on words a deletion then took: the range start leaves the deletion it opens in, and the range end, with the reference run after it, the one it closes in.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6826; `Dom::attribute` (xmllinq/mod.rs:874) L6828,6846; `marker` (revision_processor/comments.rs:403) L6830,6841; `Dom::parent` (xmllinq/mod.rs:597) L6831,6849; `Dom::name_is` (xmllinq/mod.rs:535) L6831,6849; `Dom::elements` (xmllinq/mod.rs:640) L6832,6850; `split_deletion_before` (edit.rs:6868) L6833,6852; `Dom::remove` (xmllinq/mod.rs:1118) L6836,6856,6859; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6837; `drop_if_empty` (edit.rs:6885) L6838,6862; `Dom::next_element` (xmllinq/mod.rs:855) L6844; `Dom::element` (xmllinq/mod.rs:655) L6845; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L6851; `Dom::add_after_self` (xmllinq/mod.rs:1147) L6857,6860
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `.last`, `.to_string`, `.unwrap_or`, `Some`, `W::del`, `W::id`, `W::name`
- **Called by:** `lift_deletion_comments` (edit.rs:6914)

#### `split_deletion_before`  (L6868-6882)
`fn split_deletion_before(dom: &mut Dom, del: NodeId, at: NodeId, next_id: &mut u64)`

- **Does:** Split deletion `del` before its child `at`: `at` and what follows it move into a copy of `del` right after it, under revision id `next_id`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L6869; `Dom::attributes` (xmllinq/mod.rs:899) L6870; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6871,6873; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L6876; `Dom::remove` (xmllinq/mod.rs:1118) L6878; `Dom::add` (xmllinq/mod.rs:1074) L6879; `Dom::add_after_self` (xmllinq/mod.rs:1147) L6881
- **Calls (std/external/unresolved):** `.extend`, `.to_string`, `Some`, `W::del`, `W::id`
- **Macros:** `vec!`
- **Called by:** `lift_out_of_deletion` (edit.rs:6823)

#### `drop_if_empty`  (L6885-6889)
`fn drop_if_empty(dom: &mut Dom, del: NodeId)`

- **Does:** Remove deletion `del` once a lifted marker leaves it no element.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L6886; `Dom::remove` (xmllinq/mod.rs:1118) L6887
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `lift_out_of_deletion` (edit.rs:6823)

#### `next_annotation_id`  (L6893-6908)
`fn next_annotation_id(opened: &Opened) -> u64`

- **Does:** One more than the largest `w:id` in the stories of `opened`: a revision id no annotation holds.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L6895; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L6897; `Opened::story_parts` (inspect.rs:861) L6902; `parse_part` (inspect.rs:926) L6903
- **Calls (std/external/unresolved):** `.filter_map`, `.into_iter`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `W::id`, `largest`
- **Called by:** `lift_deletion_comments` (edit.rs:6914)

#### `lift_deletion_comments`  (L6914-6929)
`fn lift_deletion_comments(redline: &[u8], ids: &[u32]) -> Result<Vec<u8>, EditError>`

- **Does:** `redline` with the markers of each comment in `ids`, on words the comparer deleted, lifted out of those deletions ([`lift_out_of_deletion`]): the comment wraps the deletion, so accepting it keeps the comment's reference.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L6918; `err` (edit.rs:1770) L6918,6928; `next_annotation_id` (edit.rs:6893) L6919; `lift_out_of_deletion` (edit.rs:6823) L6921; `Dom::serialize_document` (xmllinq/mod.rs:1338) (by name) L6923; `PartFs::set_part` (opc/mod.rs:661) (by name) L6925; `PartFs::to_zip` (opc/mod.rs:749) (by name) L6926
- **Calls (std/external/unresolved):** `.clone`, `.into_bytes`, `.is_empty`, `.map_err`, `.to_string`, `.to_vec`, `Ok`
- **Called by:** `produce` (edit/prepared.rs:928) (?)

#### `comment_holds_kept_text`  (L6932-6958)
`fn comment_holds_kept_text(redline: &[u8], id: u32) -> bool`

- **Does:** Whether comment `id`'s range in `redline` holds text that is not deleted.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L6933; `Opened::story_parts` (inspect.rs:861) (by name) L6938; `parse_part` (inspect.rs:926) L6940; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L6944; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L6945; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L6948; `XName::local_name` (xmllinq/mod.rs:208) (by name) L6949
- **Calls (std/external/unresolved):** `.as_str`, `.chain`, `.clone`, `.filter`, `.into_iter`, `.map`, `.name`, `.to_string`, `Some`, `W::id`, `is_id`, `std::iter::once`
- **Called by:** `produce` (edit/prepared.rs:928) (?); `deeper_boundary_tests::invalid_package_has_no_revisions_or_kept_comment_text` (edit.rs:8998); `deeper_boundary_tests::revision_and_comment_scans_ignore_rootless_and_foreign_story_parts` (edit.rs:10145)

#### `existing_comment_ids`  (L6960-6973)
`fn existing_comment_ids(opened: &Opened) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::related` (inspect.rs:902) L6961; `PartFs::part_string` (opc/mod.rs:636) L6962; `Dom::new` (xmllinq/mod.rs:387) L6963; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6964; `Dom::root` (xmllinq/mod.rs:822) L6965; `Dom::descendants` (xmllinq/mod.rs:668) L6966; `Dom::attribute` (xmllinq/mod.rs:874) L6969
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.into_iter`, `.max`, `.next`, `.ok`, `.parse`, `Some`, `W::id`, `W::name`
- **Called by:** `Transaction::start` (edit.rs:2680); `Transaction::skip_comment_ids_of` (edit.rs:3213)

#### `is_range_markup`  (L6976-6988)
`fn is_range_markup(dom: &Dom, node: NodeId) -> bool`

- **Does:** Markup that may sit between two block paragraphs and also inside one.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L6986; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L6987; `XName::local_name` (xmllinq/mod.rs:208) (by name) L6987
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`
- **Called by:** `separate` (edit/structural.rs:242); `Transaction::merge_partner` (edit.rs:4859); `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (edit.rs:9891)

#### `kind_name`  (L6990-7019, pub(crate))
`pub(crate) fn kind_name(kind: &OperationKind) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::assert_runtime_dispatch` (capabilities/discovery.rs:359); `outcome_of` (edit/skip.rs:224) (?); `failed_plan_json` (edit/skip.rs:279) (?); `keep_notes` (edit/skip.rs:320) (?); `untracked_kind` (edit/tracked.rs:79) (?); `outcome` (edit/v2_apply.rs:1634) (?); `Transaction::resolve` (edit.rs:3267)

#### `comment_text_of`  (L7024-7029)
`fn comment_text_of(selector: &Selector) -> Option<u32>`

- **Does:** The operation edits or comments text it finds in its paragraph, so it may follow a paragraph into the one it merged with.
- **Calls (crate):** `comment_text_id` (ids.rs:301) L7026
- **Called by:** `expected_texts` (edit/prepared.rs:482) (?); `rewrite_text` (edit/prepared.rs:675) (?); `Transaction::resolve` (edit.rs:3267); `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::resolve_rewrite` (edit.rs:4608); `Transaction::select` (edit.rs:4980); `Transaction::commented_base` (edit.rs:6518)

#### `selector_of`  (L7032-7051)
`fn selector_of(kind: &OperationKind) -> Option<&Selector>`

- **Does:** The paragraph an operation names, when it names one.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Transaction::edits_a_comment_range` (edit.rs:2913); `Transaction::resolve` (edit.rs:3267)

#### `selectors_mut`  (L7055-7079)
`fn selectors_mut(kind: &mut OperationKind) -> Vec<&mut Selector>`

- **Does:** Every paragraph an operation names: its paragraph, a comment's `through`, an inserted paragraph's `like`, a list's `paragraphs`.
- **Calls (std/external/unresolved):** `.as_mut`, `.chain`, `.collect`, `.iter_mut`, `Vec::new`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `source_only` (edit/skip.rs:356); `rename_positions` (edit/skip.rs:394)

#### `finds_text`  (L7081-7091)
`fn finds_text(kind: &OperationKind) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545)

#### `check_text`  (L7094-7102)
`fn check_text(text: &str) -> Result<(), String>`

- **Does:** Plain text only: no control characters (tabs and breaks are not run text).
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.into`, `.is_control`, `Err`, `Ok`
- **Macros:** `matches!`
- **Called by:** `Transaction::fill_value` (edit/controls.rs:318); `picture` (edit/images.rs:86); `Transaction::resolve_footnote` (edit/notes.rs:24); `check_rows` (edit/structural.rs:28); `spec` (edit/watermark.rs:43); `Transaction::resolve_one_unreported` (edit.rs:3545); `tests::check_text_rejects_controls` (edit.rs:8722)

#### `check_run_text`  (L7107-7123)
`fn check_run_text(text: &str) -> Result<(), String>`

- **Does:** Run text: plain text in which `\t` writes a tab and `\n` a line break, as the agent view prints them; any other control character, and the view's symbol placeholder U+FFFC, is refused by name.
- **Calls (std/external/unresolved):** `.chars`, `.find`, `.into`, `.is_control`, `Err`, `Ok`, `from`
- **Macros:** `format!`, `matches!`
- **Called by:** `text_component` (edit/v2_apply.rs:2269); `Transaction::resolve_comment_text` (edit.rs:3057); `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::resolve_rewrite` (edit.rs:4608)

#### `blank_is_none`  (L7127-7130)
`fn blank_is_none<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<String>, D::Error>`

- **Does:** An optional comment as a plan gives it: blank text (`""`, spaces, line breaks) is no comment, as an agent with nothing to say writes it.
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.trim`, `Ok`, `serde::Deserialize::deserialize`
- **Called by:** none found in src

#### `check_comment`  (L7134-7139)
`fn check_comment(text: &str) -> Result<(), String>`

- **Does:** Comment text: nonempty, and each line (`\n` starts a new comment paragraph) plain text that `comments.xml` can carry.
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.split`, `.trim`, `.try_for_each`, `Err`
- **Called by:** `comment_component` (edit/v2_apply.rs:2244); `text_component` (edit/v2_apply.rs:2269); `Transaction::resolve_visible` (edit/visible.rs:348) (?); `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::resolve_thread` (edit.rs:4379); `Transaction::resolve_rewrite` (edit.rs:4608)

#### `redaction`  (L7143-7145)
`fn redaction(find: &str) -> String`

- **Does:** `before {mark} after` with up to 20 chars of context on either side.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.repeat`
- **Called by:** `Transaction::resolve_one` (edit.rs:3479); `Transaction::resolve_one_unreported` (edit.rs:3545)

#### `context`  (L7147-7159)
`fn context(text: &str, start: usize, end: usize, mark: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.into_iter`, `.rev`, `.take`
- **Macros:** `format!`
- **Called by:** `public_option_source_ownership_tests::assert_source_projection` (comparer/cross_para.rs:2661) (?); `coverage_real_revision_merge_matrix_tests::compact_source_difference` (comparer/finalize.rs:24919) (?); `finalizer_remaining_source_boundary_tests::assert_body` (comparer/finalize.rs:30759) (?); `finalizer_merge_source_guard_tests::assert_body` (comparer/finalize.rs:31500) (?); `finalizer_reached_predicate_boundary_tests::assert_body` (comparer/finalize.rs:31936) (?); `existing_mark_layout_ownership_tests::assert_body` (comparer/finalize.rs:32984) (?); `interleave_terminal_source_ownership_tests::remains_source_owned` (comparer/finalize.rs:33328) (?); `whitespace_source_mark_carrier_tests::check_complete` (comparer/finalize.rs:33535) (?); `fragmented_source_label_boundary_tests::assert_complete` (comparer/finalize.rs:33704) (?); `row_pair_uses_word_table_mesh_context` (comparer/lcs_table.rs:530) (?); `Transaction::resolve_comment_text` (edit.rs:3057); `Transaction::resolve_one_unreported` (edit.rs:3545); +1 more

#### `excerpt`  (L7161-7167)
`fn excerpt(text: &str, max: usize) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.push`, `.take`
- **Called by:** `Transaction::resolve_fill_control` (edit/controls.rs:73); `Transaction::resolve_visible` (edit/visible.rs:348) (?); `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::resolve_rewrite` (edit.rs:4608); `Transaction::resolve_list` (edit.rs:4730); `tests::context_windows_and_excerpts` (edit.rs:8676); `describe` (markdown/patch.rs:290)

#### `new_position`  (L7174-7198)
`fn new_position(edits: &[ScheduledEdit], pos: usize, inclusive: bool, own: Option<usize>) -> usize`

- **Does:** Position `pos` of the source projection after `edits` were applied.
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.is_none_or`, `.iter`, `.len`, `.max`, `.position`, `Some`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_text_edits` (edit.rs:5940); `tests::new_position_accounts_for_earlier_edits_and_insertions` (edit.rs:8686)

#### `locate`  (L7203-7311)
`fn locate( projection: &Projection, find: &str, occurrence: Option<usize>, outcome: &mut EditOutcome, window: Option<(u32, usize, usize)>, ) -> Result<(usize, usize), (String, String)>`

- **Does:** The unique occurrence of `find` in the paragraph, or its `occurrence`-th hit (1-based) when given (overlapping occurrences count), whatever structure it crosses.
- **Calls (crate):** `inside` (ambiguous: `inside` (comparer/finalize.rs:14956); `inside` (markdown/critic.rs:445)) L7237,7257,7259; `reading_hits` (edit/anchor.rs:79) L7257,7259; `shown` (edit/anchor.rs:95) L7271; `near_matches` (edit/anchor.rs:187) L7275
- **Calls (std/external/unresolved):** `.as_slice`, `.char_indices`, `.collect`, `.copied`, `.filter`, `.find`, `.get`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.push`, `.set`, `.starts_with`, `.then_some`, `.to_string`, `Err`, `Ok`, `Some`, `Vec::new`, `crate::markdown::plain_anchor`, `exact`, `hits_of`, `std::cell::Cell::new`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545); `Transaction::find_range` (edit.rs:5130)

#### `editable_insertion`  (L7316-7340)
`fn editable_insertion(dom: &Dom, piece: &Piece) -> Option<NodeId>`

- **Does:** A run directly inside a paragraph's pending insertion is editable.
- **Calls (crate):** `run_of` (edit.rs:8027) L7317; `Dom::parent` (xmllinq/mod.rs:597) L7318,7319; `Dom::name_is` (xmllinq/mod.rs:535) L7318,7319,7331; `Dom::descendants` (xmllinq/mod.rs:668) L7325; `Dom::attribute` (xmllinq/mod.rs:874) L7332
- **Calls (std/external/unresolved):** `.filter`, `.saturating_sub`, `.then_some`, `W::ins`, `W::name`, `W::p`
- **Called by:** `insertion_replacement` (edit/tracked.rs:987); `emit_text` (edit/tracked.rs:1035); `edited_revisions` (edit/v2_apply.rs:2344); `Transaction::resolve_visible` (edit/visible.rs:348) (?); `Transaction::check_range` (edit.rs:5160); `Transaction::check_insert_position` (edit.rs:5381)

#### `is_symbol`  (L7343-7345)
`fn is_symbol(projection: &Projection, seg: &crate::inspect::Segment) -> bool`

- **Does:** `seg` is a `w:sym`, projected as U+FFFC.
- **Macros:** `matches!`
- **Called by:** `Transaction::check_range` (edit.rs:5160); `Transaction::check_insert_position` (edit.rs:5381)

#### `attach_segment`  (L7348-7365)
`fn attach_segment( projection: &Projection, pos: usize, attach_before: bool, ) -> Option<&crate::inspect::Segment>`

- **Does:** The segment an insertion at `pos` attaches to.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.or_else`, `.rev`
- **Called by:** `insert_reference` (edit/notes.rs:168); `emit_text` (edit/tracked.rs:1035); `edited_revisions` (edit/v2_apply.rs:2344); `Transaction::check_insert_position` (edit.rs:5381); `apply_text_edit` (edit.rs:7450); `deeper_boundary_tests::text_edit_glyphs_and_segment_attachment_are_exact` (edit.rs:9679)

#### `word_window`  (L7375-7388)
`fn word_window(text: &str, start: usize, end: usize, separators: &[char]) -> (usize, usize)`

- **Does:** The byte span of the words of `text` that `[start, end)` touches: out to the nearest whitespace or comparer word separator on each side, so at least the words the comparer would rewrite.
- **Calls (std/external/unresolved):** `.char_indices`, `.contains`, `.find`, `.is_whitespace`, `.last`, `.len`, `.map_or`, `.rev`, `.take_while`, `wordy`
- **Called by:** `Transaction::edits_beside_a_range_marker` (edit.rs:2872)

#### `range_marker`  (L7393-7400)
`fn range_marker(dom: &Dom, node: NodeId) -> Option<&'static str>`

- **Does:** `node`'s local name when it is a range marker: an opener of [`tracked::RANGE_STARTS`], a closer of [`tracked::RANGE_ENDS`], or a comment range's end.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L7394; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L7394; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7399
- **Calls (std/external/unresolved):** `.chain`, `.filter`, `.find`, `.into_iter`
- **Called by:** `Transaction::body_level_marker_beside` (edit.rs:2897); `Transaction::range_marks` (edit.rs:2970)

#### `insert_outside_ranges`  (L7406-7444)
`fn insert_outside_ranges( dom: &mut Dom, run: NodeId, t: NodeId, at: usize, len: usize, replacement: &str, ) -> bool`

- **Does:** Insert `replacement` at `at` of `t` (of `len` bytes: a `w:t`, or a tab or break glyph) in its own run, formatted as `run`, when `at` is an edge of `run` with a range marker beside it: the new words then join no range they meet, as the redline writes them (#1156).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L7414; `Dom::name_is` (xmllinq/mod.rs:535) L7417; `outside_ranges` (edit/tracked.rs:1348) L7426; `Dom::new_element` (xmllinq/mod.rs:478) L7430,7435; `Dom::element` (xmllinq/mod.rs:655) L7431; `clean_rpr` (edit/tracked.rs:1789) L7432; `Dom::add` (xmllinq/mod.rs:1074) L7433,7436; `write_text` (edit.rs:7748) L7437; `Dom::add_before_self` (xmllinq/mod.rs:1130) L7439; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7441
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.first`, `.into_iter`, `.last`, `Some`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `apply_text_edit` (edit.rs:7450)

#### `apply_text_edit`  (L7450-7551)
`fn apply_text_edit( dom: &mut Dom, projection: &Projection, start: usize, end: usize, replacement: &str, attach_before: bool, )`

- **Does:** Replace `[start, end)` of the paragraph's projection with `replacement`.
- **Calls (crate):** `attach_segment` (edit.rs:7348) L7459; `Dom::elements` (xmllinq/mod.rs:640) L7468; `Dom::name_is` (xmllinq/mod.rs:535) L7471; `insertion_boundary_anchor` (edit/tracked.rs:1303) L7479; `Dom::new_element` (xmllinq/mod.rs:478) L7481,7486,7511,7542; `Dom::element` (xmllinq/mod.rs:655) L7482; `clean_rpr` (edit/tracked.rs:1789) L7483; `Dom::add` (xmllinq/mod.rs:1074) L7484,7487; `write_text` (edit.rs:7748) L7488,7517,7544; `Dom::add_before_self` (xmllinq/mod.rs:1130) L7490,7515,7543; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7492,7513; `Dom::value` (xmllinq/mod.rs:1185) L7499,7532; `insert_outside_ranges` (edit.rs:7406) L7501,7510; `write_text_new_in` (edit.rs:7755) L7503,7537; `inner_comment_references` (edit.rs:7556) L7523; `closing_comment_references` (edit.rs:7595) L7524; `Dom::remove` (xmllinq/mod.rs:1118) L7546; `remove_comment_references` (edit.rs:7657) L7550
- **Calls (std/external/unresolved):** `.collect`, `.expect`, `.extend`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.last`, `.len`, `.min`, `.saturating_sub`, `.take`, `.unwrap_or_default`, `Some`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `emit` (edit/tracked.rs:452); `emit_text` (edit/tracked.rs:1035); `Transaction::apply_text_edits` (edit.rs:5940); `deeper_boundary_tests::text_edit_glyphs_and_segment_attachment_are_exact` (edit.rs:9679)

#### `inner_comment_references`  (L7556-7587)
`fn inner_comment_references( dom: &Dom, projection: &Projection, start: usize, end: usize, ) -> Vec<NodeId>`

- **Does:** The comment references between the first and the last character of `[start, end)`: a selection over them takes their mark along, and Word then drops the comment (rule R1).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L7575; `Dom::descendants` (xmllinq/mod.rs:668) L7578,7583; `at` (markdown/from_docx/drawing.rs:1360) (by name) L7580,7585
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.first`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.next_back`, `.next`, `.position`, `Some`, `Vec::new`, `W::name`, `W::p`
- **Called by:** `apply_text_edit` (edit.rs:7450)

#### `closing_comment_references`  (L7595-7654)
`fn closing_comment_references( dom: &Dom, projection: &Projection, start: usize, end: usize, ) -> Vec<NodeId>`

- **Does:** The references of the comments wholly inside `[start, end)` that close where it ends: the edit deletes all their text, as it does a comment whose reference stands between its first and last character, but the reference follows the last one, where [`inner_comment_references`] does not look.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L7615; `Dom::descendants` (xmllinq/mod.rs:668) L7618; `at` (markdown/from_docx/drawing.rs:1360) (by name) L7620,7630; `Dom::name_is` (xmllinq/mod.rs:535) L7636,7640,7648; `Dom::attribute` (xmllinq/mod.rs:874) L7636,7641,7649; `marker` (revision_processor/comments.rs:403) L7642
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.contains`, `.copied`, `.filter_map`, `.filter`, `.find`, `.first`, `.is_some_and`, `.iter`, `.len`, `.max`, `.next_back`, `.next`, `.position`, `.unwrap_or`, `Some`, `Vec::new`, `W::id`, `W::name`, `W::p`, `node`
- **Called by:** `apply_text_edit` (edit.rs:7450)

#### `remove_comment_references`  (L7657-7668)
`fn remove_comment_references(dom: &mut Dom, references: &[NodeId])`

- **Does:** Remove comment references, with the run when it holds nothing else.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L7659; `Dom::name_is` (xmllinq/mod.rs:535) L7660,7664; `Dom::elements` (xmllinq/mod.rs:640) L7661; `Dom::remove` (xmllinq/mod.rs:1118) L7666
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.into_iter`, `.unwrap_or`, `W::r_pr`, `W::r`
- **Called by:** `apply_text_edit` (edit.rs:7450)

#### `set_text`  (L7670-7673)
`fn set_text(dom: &mut Dom, t: NodeId, value: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::set_value` (xmllinq/mod.rs:1219) L7671; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L7672; `XNamespace::xml` (xmllinq/mod.rs:128) L7672
- **Calls (std/external/unresolved):** `.name`, `Some`
- **Called by:** `emit` (edit/tracked.rs:452); `emit_text` (edit/tracked.rs:1035); `write_text_new_in` (edit.rs:7755); `split_run_at` (edit.rs:7790); `merge_into` (edit.rs:8203)

#### `mark_run`  (L7679-7694, pub(super))
`pub(super) fn mark_run(dom: &mut Dom, paragraph: NodeId) -> NodeId`

- **Does:** A new run for text typed into the empty `paragraph`, as Word types it: formatted as the paragraph mark (its `w:pPr/w:rPr`, without the mark's own revisions or `w:oMath`, which only a mark takes).
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L7680; `Dom::element` (xmllinq/mod.rs:655) L7681,7683,7686; `clean_rpr` (edit/tracked.rs:1789) L7685; `Dom::remove` (xmllinq/mod.rs:1118) L7687; `Dom::has_elements` (xmllinq/mod.rs:862) L7689; `Dom::add` (xmllinq/mod.rs:1074) L7690
- **Calls (std/external/unresolved):** `.and_then`, `W::name`, `W::p_pr`, `W::r_pr`, `W::r`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `Transaction::apply_text_edits` (edit.rs:5940)

#### `write_mark_run_text`  (L7697-7701, pub(super))
`pub(super) fn write_mark_run_text(dom: &mut Dom, run: NodeId, text: &str)`

- **Does:** `text` written into `run`, a [`mark_run`] already in the tree.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L7698; `Dom::add` (xmllinq/mod.rs:1074) L7699; `write_text` (edit.rs:7748) L7700
- **Calls (std/external/unresolved):** `W::t`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `Transaction::apply_text_edits` (edit.rs:5940)

#### `EmptyPlacement::place`  (L7717-7735, pub(super))
`pub(super) fn place( &mut self, dom: &mut Dom, paragraph: NodeId, node: NodeId, attach_before: bool, )`

- **Does:** Put `node` where its text was typed in `paragraph`: last for an insert at the end (before the end text already placed), else first, after the paragraph's properties (an insert at the start, a rewrite).
- **Calls (crate):** `Dom::add_before_self` (xmllinq/mod.rs:1130) L7726; `Dom::add` (xmllinq/mod.rs:1074) L7727; `Dom::element` (xmllinq/mod.rs:655) L7730; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7731; `Dom::add_first` (xmllinq/mod.rs:1101) L7733
- **Calls (std/external/unresolved):** `Some`, `W::p_pr`
- **Called by:** none found in src

#### `sort_edits`  (L7741-7743)
`fn sort_edits(edits: &mut [ScheduledEdit], empty: bool)`

- **Does:** Sort a paragraph's edits for applying: by source range, then plan order.
- **Calls (std/external/unresolved):** `.sort_by_key`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_text_edits` (edit.rs:5940)

#### `write_text`  (L7748-7750)
`fn write_text(dom: &mut Dom, t: NodeId, value: &str)`

- **Does:** Set the text of `t`, a `w:t` in a run, writing each `\t` as a `w:tab` and each `\n` as a `w:br` of that run, with the text between them in `w:t`s of its own; `t` itself goes when no text is left for it.
- **Calls (crate):** `write_text_new_in` (edit.rs:7755) L7749
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `insert_outside_ranges` (edit.rs:7406); `apply_text_edit` (edit.rs:7450); `write_mark_run_text` (edit.rs:7697); `build_paragraph` (edit.rs:8299)

#### `write_text_new_in`  (L7755-7786)
`fn write_text_new_in(dom: &mut Dom, t: NodeId, value: &str, new: std::ops::Range<usize>)`

- **Does:** [`write_text`] for `value` whose new text is `new` alone: the text kept around it stays as the source wrote it, a tab typed in `w:t` included, so an edit never restates what it did not touch.
- **Calls (crate):** `set_text` (edit.rs:7670) L7761,7764,7779; `Dom::new_element` (xmllinq/mod.rs:478) L7772,7777; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7773,7778; `Dom::remove` (xmllinq/mod.rs:1118) L7784
- **Calls (std/external/unresolved):** `.as_bytes`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.first`, `.get`, `.is_empty`, `.iter`, `.len`, `.map`, `.match_indices`, `.unwrap_or`, `W::name`, `W::t`
- **Called by:** `apply_text_edit` (edit.rs:7450); `write_text` (edit.rs:7748)

#### `split_run_at`  (L7790-7824)
`fn split_run_at(dom: &mut Dom, seg: &crate::inspect::Segment, at: usize)`

- **Does:** Split the run owning `seg` so that a run boundary falls at projection byte `at` (inside `seg`).
- **Calls (crate):** `Dom::value` (xmllinq/mod.rs:1185) L7797; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L7802; `Dom::elements` (xmllinq/mod.rs:640) L7803,7804; `Dom::name_is` (xmllinq/mod.rs:535) L7810; `Dom::remove` (xmllinq/mod.rs:1118) L7812,7817; `set_text` (edit.rs:7670) L7820,7822; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7823
- **Calls (std/external/unresolved):** `.enumerate`, `.expect`, `.iter`, `.position`, `.to_string`, `W::r_pr`
- **Called by:** `insert_reference` (edit/notes.rs:168); `emit_text` (edit/tracked.rs:1035); `isolate_range` (edit/tracked.rs:1516); `wrap_range` (edit.rs:7979); `runs_in_range` (edit.rs:8054); `deeper_boundary_tests::splitting_multichild_run_preserves_order_and_properties` (edit.rs:9726)

#### `anchor_comment`  (L7828-7845)
`fn anchor_comment(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, id: u32)`

- **Does:** Wrap the projection range `[start, end)` of `paragraph` in comment markers for comment `id`, splitting runs at the boundaries as needed.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L7830,7832,7834,7835; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L7831,7833,7836; `Dom::add` (xmllinq/mod.rs:1074) L7837,7841,7842,7843; `wrap_range` (edit.rs:7979) L7838; `place_reference` (edit.rs:7853) L7839
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::id`, `W::name`, `W::r`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_text_edits` (edit.rs:5940); `Transaction::apply_insertions` (edit.rs:6186); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:9760)

#### `place_reference`  (L7853-7866)
`fn place_reference(dom: &mut Dom, range_start: NodeId, range_end: NodeId, reference_run: NodeId)`

- **Does:** Put a comment's reference run right after its end marker, or, when the range starts outside the tracked insertion holding that marker, after the insertion: rejecting it would take a reference inside it, and the comment with it, though only part of the commented text was inserted.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L7855,7860; `Dom::name_is` (xmllinq/mod.rs:535) L7856,7859; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7865
- **Calls (std/external/unresolved):** `.contains`, `W::ins`, `W::name`, `W::p`
- **Called by:** `anchor_comment` (edit.rs:7828); `anchor_span` (edit.rs:7884)

#### `comment_marker`  (L7868-7872)
`fn comment_marker(dom: &mut Dom, local: &str, id: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L7869; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L7870
- **Calls (std/external/unresolved):** `Some`, `W::id`, `W::name`
- **Called by:** `comment_around` (edit/visible.rs:299) (?); `comment_reference_run` (edit.rs:7874); `anchor_span` (edit.rs:7884); `place_reply_markers` (edit.rs:7913); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:9760)

#### `comment_reference_run`  (L7874-7879)
`fn comment_reference_run(dom: &mut Dom, id: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L7875; `comment_marker` (edit.rs:7868) L7876; `Dom::add` (xmllinq/mod.rs:1074) L7877
- **Calls (std/external/unresolved):** `W::r`
- **Called by:** `comment_around` (edit/visible.rs:299) (?); `anchor_span` (edit.rs:7884); `place_reply_markers` (edit.rs:7913)

#### `anchor_span`  (L7884-7908)
`fn anchor_span(dom: &mut Dom, first: NodeId, last: NodeId, id: u32)`

- **Does:** Comment `id` from the start of paragraph `first` to the end of `last`: the start marker before `first`'s first run, the end marker and the reference run after `last`'s last run (appended to an empty paragraph).
- **Calls (crate):** `comment_marker` (edit.rs:7868) L7886,7887; `comment_reference_run` (edit.rs:7874) L7888; `project_paragraph` (inspect.rs:1265) L7889,7899; `Dom::new_element` (xmllinq/mod.rs:478) L7890,7900; `wrap_range` (edit.rs:7979) L7891,7901; `Dom::remove` (xmllinq/mod.rs:1118) L7892,7902; `Dom::element` (xmllinq/mod.rs:655) L7894; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7895; `Dom::add_first` (xmllinq/mod.rs:1101) L7896; `place_reference` (edit.rs:7853) L7903; `Dom::add` (xmllinq/mod.rs:1074) L7905,7906
- **Calls (std/external/unresolved):** `.len`, `.to_string`, `W::name`, `W::p_pr`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_comment_spans` (edit.rs:6172); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:9760)

#### `place_reply_markers`  (L7913-7947)
`fn place_reply_markers(dom: &mut Dom, roots: &[NodeId], parent: u32, id: u32)`

- **Does:** Markers of reply `id` beside those of comment `parent`, as Word writes them: the start after the parent's start, the end after the parent's end, the reference run after the parent's reference run.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7918; `Dom::attribute` (xmllinq/mod.rs:874) L7920; `find` (ambiguous: `find` (markdown/pages.rs:452); `find` (markdown/unified.rs:474)) L7924,7925,7926; `Dom::parent` (xmllinq/mod.rs:597) L7931; `Dom::name_is` (xmllinq/mod.rs:535) L7933; `comment_reference_run` (edit.rs:7874) L7935; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7936,7939,7944; `comment_marker` (edit.rs:7868) L7937,7942; `Dom::add_before_self` (xmllinq/mod.rs:1130) L7940,7945
- **Calls (std/external/unresolved):** `.as_str`, `.filter`, `.find_map`, `.find`, `.into_iter`, `.iter`, `.to_string`, `.unwrap_or`, `Some`, `W::id`, `W::name`, `W::r`
- **Called by:** `thread_markers` (edit/tracked.rs:956); `Transaction::apply_reply_markers` (edit.rs:6381); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:9760)

#### `remove_comment_markers`  (L7951-7974)
`fn remove_comment_markers(dom: &mut Dom, roots: &[NodeId], ids: &[String])`

- **Does:** Remove the range markers and references of the comments `ids`; a run left holding nothing but its properties goes too.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7954; `Dom::attribute` (xmllinq/mod.rs:874) L7955; `Dom::parent` (xmllinq/mod.rs:597) L7961; `Dom::name_is` (xmllinq/mod.rs:535) L7961,7967; `Dom::remove` (xmllinq/mod.rs:1118) L7962,7969; `Dom::elements` (xmllinq/mod.rs:640) L7964
- **Calls (std/external/unresolved):** `.all`, `.any`, `.filter`, `.is_some_and`, `.iter`, `Some`, `W::id`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `thread_markers` (edit/tracked.rs:956); `Transaction::apply_reply_markers` (edit.rs:6381); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:9760)

#### `wrap_range`  (L7979-8025)
`fn wrap_range( dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, open: NodeId, close: NodeId, ) -> bool`

- **Does:** Put `open` right before the run holding projection offset `start` and `close` right after the run ending at `end`, splitting runs at both offsets first.
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) L7987,7996,8005; `split_run_at` (edit.rs:7790) L7994,8003; `run_of` (edit.rs:8027) L8010,8016; `Dom::add_before_self` (xmllinq/mod.rs:1130) L8019; `Dom::add_after_self` (xmllinq/mod.rs:1147) L8020
- **Calls (std/external/unresolved):** `.cloned`, `.find`, `.iter`, `.map`, `.max`, `.rev`
- **Called by:** `mark` (edit/whole.rs:53); `anchor_comment` (edit.rs:7828); `anchor_span` (edit.rs:7884); `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (edit.rs:9760)

#### `run_of`  (L8027-8031)
`fn run_of(piece: &Piece) -> NodeId`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `insert_reference` (edit/notes.rs:168); `emit_text` (edit/tracked.rs:1035); `offset_before` (edit/visible.rs:328) (?); `Transaction::resolve_visible` (edit/visible.rs:348) (?); `Transaction::check_range` (edit.rs:5160); `Transaction::check_insert_position` (edit.rs:5381); `Transaction::apply_text_edits` (edit.rs:5940); `editable_insertion` (edit.rs:7316); `wrap_range` (edit.rs:7979); `runs_in_range` (edit.rs:8054)

#### `format_range`  (L8035-8050)
`fn format_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, format: &RunFormat)`

- **Does:** Apply `format` to the runs holding projection range `[start, end)` of `paragraph`, splitting runs at the boundaries first.
- **Calls (crate):** `runs_in_range` (edit.rs:8054) L8036; `Dom::element` (xmllinq/mod.rs:655) L8037; `Dom::new_element` (xmllinq/mod.rs:478) L8040; `Dom::add_first` (xmllinq/mod.rs:1101) L8041; `apply_run_format` (edit.rs:8374) L8045; `Dom::elements` (xmllinq/mod.rs:640) L8046; `Dom::remove` (xmllinq/mod.rs:1118) L8047
- **Calls (std/external/unresolved):** `.is_empty`, `W::r_pr`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `Transaction::apply_text_edits` (edit.rs:5940); `deeper_boundary_tests::format_range_deduplicates_runs_and_removes_empty_properties` (edit.rs:9811)

#### `runs_in_range`  (L8054-8080)
`fn runs_in_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize) -> Vec<NodeId>`

- **Does:** The runs holding projection range `[start, end)` of `paragraph`, in order, split at the boundaries first; none for an empty range.
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) L8059,8069; `split_run_at` (edit.rs:7790) L8066; `run_of` (edit.rs:8027) L8073
- **Calls (std/external/unresolved):** `.cloned`, `.contains`, `.find`, `.iter`, `.push`, `Vec::new`
- **Called by:** `track_run_format` (edit/tracked.rs:924); `format_range` (edit.rs:8035)

#### `format_paragraph`  (L8083-8139)
`fn format_paragraph( dom: &mut Dom, paragraph: NodeId, style: Option<&str>, alignment: Option<Alignment>, spacing: Spacing, )`

- **Does:** Set the paragraph style, alignment and spacing in `paragraph`'s `w:pPr`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8090,8099,8108,8117; `Dom::new_element` (xmllinq/mod.rs:478) L8093,8102,8111,8120; `Dom::add_first` (xmllinq/mod.rs:1101) L8094,8104; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8100,8103,8109,8112,8126,8127...; `insert_ppr_child` (edit.rs:8182) L8113,8121
- **Calls (std/external/unresolved):** `.jc`, `.to_string`, `Some`, `Spacing::default`, `W::name`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_paragraph_formatting` (edit.rs:6282); `deeper_boundary_tests::paragraph_format_partial_spacing_preserves_other_axes` (edit.rs:9833)

#### `insert_ppr_child`  (L8182-8198)
`fn insert_ppr_child(dom: &mut Dom, ppr: NodeId, child: NodeId)`

- **Does:** Insert `child` into `ppr` at its schema position.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L8189,8191; `rank` (edit.rs:8536) L8189,8192; `XName::local_name` (xmllinq/mod.rs:208) (by name) L8189,8192; `Dom::elements` (xmllinq/mod.rs:640) L8190; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L8192; `Dom::add_after_self` (xmllinq/mod.rs:1147) L8195; `Dom::add_first` (xmllinq/mod.rs:1101) L8196
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.position`, `.rev`, `.unwrap_or`
- **Called by:** `number_paragraph` (edit/structural.rs:435); `track_properties` (edit/tracked.rs:888); `mark` (edit/tracked.rs:1766); `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (edit/tracked.rs:1995); `format_paragraph` (edit.rs:8083); `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (edit.rs:9891)

#### `merge_into`  (L8203-8251)
`fn merge_into(dom: &mut Dom, head: NodeId, next: NodeId, separator: &str)`

- **Does:** Move `head`'s content, then `separator` as a run, then any range markup between the two, to the start of `next`, and drop `head`.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L8204,8237; `Dom::name_is` (xmllinq/mod.rs:535) L8207,8218,8219,8220,8240; `Dom::new_element` (xmllinq/mod.rs:478) L8210,8227; `Dom::elements` (xmllinq/mod.rs:640) L8211,8217; `Dom::element` (xmllinq/mod.rs:655) L8214; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L8216; `Dom::remove` (xmllinq/mod.rs:1118) L8222,8243,8250; `Dom::add` (xmllinq/mod.rs:1074) L8225,8229,8247; `set_text` (edit.rs:7670) L8228; `Dom::next_element` (xmllinq/mod.rs:855) L8232,8235; `Dom::parent` (xmllinq/mod.rs:597) L8242; `Dom::add_before_self` (xmllinq/mod.rs:1130) L8246
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some`, `.last`, `.push`, `Some`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `Transaction::apply_merges` (edit.rs:6347); `deeper_boundary_tests::merge_separator_copies_properties_without_revision_children` (edit.rs:9935)

#### `toc_paragraphs`  (L8255-8294)
`fn toc_paragraphs(dom: &mut Dom, toc: &TocSpec) -> Vec<NodeId>`

- **Does:** `insert_toc`'s paragraphs: the optional `TOCHeading` title, then a paragraph holding an empty `TOC` field.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L8258,8259,8260,8264,8265,8272...; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8261,8266,8276,8283; `Dom::add` (xmllinq/mod.rs:1074) L8262,8263,8268,8269,8277,8278...; `XNamespace::xml` (xmllinq/mod.rs:128) L8266,8283; `Dom::add_text` (xmllinq/mod.rs:1087) L8267,8284; `mark` (ambiguous, 5 candidates) L8280,8290,8291
- **Calls (std/external/unresolved):** `.name`, `.push`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p_style`, `W::p`, `W::r`, `W::t`, `W::val`
- **Macros:** `format!`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_insertions` (edit.rs:6186)

#### `build_paragraph`  (L8299-8371)
`fn build_paragraph(dom: &mut Dom, anchor: NodeId, runs: &[RunSpec], style: Option<&str>) -> NodeId`

- **Does:** A new paragraph modeled on `anchor`: its `pPr` minus section break and revision marks, runs formatted like the anchor's first run plus the requested toggles.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L8300,8315,8322,8323,8346,8349...; `Dom::element` (xmllinq/mod.rs:655) L8301,8312,8341; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L8302,8348; `Dom::elements` (xmllinq/mod.rs:640) L8303,8331,8336,8351,8360; `Dom::name_is` (xmllinq/mod.rs:535) L8304,8305,8306,8352,8353,8354; `Dom::remove` (xmllinq/mod.rs:1118) L8308,8356,8361; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8313,8316,8324; `Dom::add_first` (xmllinq/mod.rs:1101) L8317; `Dom::add` (xmllinq/mod.rs:1074) L8320,8325,8326,8363,8366,8368; `Dom::value` (xmllinq/mod.rs:1185) L8338; `apply_run_format` (edit.rs:8374) L8359; `.format` (ambiguous: `ListKind::format` (edit.rs:822); `RunSpec::format` (edit.rs:1073); `Writer::format` (markdown/write.rs:3254)) L8359; `write_text` (edit.rs:7748) L8367
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.count`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.max_by_key`, `.rev`, `.sum`, `Some`, `W::del`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::r_pr_change`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `W::val`
- **Called by:** `cell_paragraph` (edit/structural.rs:220); `emit` (edit/tracked.rs:452); `Transaction::apply_insertions` (edit.rs:6186); `deeper_boundary_tests::new_paragraph_style_and_revision_cleanup_truth_table` (edit.rs:9968)

#### `apply_run_format`  (L8374-8429)
`fn apply_run_format(dom: &mut Dom, rpr: NodeId, format: &RunFormat)`

- **Does:** Set or clear the requested properties in `rpr`.
- **Calls (crate):** `toggle` (edit.rs:8557) L8375,8376,8377,8378,8399,8400; `Dom::element` (xmllinq/mod.rs:655) L8380,8390,8402,8412,8421; `Dom::remove` (xmllinq/mod.rs:1118) L8381,8391,8403,8413,8422; `Dom::new_element` (xmllinq/mod.rs:478) L8384,8394,8405,8415,8424; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8385,8395,8407,8416,8425; `insert_rpr_child` (edit.rs:8544) L8386,8396,8409,8417,8426
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::name`, `W::val`
- **Called by:** `track_run_format` (edit/tracked.rs:924); `emit_text` (edit/tracked.rs:1035); `format_range` (edit.rs:8035); `build_paragraph` (edit.rs:8299); `deeper_boundary_tests::replacing_and_clearing_existing_run_properties` (edit.rs:10015)

#### `check_format`  (L8453-8491)
`fn check_format(format: &RunFormat, text: &str) -> Result<(), String>`

- **Does:** A format must name a real highlight colour and apply to some text.
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.bytes`, `.chars`, `.contains`, `.count`, `.into`, `.is_ascii_hexdigit`, `.is_empty`, `.join`, `.len`, `.trim`, `Err`, `Ok`, `RunFormat::default`, `from`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_format_run` (edit/runs.rs:17); `Transaction::resolve_one_unreported` (edit.rs:3545); `deeper_boundary_tests::format_validation_unicode_length_and_color_truth_tables` (edit.rs:10070)

#### `rank`  (L8536-8541)
`fn rank(local: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.position`, `.unwrap_or`
- **Called by:** `insert_ppr_child` (edit.rs:8182); `insert_rpr_child` (edit.rs:8544)

#### `insert_rpr_child`  (L8544-8554)
`fn insert_rpr_child(dom: &mut Dom, rpr: NodeId, child: NodeId)`

- **Does:** Insert `child` into `rpr` at its schema position.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L8545,8547; `rank` (edit.rs:8536) L8545,8548; `XName::local_name` (xmllinq/mod.rs:208) (by name) L8545,8548; `Dom::elements` (xmllinq/mod.rs:640) L8546; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L8548; `Dom::add_after_self` (xmllinq/mod.rs:1147) L8551; `Dom::add_first` (xmllinq/mod.rs:1101) L8552
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.map_or`, `.rev`
- **Called by:** `set_glyph_font` (edit/controls.rs:650); `apply_run_format` (edit.rs:8374); `toggle` (edit.rs:8557); `tests::rpr_children_are_inserted_in_schema_order` (edit.rs:8730); `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (edit.rs:9891)

#### `toggle`  (L8557-8567)
`fn toggle(dom: &mut Dom, rpr: NodeId, local: &str, value: Option<bool>)`

- **Does:** Set or clear a boolean run property (`w:b`, `w:i`, ...).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8559; `Dom::remove` (xmllinq/mod.rs:1118) L8560; `Dom::new_element` (xmllinq/mod.rs:478) L8562; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8564; `insert_rpr_child` (edit.rs:8544) L8566
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::val`
- **Called by:** `apply_run_format` (edit.rs:8374); `tests::rpr_children_are_inserted_in_schema_order` (edit.rs:8730); `definition` (markdown/view_styles.rs:939) (?)

## Test functions (44)

- `tests::plan_json_roundtrips` (L8575): _no doc comment_
- `tests::plan_json_rejects_unknown_fields` (L8591): _no doc comment_
- `tests::plan_json_rejects_malformed_json` (L8599): _no doc comment_
- `tests::plan_json_refuses_another_schema_version` (L8605): _no doc comment_
- `tests::plan_to_json_is_pretty_and_a_fixed_point` (L8616): _no doc comment_
- `tests::plan_fields_name_every_field_a_plan_serializes` (L8630): _no doc comment_
- `tests::nearest_names_the_closest_within_a_third` (L8663): _no doc comment_
- `tests::context_windows_and_excerpts` (L8676): _no doc comment_
- `tests::new_position_accounts_for_earlier_edits_and_insertions` (L8686): _no doc comment_
- `tests::check_text_rejects_controls` (L8722): _no doc comment_
- `tests::rpr_children_are_inserted_in_schema_order` (L8730): _no doc comment_
- `tests::error_display_names_the_operation` (L8754): _no doc comment_
- `deeper_boundary_tests::plan` (L8773): _no doc comment_
- `deeper_boundary_tests::dom` (L8779): _no doc comment_
- `deeper_boundary_tests::names` (L8789): _no doc comment_
- `deeper_boundary_tests::outcome` (L8796): _no doc comment_
- `deeper_boundary_tests::a_retried_insertion_point_leaves_no_refusal` (L8818): A retry that holds clears the refusal its first side stored, so a later failure of the same operation names no revision that did not block it (#1308 review).
- `deeper_boundary_tests::report_optional_fields_and_resolution_truth_table` (L8841): _no doc comment_
- `deeper_boundary_tests::half_points_finite_range_and_rounding_boundaries` (L8895): 702's two disjuncts, including values JSON cannot represent. _(from a code comment)_
- `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (L8914): 1055, 1069, 1073: no selection, empty filters, reject-only selection. _(from a code comment)_
- `deeper_boundary_tests::admission_boundaries_and_signature_prefix_are_exact` (L8956): 1652, 1669, 1677; validation precedes package parsing. _(from a code comment)_
- `deeper_boundary_tests::invalid_package_has_no_revisions_or_kept_comment_text` (L8998): 1376, 1389, 4519: invalid byte input is handled without panic. _(from a code comment)_
- `deeper_boundary_tests::resolution_empty_text_and_combined_comment_anchors` (L9005): 2259, 2312, 2339, 2503, 2703, 3173. _(from a code comment)_
- `deeper_boundary_tests::header_refuses_body_only_picture_toc_and_list` (L9066): Body-only operations with a real in-memory header story: 2432, 2688, 2987. _(from a code comment)_
- `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (L9102): 3109, 3114: a related style part can disappear/become rootless. _(from a code comment)_
- `deeper_boundary_tests::projection_range_and_insert_boundary_truth_tables` (L9129): 3336, 3342, 3368: end coverage, field-boundary equality, and glyph attachment. _(from a code comment)_
- `deeper_boundary_tests::text_range` (L9172): _no doc comment_
- `deeper_boundary_tests::conflicts` (L9184): _no doc comment_
- `deeper_boundary_tests::overlapping_range_and_comment_edge_truth_tables` (L9378): 3460/3461 and 3477: same-start replacements vs insertions, and either comment edge. _(from a code comment)_
- `deeper_boundary_tests::deleted_paragraph_rejects_each_anchor_owner` (L9435): All deletion conflict arms use explicit functional error assertions, not a call sweep. _(from a code comment)_
- `deeper_boundary_tests::table_insertion_sides_at_merge_boundaries` (L9555): 3586's Before/tail arm, with a table rather than the existing paragraph integration case. _(from a code comment)_
- `deeper_boundary_tests::comment_reference_identity_and_thread_conflicts` (L9595): 2804, 2842, 2851, 3667, 3680. _(from a code comment)_
- `deeper_boundary_tests::text_edit_glyphs_and_segment_attachment_are_exact` (L9679): 4697, 4713, 4726: insertion affinity and glyphs never turn into text nodes. _(from a code comment)_
- `deeper_boundary_tests::splitting_multichild_run_preserves_order_and_properties` (L9726): 4751, 4754, 4771, 4776: both edges, a glyph, and children on either side of t. _(from a code comment)_
- `deeper_boundary_tests::empty_comment_anchors_and_reference_cleanup` (L9760): 4798, 4830, 4840, 4867, 4903, 4957. _(from a code comment)_
- `deeper_boundary_tests::format_range_deduplicates_runs_and_removes_empty_properties` (L9811): 4975, 4992, 4994, 5009: no-op ranges, empty t, and two segments in one run. _(from a code comment)_
- `deeper_boundary_tests::paragraph_format_partial_spacing_preserves_other_axes` (L9833): 5032, 5041, 5058, 5063, 5067; only the requested spacing axis changes. _(from a code comment)_
- `deeper_boundary_tests::schema_insertion_ignores_foreign_and_later_properties` (L9891): 5125, 5482: higher-ranked and foreign children do not supply the insertion anchor. _(from a code comment)_
- `deeper_boundary_tests::merge_separator_copies_properties_without_revision_children` (L9935): 5142, 5148, 5151-5153: separator inherits clean properties only. _(from a code comment)_
- `deeper_boundary_tests::new_paragraph_style_and_revision_cleanup_truth_table` (L9968): 5238, 5244, 5245, 5276, 5285-5287: property cloning and empty run specs. _(from a code comment)_
- `deeper_boundary_tests::replacing_and_clearing_existing_run_properties` (L10015): 5314, 5317, 5324, 5327, 5336, 5346, 5355. _(from a code comment)_
- `deeper_boundary_tests::format_validation_unicode_length_and_color_truth_tables` (L10070): 5398, 5399, 5406, 5407, 5421. _(from a code comment)_
- `deeper_boundary_tests::whole_deletion_builds_no_bookmark_and_keeps_surrounding_text` (L10128): 4009: whole deletion skips replacement bookmarks. _(from a code comment)_
- `deeper_boundary_tests::revision_and_comment_scans_ignore_rootless_and_foreign_story_parts` (L10145): 1402, 1414, 4526: tolerate an unreadable story and ignore foreign revisions. _(from a code comment)_
