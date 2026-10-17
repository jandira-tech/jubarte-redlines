# Functions and methods in `src/changes.rs`

29 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `ChangeFilter::ids`  (L117-122, pub)
`pub fn ids<I: IntoIterator<Item = S>, S: Into<String>>(ids: I) -> Self`

- **Does:** Build a filter that selects the changes `ids` names (`r12`, `header1.r1`, `r1+2`).
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Self::default`, `Some`
- **Called by:** `validate_matches` (cli.rs:2566) (?); `Transaction::cell_control` (edit/controls.rs:298) (?); `plan_from_flags` (edit/flags.rs:76); `tests::recording_reads_ids_and_names_as_the_plan_spells_them` (edit/flags.rs:910); `tests::revisions_go_to_resolve_revisions` (edit/flags.rs:1351); `resolve_selected` (edit.rs:1161); `Transaction::select` (edit.rs:3778) (?); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:6789)

#### `ChangeFilter::normalized`  (L130-149, pub)
`pub fn normalized(&self) -> Result<Self, ChangeError>`

- **Does:** The filter with every id in its settled short spelling, the form [`Change::id`] holds and [`Self::check`] and [`Self::matches`] compare: `r12`, `header1.r1`, a pair `r1+2` as its two ids, each once.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L136; `ChangeId::short` (ids.rs:155) (by name) L139
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.len`, `.map_err`, `.push`, `.to_string`, `ChangeError::InvalidId`, `Ok`, `Some`, `Vec::with_capacity`
- **Called by:** `run_resolution` (bin/jubarte.rs:361); `resolve` (changes.rs:349); `record_into` (edit/flags.rs:574) (?); `record` (edit/record.rs:181) (?); `resolve_selected` (edit.rs:1161) (?)

#### `ChangeFilter::check`  (L159-174, pub)
`pub fn check(&self, listed: &[Change]) -> Result<(), ChangeError>`

- **Does:** The first error the filter's ids meet in `listed`: an id that names no change, or one that names several.
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.filter`, `.flatten`, `.into_iter`, `.iter`, `.len`, `ChangeError::UnknownChange`, `Err`, `Ok`
- **Called by:** `resolve` (changes.rs:349); `resolve_selected` (edit.rs:1161) (?)

#### `ChangeFilter::matches`  (L177-187, pub)
`pub fn matches(&self, change: &Change) -> bool`

- **Does:** Whether `change` is selected.
- **Calls (std/external/unresolved):** `.as_ref`, `.contains`, `.is_none_or`, `.is_some_and`
- **Called by:** `resolve` (changes.rs:349); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1371) (?); `tests::regression_tests::omitted_cell_and_row_end_tags_keep_the_table` (convert/altchunk.rs:875) (?); `tests::regression_tests::comments_scripts_and_styles_do_not_become_document_text` (convert/altchunk.rs:994) (?); `tests::regression_tests::html_tables_keep_spans_empty_cells_and_header_formatting` (convert/altchunk.rs:1026) (?); `tests::regression_tests::invalid_or_zero_colspan_falls_back_to_one_column` (convert/altchunk.rs:1047) (?); `collect_hf_runs` (convert/mod.rs:21523) (?); `Layout::emit_picture_break_lines` (convert/mod.rs:27797) (?); `converter_semantic_boundary_tests::attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent` (convert/mod.rs:55981) (?); `residual_byte_and_geometry_contract_tests::unicode_cmap_chunks_preserve_every_cid_and_utf16_surrogate_exactly` (convert/pdf.rs:2708) (?); `residual_byte_and_geometry_contract_tests::authored_width_scales_close_text_objects_without_leaking_into_following_glyphs` (convert/pdf.rs:2883) (?); `residual_byte_and_geometry_contract_tests::notdef_runs_carry_their_text_as_actual_text` (convert/pdf.rs:3023) (?); +19 more

#### `<ChangeError as fmt::Display>::fmt`  (L210-241, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `kind_name` (changes.rs:246) L235
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.collect`, `.iter`, `.join`, `.len`, `.map`, `.unwrap_or_default`, `.write_str`
- **Macros:** `format!`, `matches!`, `write!`
- **Called by:** none found in src

#### `kind_name`  (L246-253)
`fn kind_name(kind: ChangeKind) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `<ChangeError as fmt::Display>::fmt` (changes.rs:210); `changes_text` (changes.rs:259); `untracked_kind` (edit/tracked.rs:53) (?)

#### `changes_text`  (L259-279, pub)
`pub fn changes_text(changes: &[Change]) -> String`

- **Does:** `jubarte changes`' listing: one line per change, `{ref}\t{kind}\t{target} \t{author}\t{paragraph}\t{text}[\tinside {ref}]` (`-` for an absent author or paragraph; the text quoted, its first 60 characters), then `{n} change(s)`.
- **Calls (crate):** `kind_name` (changes.rs:246) L272
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.collect`, `.len`, `.map`, `.push_str`, `.take`, `.unwrap_or_default`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `run_changes` (bin/jubarte.rs:399)

#### `list_changes`  (L287-289, pub)
`pub fn list_changes(docx: &[u8]) -> Result<Vec<Change>, ChangeError>`

- **Does:** Every tracked change, story by story (body, headers, footers, endnotes, footnotes, styles), in document order.
- **Calls (crate):** `changes_in` (changes.rs:302) L288; `open` (changes.rs:391) L288
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_changes` (bin/jubarte.rs:399); `plan_from_flags` (edit/flags.rs:76); `resolution_lines` (edit/resolution.rs:236); `revision_counts` (edit/tracked.rs:237); `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:1422); `resolve_selected` (edit.rs:1161); `revisions_left` (edit.rs:1238); `revision_counts` (edit.rs:1755); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:6789); `patch_own_changes` (markdown/unified.rs:308); `audit_tracked` (validate.rs:2036)

#### `list_changes_tolerant`  (L294-298, pub(crate))
`pub(crate) fn list_changes_tolerant(docx: &[u8]) -> Result<Vec<Change>, ChangeError>`

- **Does:** [`list_changes`] for a view that already renders the package: no admission, and a part that does not parse lists nothing, so the view's count fails only where the package itself does.
- **Calls (crate):** `strict_to_transitional_docx` (strict_translation.rs:743) L295; `PartFs::open` (opc/mod.rs:456) L296; `changes_in` (changes.rs:302) L297
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `ChangeError::Package`, `Ok`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `changes_in`  (L302-322, pub(crate))
`pub(crate) fn changes_in(pkg: &PartFs) -> Vec<Change>`

- **Does:** The changes of an opened (transitional) package, as [`list_changes`] lists them.
- **Calls (crate):** `main_part` (changes.rs:398) L303; `revision_bearing_parts` (revision_processor.rs:3403) L305; `PartFs::part_string` (opc/mod.rs:498) L306; `Dom::new` (xmllinq/mod.rs:387) L309; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L310; `Dom::root` (xmllinq/mod.rs:822) L311; `story_id` (changes.rs:405) L314; `carriers` (changes.rs:441) L316
- **Calls (std/external/unresolved):** `.extend`, `.filter_map`, `.into_iter`, `Vec::new`
- **Called by:** `list_changes` (changes.rs:287); `list_changes_tolerant` (changes.rs:294); `resolve` (changes.rs:349); `settle` (edit.rs:1260); `snapshot` (inspect.rs:362); `Opened::summary` (inspect.rs:545); `table_line` (markdown/from_docx/agent.rs:1812)

#### `accept_changes`  (L335-337, pub)
`pub fn accept_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError>`

- **Does:** Accept the changes `filter` selects; the others stay tracked.
- **Calls (crate):** `resolve` (changes.rs:349) L336
- **Called by:** `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:1422); `resolve_selected` (edit.rs:1161); `patch_own_changes` (markdown/unified.rs:308)

#### `reject_changes`  (L345-347, pub)
`pub fn reject_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError>`

- **Does:** Reject the changes `filter` selects; the others stay tracked.
- **Calls (crate):** `resolve` (changes.rs:349) L346
- **Called by:** `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:1422); `resolve_selected` (edit.rs:1161); `audit_tracked` (validate.rs:2036)

#### `resolve`  (L349-388)
`fn resolve( docx: &[u8], filter: &ChangeFilter, resolution: Resolution, ) -> Result<Vec<u8>, ChangeError>`

- **Does:** _no doc comment_
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:130) L354; `open` (changes.rs:391) L355; `changes_in` (changes.rs:302) L356; `ChangeFilter::check` (changes.rs:159) L357; `ChangeFilter::matches` (changes.rs:177) L359,367; `main_part` (changes.rs:398) L362; `carriers` (changes.rs:441) L364; `story_id` (changes.rs:405) L364; `deleted_text` (changes.rs:792) L375; `recorded_marks` (changes.rs:707) L376; `Dom::set_name` (xmllinq/mod.rs:539) (by name) L381; `XName::get` (xmllinq/mod.rs:180) L381; `XName::local_name` (xmllinq/mod.rs:208) (by name) L381; `resolve_package` (revision_processor.rs:3495) L385; `PartFs::to_zip` (opc/mod.rs:548) (by name) L386
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.chain`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.is_some_and`, `.iter`, `.map_err`, `.map`, `.name`, `.to_string`, `.to_vec`, `ChangeError::Package`, `Ok`, `Some`, `std::iter::once`
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `accept_changes` (changes.rs:335); `reject_changes` (changes.rs:345)

#### `open`  (L391-396)
`fn open(docx: &[u8]) -> Result<PartFs, ChangeError>`

- **Does:** The package, Strict normalized to transitional as `inspect` reads it.
- **Calls (crate):** `admit_package` (document_comparer.rs:4924) L392; `strict_to_transitional_docx` (strict_translation.rs:743) L394; `PartFs::open` (opc/mod.rs:456) L395
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `ChangeError::Package`
- **Called by:** `list_changes` (changes.rs:287); `resolve` (changes.rs:349); `Origin::build` (edit/origin.rs:256) (?); `record_mid_changes` (edit/sections.rs:453) (?); `rewrite` (edit/whole.rs:130) (?); `collect_fields` (fields.rs:281) (?)

#### `main_part`  (L398-401)
`fn main_part(pkg: &PartFs) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L399
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`
- **Called by:** `changes_in` (changes.rs:302); `resolve` (changes.rs:349)

#### `story_id`  (L405-410)
`fn story_id(main: &str, part: &str) -> String`

- **Does:** `body` for the main part, else the part's file stem (`header1`, `footnotes`, `styles`).
- **Calls (crate):** `part_story` (ids.rs:392) L409
- **Calls (std/external/unresolved):** `.to_string`, `.trim_start_matches`
- **Called by:** `changes_in` (changes.rs:302); `resolve` (changes.rs:349)

#### `carriers`  (L441-557)
`fn carriers(dom: &Dom, root: NodeId, story: &str) -> Vec<Carrier>`

- **Does:** The revision elements under `root`, in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L448; `Dom::name` (xmllinq/mod.rs:526) L449; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L452; `XName::local_name` (xmllinq/mod.rs:208) (by name) L455; `Dom::attribute` (xmllinq/mod.rs:874) L456,459,526,527; `in_recorded_properties` (changes.rs:694) L492; `target` (changes.rs:768) L499; `paragraph_move` (changes.rs:731) L508; `owner_text` (changes.rs:816) L515; `text_of` (changes.rs:806) L516; `ChangeId::short` (ids.rs:155) (by name) L522,523; `ChangeId::new` (ids.rs:142) L522,523; `fold_twins` (changes.rs:568) L540; `fold_style_pairs` (changes.rs:630) L541; `paragraph_index` (changes.rs:665) L546; `holder` (changes.rs:743) L549; `paragraph_of` (changes.rs:676) L552; `paragraph_id` (ids.rs:410) L553
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.iter`, `.last`, `.map_or_else`, `.map_or`, `.map`, `.or`, `.push`, `.remove`, `.rposition`, `.then`, `.to_string`, `.unwrap_or`, `Group::Move`, `Group::Own`, `Some`, `String::new`, `Vec::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `changes_in` (changes.rs:302); `resolve` (changes.rs:349)

#### `fold_twins`  (L568-623)
`fn fold_twins(dom: &Dom, carriers: &mut [Carrier])`

- **Does:** Word stores a text box twice, in an `mc:AlternateContent`'s `mc:Choice` and its `mc:Fallback`, and gives the Fallback copy of each change its own consecutive `w:id`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L576,577,591; `Dom::name_is` (xmllinq/mod.rs:535) L585; `Dom::parent` (xmllinq/mod.rs:597) L598; `Dom::element` (xmllinq/mod.rs:655) L598; `key` (ambiguous: `same_path::{fn key}` (bin/jubarte.rs:159); `key` (markdown/patch.rs:310)) L613
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.sort_by_key`, `MC::name`, `Some`, `Vec::new`, `std::cmp::Reverse`, `under`
- **Called by:** `carriers` (changes.rs:441)

#### `fold_style_pairs`  (L630-661)
`fn fold_style_pairs(dom: &Dom, carriers: &mut [Carrier])`

- **Does:** Word records a style's paragraph and run formatting changes (`w:style/w:pPr/w:pPrChange`, `w:style/w:rPr/w:rPrChange`) under one `w:id`: one change.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L634,635; `Dom::name` (xmllinq/mod.rs:526) L637; `XName::local_name` (xmllinq/mod.rs:208) (by name) L638; `Dom::name_is` (xmllinq/mod.rs:535) L639; `change_id` (ids.rs:187) L650
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.get`, `.insert`, `.iter_mut`, `.map_or_else`, `.then_some`, `HashMap::new`, `W::name`, `own_style`
- **Called by:** `carriers` (changes.rs:441)

#### `paragraph_index`  (L665-671)
`fn paragraph_index(dom: &Dom, root: NodeId) -> HashMap<NodeId, usize>`

- **Does:** Each listed paragraph of the story under `root` (the body's or a header's, footer's or notes part's, as `inspect` numbers them) by node.
- **Calls (crate):** `story_paragraph_nodes` (inspect.rs:937) L666
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.into_iter`, `.map`
- **Called by:** `carriers` (changes.rs:441)

#### `paragraph_of`  (L676-690)
`fn paragraph_of(dom: &Dom, e: NodeId, index: &HashMap<NodeId, usize>) -> Option<usize>`

- **Does:** The paragraph a revision element sits in: the nearest listed paragraph around it (a text box's paragraphs are not listed, so its anchor's), or the first listed paragraph of the row, cell or table around it.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L677; `Dom::name` (xmllinq/mod.rs:526) L684; `Dom::descendants` (xmllinq/mod.rs:668) L686
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.copied`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `Some`, `W::p`, `W::tbl`, `W::tc`, `W::tr`, `std::iter::once`
- **Called by:** `carriers` (changes.rs:441); `Anchors::scan` (comments.rs:241) (?)

#### `in_recorded_properties`  (L694-699)
`fn in_recorded_properties(dom: &Dom, e: NodeId) -> bool`

- **Does:** True when `e` sits in the properties a formatting change records: a mark there is history, part of that change.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L695; `Dom::name` (xmllinq/mod.rs:526) L696; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L697; `XName::local_name` (xmllinq/mod.rs:208) (by name) L697
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`
- **Called by:** `carriers` (changes.rs:441)

#### `recorded_marks`  (L707-726)
`fn recorded_marks(dom: &Dom, e: NodeId) -> Vec<NodeId>`

- **Does:** What a formatting change `e` records: the revision marks inside it (a cell's `w:cellIns`, `w:cellDel` or `w:cellMerge` in its `w:tcPrChange`) and the one formatting history the schema allows there, a list's `w:numberingChange` in its `w:pPrChange`.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L708,717; `XName::local_name` (xmllinq/mod.rs:208) (by name) L710,720,722; `Dom::descendants` (xmllinq/mod.rs:668) L714; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L718
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some_and`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `resolve` (changes.rs:349)

#### `paragraph_move`  (L731-737)
`fn paragraph_move(dom: &Dom, e: NodeId, start: &str) -> Option<String>`

- **Does:** The move a paragraph mark `e` belongs to: the last range of its side (`start`) opening inside its paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L732; `Dom::descendants` (xmllinq/mod.rs:668) L733; `Dom::attribute` (xmllinq/mod.rs:874) L736
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.map`, `.next`, `.rev`, `Some`, `W::name`, `W::p`
- **Called by:** `carriers` (changes.rs:441)

#### `holder`  (L743-766)
`fn holder(dom: &Dom, e: NodeId, ids: &HashMap<NodeId, String>) -> Option<String>`

- **Does:** The id of the innermost change around `e` whose content holds it: an ancestor revision element, the row or cell an ancestor `w:tr` / `w:tc` marks inserted or deleted, or the mark of the paragraph whose `pPr` holds `e` (the paragraph's properties go with its mark).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L745; `Dom::name` (xmllinq/mod.rs:526) L749,763; `Dom::element` (xmllinq/mod.rs:655) L759; `Dom::elements` (xmllinq/mod.rs:640) L760; `XName::local_name` (xmllinq/mod.rs:208) (by name) L763
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.contains`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_some_and`, `Some`, `W::p_pr`, `W::r_pr`, `W::tc_pr`, `W::tc`, `W::tr_pr`, `W::tr`
- **Called by:** `carriers` (changes.rs:441)

#### `target`  (L768-787)
`fn target(dom: &Dom, e: NodeId, local: &str) -> &'static str`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L775,776,778; `Dom::name` (xmllinq/mod.rs:526) L775,779
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.starts_with`, `Some`, `W::p_pr`, `W::r_pr`, `W::tr_pr`
- **Called by:** `carriers` (changes.rs:441); `carry_with_own_relationships` (comparer/parts.rs:627) (?); `tests::rel_attributes_survive_round_trips_unescaped` (opc/mod.rs:1169) (?)

#### `deleted_text`  (L792-803)
`fn deleted_text(dom: &Dom, e: NodeId) -> Vec<NodeId>`

- **Does:** The `w:delText` / `w:delInstrText` a `w:del` holds itself (not through a nested `w:del`): the resolution rewrites deleted text wherever it sits, so a kept deletion keeps its text only when that text sits out too.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L794,800; `Dom::descendants` (xmllinq/mod.rs:668) L798; `Dom::ancestors` (xmllinq/mod.rs:770) L801
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.first`, `.into_iter`, `.is_some_and`, `Some`, `Vec::new`, `W::del`, `W::name`
- **Called by:** `resolve` (changes.rs:349)

#### `text_of`  (L806-813)
`fn text_of(dom: &Dom, e: NodeId) -> String`

- **Does:** Inserted, deleted or moved text under `e`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L808; `Dom::name` (xmllinq/mod.rs:526) L810; `Dom::value` (xmllinq/mod.rs:1169) L811
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `W::name`, `W::t`
- **Called by:** `carriers` (changes.rs:441); `owner_text` (changes.rs:816); `split_head_short_title_long_del_mix` (comparer/finalize.rs:4829) (?); `change_lines` (debug.rs:1310) (?); `compare_documents_impl` (document_comparer.rs:6501) (?); `ParaFormat::read` (markdown/view_stories.rs:81) (?); `story` (markdown/view_stories.rs:384) (?)

#### `owner_text`  (L816-822)
`fn owner_text(dom: &Dom, change: NodeId) -> String`

- **Does:** The text of the run or paragraph whose properties a change records.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L817,818; `Dom::name` (xmllinq/mod.rs:526) L819; `text_of` (changes.rs:806) L820
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_some_and`, `.map`, `.unwrap_or_default`, `W::p`, `W::r`
- **Called by:** `carriers` (changes.rs:441)
