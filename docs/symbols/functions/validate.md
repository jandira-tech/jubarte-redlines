# Functions and methods in `src/validate.rs`

107 non-test, 44 test. Part of the [symbol manifest](../README.md).

#### `Finding::new`  (L133-158, pub)
`pub fn new( code: &str, part: &str, location: impl Into<String>, message: impl Into<String>, ) -> Self`

- **Does:** A `word` finding with `word_fatal` and `repairable` taken from the code's table ([`traits`]).
- **Calls (crate):** `traits` (validate.rs:163) L139
- **Calls (std/external/unresolved):** `.clone`, `.into`, `.to_string`
- **Called by:** `findings` (debug.rs:2575); `check_data_bindings_have_stores` (validate.rs:1189); `check_move_range_counts` (validate.rs:1229); `check_nested_text_boxes` (validate.rs:1269); `check_style_names` (validate.rs:1338); `check_runs_inside_paragraphs` (validate.rs:1371); `check_content_types_and_xml` (validate.rs:1468); `check_relationship_integrity` (validate.rs:1548); `check_picture_bullets` (validate.rs:1645); `collect_ids` (validate.rs:1718); `check_para_text_id_bounds` (validate.rs:1807); `check_del_text_under_del` (validate.rs:1831); +10 more

#### `traits`  (L163-198, pub)
`pub fn traits(code: &str) -> (bool, bool)`

- **Does:** `(word_fatal, repairable)` for a code; an unknown code is a lead with no repair.
- **Calls:** none
- **Called by:** `Finding::new` (validate.rs:133)

#### `<ValidateError as fmt::Display>::fmt`  (L253-262, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L268-270, impl From)
`fn from(e: AdmissionError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Refused`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L274-276, impl From)
`fn from(e: ChangeError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Changes`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L280-282, impl From)
`fn from(e: InspectError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Inspect`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L286-288, impl From)
`fn from(e: crate::opc::OpcError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `Self::Package`
- **Called by:** none found in src

#### `validate_with`  (L316-357, pub)
`pub fn validate_with( docx: &[u8], rules: &[&str], original: Option<(&[u8], &str)>, layout: Option<crate::audit::LayoutPass<'_>>, ) -> Result<Vec<Finding>, ValidateError>`

- **Does:** [`validate`] with rule sets: `word` (the default), `edits`, and the [`crate::audit`] sets or codes (`a11y`, `style`, `structure`, `LITERAL_BULLET`, ...).
- **Calls (crate):** `is_rule` (audit.rs:156) L329; `unknown_rule` (validate.rs:394) L330,351; `validate` (validate.rs:1140) L335; `audit_tracked` (validate.rs:3027) L343; `audit_report_with` (audit.rs:177) L347
- **Calls (std/external/unresolved):** `.extend`, `.into_iter`, `.is_empty`, `.is_some`, `.map_err`, `.map`, `.push`, `.to_string`, `Err`, `Ok`, `ValidateError::Inspect`, `ValidateError::Layout`, `ValidateError::Usage`, `Vec::new`
- **Called by:** `run_validate` (bin/jubarte.rs:3812); `validate_baselined` (validate.rs:370)

#### `validate_baselined`  (L370-391, pub)
`pub fn validate_baselined( docx: &[u8], base: &[u8], rules: &[&str], original: Option<&[u8]>, author: Option<&str>, mut layout: Option<crate::audit::LayoutPass<'_>>, ) -> Result<Baselined, ValidateError>`

- **Does:** [`validate_with`] on `docx`, split by the findings of `base`, the document the edits started from, under the same rules and against the same original ([`baseline_keyed`], each side's keys read in its own document by [`identities`]).
- **Calls (crate):** `validate_with` (validate.rs:316) L379,387; `identities` (validate.rs:523) L388,389; `baseline_keyed` (validate.rs:878) L390
- **Calls (std/external/unresolved):** `.as_mut`, `.map`, `.or`, `.zip`, `Ok`, `Some`
- **Called by:** `run_validate` (bin/jubarte.rs:3812)

#### `unknown_rule`  (L394-401)
`fn unknown_rule(rule: &str) -> String`

- **Does:** The usage message for a rule that is neither a set nor an audit code.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `validate_with` (validate.rs:316)

#### `fails`  (L405-411, pub)
`pub fn fails(findings: &[Finding], strict: bool) -> bool`

- **Does:** True when `findings` fail a run: any `word` or `edits` finding, any `error`, or (`strict`) any `warning`.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.iter`
- **Macros:** `matches!`
- **Called by:** `run_validate` (bin/jubarte.rs:3812)

#### `findings_text`  (L419-437, pub)
`pub fn findings_text(findings: &[Finding]) -> String`

- **Does:** One line per finding, then a summary.
- **Calls (crate):** `finding_line` (validate.rs:440) L422; `limit_test_support::count` (admission.rs:2720) L431,432,433
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.iter`, `.len`, `.push_str`, `String::new`
- **Macros:** `format!`
- **Called by:** `run_validate` (bin/jubarte.rs:3812)

#### `finding_line`  (L440-451)
`fn finding_line(f: &Finding) -> String`

- **Does:** One [`findings_text`] line, newline included.
- **Calls (std/external/unresolved):** `.as_str`, `.clone`
- **Macros:** `format!`, `matches!`
- **Called by:** `findings_text` (validate.rs:419); `baselined_text` (validate.rs:965); `baseline_tests::baselined_text_and_json_name_each_list` (validate.rs:4818)

#### `Finding::identity`  (L504-513, pub)
`pub fn identity(&self) -> FindingKey`

- **Does:** This finding's [`FindingKey`]: [`baseline`] matches findings by it.
- **Calls (crate):** `normalised` (validate.rs:797) L510
- **Calls (std/external/unresolved):** `.clone`, `Place::default`
- **Called by:** `identities` (validate.rs:523) (?); `baseline_tests::identity_drops_element_indices_and_paragraph_ids_from_the_message` (validate.rs:4744) (?); `baseline_tests::identity_keeps_where_the_finding_sits` (validate.rs:4784) (?)

#### `identities`  (L523-547, pub)
`pub fn identities(docx: &[u8], findings: &[Finding]) -> Vec<FindingKey>`

- **Does:** Each of `findings`' [`FindingKey`], read in `docx`, the document they were found in.
- **Calls (crate):** `Anchors::new` (validate.rs:558) L524; `Finding::identity` (validate.rs:504) (by name) L528; `Anchors::change` (validate.rs:568) L531; `normalised` (validate.rs:797) L533; `Anchors::place` (validate.rs:593) L540
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.split_once`, `.to_string`, `Place::default`, `Some`
- **Called by:** `validate_baselined` (validate.rs:370)

#### `Anchors::new`  (L558-565)
`fn new(docx: &'a [u8]) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L561
- **Calls (std/external/unresolved):** `.ok`, `HashMap::new`
- **Called by:** `identities` (validate.rs:523)

#### `Anchors::change`  (L568-588)
`fn change(&mut self, id: &str) -> Option<String>`

- **Does:** `kind, author, text` of change `id` (`r3`).
- **Calls (crate):** `list_changes` (changes.rs:564) L572
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.get_or_insert_with`, `.get`, `.into_iter`, `.map`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?); `identities` (validate.rs:523)

#### `Anchors::place`  (L593-615)
`fn place(&mut self, part: &str, location: &str) -> Option<(String, Place)>`

- **Does:** The location `location` in `part` keys by, and its [`Place`]; `None` when it is neither an element chain of the part nor one of the part's paragraph ids.
- **Calls (crate):** `parse_part` (validate.rs:1442) L601; `Part::new` (validate.rs:633) L601; `paragraph_place` (ids.rs:417) L611
- **Calls (std/external/unresolved):** `.around`, `.as_mut`, `.as_ref`, `.collect`, `.contains`, `.entry`, `.get`, `.is_empty`, `.join`, `.map_or`, `.map`, `.or_insert_with`, `.paragraphs`, `.resolve`, `.split_once`, `.split`, `.to_string`, `Some`
- **Called by:** `identities` (validate.rs:523)

#### `Part::new`  (L633-641)
`fn new(dom: Dom, root: NodeId) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `HashMap::new`
- **Called by:** `Anchors::place` (validate.rs:593); `baseline_tests::a_chain_resolves_to_the_element_it_names` (validate.rs:4727)

#### `Part::resolve`  (L644-663)
`fn resolve(&mut self, chain: &str) -> Option<NodeId>`

- **Does:** The element `chain` ([`element_path`]) names under the root.
- **Calls (crate):** `well_known_namespace` (xmllinq/serialize.rs:190) L657; `named` (validate.rs:748) L660
- **Calls (std/external/unresolved):** `.get`, `.ok`, `.parse`, `.split_once`, `.split`, `.strip_suffix`, `Some`
- **Called by:** `tests::resolve_keeps_quoted_css_list_intact` (convert/font.rs:4187) (?); `tests::resolve_quoted_cambria_still_finds_cambria` (convert/font.rs:4208) (?); `tests::resolve_dejavu_sans_mono_follows_word_subst_verdana` (convert/font.rs:4221) (?); `tests::resolve_unknown_family_is_cambria_not_calibri` (convert/font.rs:4231) (?); `tests::resolve_wide_latin_stays_calibri_mini_505` (convert/font.rs:4262) (?); `tests::resolve_empty_family_is_times_new_roman` (convert/font.rs:4272) (?); `tests::resolve_prefers_embedded_unknown_family` (convert/font.rs:4798) (?); `tests::resolve_embedded_does_not_steal_unrelated_families` (convert/font.rs:4838) (?); `tests::font_table_scope_is_restored_after_a_panicking_conversion` (convert/font.rs:5231) (?); `tests::nested_font_table_scope_restores_outer_table` (convert/font.rs:5328) (?); `chrome_part_xml` (convert/mod.rs:20786) (?); `hf_pic_trails_line` (convert/mod.rs:21893) (?); +45 more

#### `Part::paragraphs`  (L668-684)
`fn paragraphs(&mut self, story: &str) -> &[NodeId]`

- **Does:** The paragraphs of `story`, this part's, in the order its paragraph ids (`p3`, `header1.p2`) count them: the order `inspect` and the audit number them in.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) (by name) L677; `body_paragraph_nodes` (inspect.rs:988) L678; `story_paragraph_nodes` (inspect.rs:1178) L681
- **Calls (std/external/unresolved):** `.get_or_insert_with`, `.map`, `.unwrap_or_default`, `W::body`
- **Called by:** none found in src

#### `Part::around`  (L688-743)
`fn around(&mut self, node: NodeId) -> Place`

- **Does:** The [`Place`] of `node`: what it sits in, read from its paragraph (else its table, else itself), and the change around it.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L696; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L700,710; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L702,703,711; `XName::local_name` (xmllinq/mod.rs:208) (by name) L704,724; `signature` (validate.rs:777) L708; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L716; `named` (validate.rs:748) L718; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L723; `text` (ambiguous: `revised_two_blank_table_titles::{fn text}` (comparer/finalize.rs:3891); `coverage_tests::text` (comparer/finalize.rs:15535); `coverage_real_revision_merge_matrix_tests::text` (comparer/finalize.rs:24885)) L730,731
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.checked_sub`, `.collect`, `.copied`, `.find_map`, `.find`, `.get`, `.iter`, `.map`, `.name`, `.value`, `Some`, `W14::name`, `W::author`, `W::id`, `W::p`, `W::tbl`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** none found in src

#### `named`  (L748-772)
`fn named<'c>( dom: &Dom, children: &'c mut HashMap<(NodeId, String, String), Vec<NodeId>>, rank: &mut HashMap<NodeId, usize>, parent: NodeId, namespace: &str, local: &str, ) -> &'c [NodeId]`

- **Does:** `parent`'s child elements named `local` in `namespace`, listed once into `children`, each one's place in the list into `rank`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L759; `Dom::name` (xmllinq/mod.rs:526) L763; `XName::local_name` (xmllinq/mod.rs:208) (by name) L764; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L764
- **Calls (std/external/unresolved):** `.collect`, `.entry`, `.enumerate`, `.filter`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.or_insert_with`, `.to_string`
- **Called by:** `Task::cannot_write` (cli.rs:1971) (?); `Part::resolve` (validate.rs:644); `Part::around` (validate.rs:688)

#### `signature`  (L777-792)
`fn signature(dom: &Dom, held: &[NodeId]) -> String`

- **Does:** The text of `held[0]`, then the attributes of it and of each element holding it (`held[1..]`), but namespace declarations and the ones an edit renumbers (`rsid…`, `paraId`, `textId`, `id`).
- **Calls (crate):** `Dom::value` (xmllinq/mod.rs:1185) L778; `Dom::attributes` (xmllinq/mod.rs:899) L780; `XName::local_name` (xmllinq/mod.rs:208) (by name) L781; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L782
- **Calls (std/external/unresolved):** `.first`, `.map`, `.push_str`, `.starts_with`, `.unwrap_or_default`
- **Macros:** `format!`, `matches!`
- **Called by:** `Part::around` (validate.rs:688)

#### `normalised`  (L797-827)
`fn normalised(message: &str) -> String`

- **Does:** `message` with each `[N]` element index dropped and each paragraph or cell id ([`is_paragraph_id`]) spelled `p#`, so an inherited finding keeps its identity when an edit renumbers what it names in passing.
- **Calls (crate):** `is_paragraph_id` (validate.rs:833) L818
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.find`, `.is_ascii_alphanumeric`, `.len`, `.push_str`, `.push`, `.starts_with`, `.strip_prefix`, `.take_while`, `.trim_end_matches`, `.unwrap_or`, `String::with_capacity`
- **Called by:** `Finding::identity` (validate.rs:504); `identities` (validate.rs:523); `baseline_tests::identity_drops_element_indices_and_paragraph_ids_from_the_message` (validate.rs:4744)

#### `is_paragraph_id`  (L833-844)
`fn is_paragraph_id(token: &str) -> bool`

- **Does:** True for a paragraph or cell id `-p` takes (`p12`, `header1.p2`, `t0.row1.cell2`, `t0.row1.cell2.p1`): a position, which an edit renumbers.
- **Calls (crate):** `coverage_round_four_tests::numbered` (comparer/lcs.rs:12990) (by name) L839; `parse_location` (ids.rs:326) L841
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.next`, `.rsplit`, `.strip_prefix`, `.unwrap_or`, `Ok`, `crate::ids::Location::Paragraph`
- **Macros:** `matches!`
- **Called by:** `normalised` (validate.rs:797)

#### `baseline`  (L864-868, pub)
`pub fn baseline(new: Vec<Finding>, base: Vec<Finding>) -> Baselined`

- **Does:** Splits `new` (the findings of an edited document) by those of `base` (the document before the edits), matched by [`Finding::identity`]: each baseline finding vouches for one finding of `new` with its identity, so a second finding with the same identity is new, and a finding that replaced another is new while the one i...
- **Calls (crate):** `baseline_keyed` (validate.rs:878) L867
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `baseline_tests::baseline_matches_one_for_one_by_identity` (validate.rs:4794)

#### `baseline_keyed`  (L878-919, pub)
`pub fn baseline_keyed( new: Vec<Finding>, new_keys: &[FindingKey], base: Vec<Finding>, base_keys: &[FindingKey], ) -> Baselined`

- **Does:** [`baseline`] with each finding's key given, in step with its list: [`identities`] reads them in the documents themselves.
- **Calls (crate):** `tier` (validate.rs:927) L890,897
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.entry`, `.enumerate`, `.filter_map`, `.get_mut`, `.into_iter`, `.iter`, `.len`, `.or_default`, `.push`, `.rev`, `.then_some`, `.zip`, `Baselined::default`, `HashMap::new`
- **Macros:** `vec!`
- **Called by:** `validate_baselined` (validate.rs:370); `baseline` (validate.rs:864)

#### `tier`  (L927-959)
`fn tier(pass: usize, key: &FindingKey) -> Option<([&str; 5], [Option<&str>; 5])>`

- **Does:** What pass `pass` of [`baseline_keyed`] matches `key` by: the key but its [`Place`], and the parts of the place the pass reads; `None` when `key` lacks what the pass needs.
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.is_some`, `Some`
- **Called by:** `baseline_keyed` (validate.rs:878)

#### `baselined_text`  (L965-996, pub)
`pub fn baselined_text(split: &Baselined) -> String`

- **Does:** [`baseline`]'s split as `validate --baseline` prints it: `new: none` or `new: N` and the new findings ([`findings_text`]'s lines), then `already in the input: N (not yours)` (or `none`), then, when the edits fixed some, `resolved: N (in the input, not in this file)` and those.
- **Calls (crate):** `limit_test_support::count` (admission.rs:2720) L973,979; `finding_line` (validate.rs:440) L975,992
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.to_string`
- **Macros:** `format!`
- **Called by:** `run_validate` (bin/jubarte.rs:3812); `baseline_tests::baselined_text_and_json_name_each_list` (validate.rs:4818)

#### `baselined_json`  (L1006-1025, pub)
`pub fn baselined_json(split: &Baselined) -> Result<String, serde_json::Error>`

- **Does:** [`baseline`]'s split as JSON Lines: each finding of the document, new ones first, as its JSON object with `"status": "new"` or `"baseline"`, then one `{"resolved": [...]}` line with the baseline's findings the document no longer has (an empty array when there are none).
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `Ok`, `String::new`, `serde_json::to_string`
- **Macros:** `serde_json::json!`
- **Called by:** `run_validate` (bin/jubarte.rs:3812); `baseline_tests::baselined_text_and_json_name_each_list` (validate.rs:4818)

#### `input_notice`  (L1032-1034, pub)
`pub fn input_notice(findings: &[Finding], file: &str) -> Option<String>`

- **Does:** The one stderr line `read`, `changes` and `comments` give an input with validity findings: `input has 2 validity warnings; jubarte validate FILE lists them` (an `error` counts as an error, `info` not at all), or `None` when there is nothing to say.
- **Calls (crate):** `input_notice_of` (validate.rs:1038) L1033; `input_validity` (validate.rs:1076) L1033
- **Called by:** `read_input` (bin/jubarte.rs:3069); `baseline_tests::input_notice_counts_errors_and_warnings_but_not_info` (validate.rs:4852)

#### `input_notice_of`  (L1038-1055, pub)
`pub fn input_notice_of(validity: &InputValidity) -> Option<String>`

- **Does:** [`input_notice`] from counts already taken, for a read that also reports them in its JSON (`read -o FILE.json`).
- **Calls (std/external/unresolved):** `.as_deref`, `Some`, `spell`
- **Macros:** `format!`
- **Called by:** `run_for_edit` (bin/jubarte.rs:961); `run_text` (bin/jubarte.rs:1036); `input_notice` (validate.rs:1032)

#### `input_validity`  (L1076-1085, pub)
`pub fn input_validity(findings: &[Finding], file: &str) -> InputValidity`

- **Does:** What [`InputValidity`] says of `findings`; `file` is the path as given, quoted for a shell when it needs to be ([`crate::util::shell_word`]).
- **Calls (crate):** `limit_test_support::count` (admission.rs:2720) L1078; `shell_word` (util/mod.rs:35) L1083
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `.then`
- **Macros:** `format!`
- **Called by:** `input_notice` (validate.rs:1032); `input_validity_of` (validate.rs:1091)

#### `input_validity_of`  (L1091-1093, pub)
`pub fn input_validity_of(docx: &[u8], file: &str) -> InputValidity`

- **Does:** [`input_validity`] of the package `docx` read from `file`, as the CLI and the bindings report it: a package that cannot be validated has no findings to count.
- **Calls (crate):** `input_validity` (validate.rs:1076) L1092; `validate` (validate.rs:1140) L1092
- **Calls (std/external/unresolved):** `.unwrap_or_default`
- **Called by:** `read_input_json` (bin/jubarte.rs:3099)

#### `input_validity_line`  (L1101-1106, pub)
`pub fn input_validity_line(validity: &InputValidity) -> Result<String, serde_json::Error>`

- **Does:** The last line `changes --json` and `comments --json` print: `{"input_validity":{"errors":0,"warnings":2,"next":"jubarte validate FILE"}}`, its keys in that order.
- **Calls (std/external/unresolved):** `Ok`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `validity_line` (bin/jubarte.rs:557)

#### `<Finding as From>::from`  (L1112-1130, impl From)
`fn from(f: crate::audit::AuditFinding) -> Self`

- **Does:** The audit's finding in the shared shape: its part is the story's part for a paragraph id (`p3` → `word/document.xml`), else the location itself (a part name); it is never Word-fatal or repairable.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L1113; `story_part` (validate.rs:2997) L1114
- **Calls (std/external/unresolved):** `.clone`, `.is_some`
- **Called by:** none found in src

#### `validate`  (L1140-1161, pub)
`pub fn validate(docx: &[u8]) -> Result<Vec<Finding>, ValidateError>`

- **Does:** Findings sorted by part, location, code and message (the same order every run); an empty vector is a pass.
- **Calls (crate):** `admit` (admission.rs:439) L1141; `<InputLimits as Default>::default` (admission.rs:64) L1141; `PartFs::open` (opc/mod.rs:585) L1142; `ring1` (validate.rs:1165) L1143; `findings` (debug.rs:2575) L1144
- **Calls (std/external/unresolved):** `.as_str`, `.cmp`, `.contains`, `.extend`, `.filter`, `.into_iter`, `.map_err`, `.sort_by`, `Ok`
- **Called by:** `read_input` (bin/jubarte.rs:3069); `check_candidates` (edit/prepared.rs:397); `tests::unvalidatable_source_never_claims_findings_introduced` (edit/prepared.rs:1157); `scrub` (scrub.rs:109); `validate_with` (validate.rs:316); `input_validity_of` (validate.rs:1091); `repair` (validate.rs:2623); `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (validate.rs:4028); `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (validate.rs:4067); `retained_namespace_finding_source_contract_tests::repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content` (validate.rs:4448); `duplicate_drawing_owned_repair_contract_tests::duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner` (validate.rs:4551)

#### `ring1`  (L1165-1183, pub)
`pub fn ring1(pkg: &PartFs) -> Vec<Finding>`

- **Does:** The Ring-1 invariants alone, on an opened package: no admission, no triage checks.
- **Calls (crate):** `check_content_types_and_xml` (validate.rs:1468) L1167; `check_relationship_integrity` (validate.rs:1548) L1168; `check_picture_bullets` (validate.rs:1645) L1169; `check_revision_and_drawing_ids` (validate.rs:1669) L1170; `check_para_text_id_bounds` (validate.rs:1807) L1171; `check_del_text_under_del` (validate.rs:1831) L1172; `check_deleted_text_has_deletion` (validate.rs:1897) L1173; `check_bookmarks_outside_single_value_controls` (validate.rs:1961) L1174; `check_comment_graph` (validate.rs:2067) L1175; `check_namespace_qname_contexts` (validate.rs:1462) L1176; `check_runs_inside_paragraphs` (validate.rs:1371) L1177; `check_style_names` (validate.rs:1338) L1178; `check_nested_text_boxes` (validate.rs:1269) L1179; `check_move_range_counts` (validate.rs:1229) L1180; `check_data_bindings_have_stores` (validate.rs:1189) L1181
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `warn_nested_text_boxes` (bin/jubarte.rs:1121); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23895); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26112); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2090); `tests::only_rows_marked_as_headers_repeat_as_headers` (legacy_doc.rs:2207); `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2368); `tests::a_heading_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2399); `tests::title_and_headings_7_to_9_keep_their_word_styles` (legacy_doc.rs:2528); `validate` (validate.rs:1140); `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (validate.rs:4028); `authored_comment_family_validation_contract_tests::auxiliary_comment_parts_without_their_source_definition_are_reported_individually` (validate.rs:4330)

#### `check_data_bindings_have_stores`  (L1189-1225)
`fn check_data_bindings_have_stores(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Word fills a bound content control from the custom XML store its `w:storeItemID` names, and shows the control empty when the main part reaches no such store (#1427).
- **Calls (crate):** `main_part` (validate.rs:2055) L1191; `xml_parts` (validate.rs:1450) L1195; `PartFs::part_bytes` (opc/mod.rs:628) L1197; `parse_part` (validate.rs:1442) L1203; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1206; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1207; `built_in` (ambiguous: `built_in` (markdown/data_stores.rs:186); `built_in` (markdown/package.rs:806)) L1210; `stores` (markdown/data_stores.rs:155) (by name) L1212; `Finding::new` (validate.rs:133) L1217; `element_path` (validate.rs:1418) L1220
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.get_or_insert_with`, `.is_some_and`, `.push`, `.starts_with`, `.to_ascii_uppercase`, `.windows`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `check_move_range_counts`  (L1229-1264)
`fn check_move_range_counts(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Word opens unequal per-name move ranges cleanly, but Reject All can restore the destination over only one source range and duplicate text.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1230; `parse_part` (validate.rs:1442) L1231; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1236; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1237; `Finding::new` (validate.rs:133) L1255; `element_path` (validate.rs:1418) L1258
- **Calls (std/external/unresolved):** `.entry`, `.filter`, `.is_empty`, `.or_insert`, `.push`, `.to_string`, `Some`, `W::name`, `std::collections::BTreeMap::new`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `check_nested_text_boxes`  (L1269-1310)
`fn check_nested_text_boxes(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Word crashes or refuses a box in another box's readable content, even though OpenXmlValidator passes it.
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L1270; `xml_parts` (validate.rs:1450) L1275; `parse_part` (validate.rs:1442) L1276; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1279; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L1286; `Finding::new` (validate.rs:133) L1297; `element_path` (validate.rs:1418) L1300; `text_box_name` (validate.rs:1313) L1303,1304
- **Calls (std/external/unresolved):** `.any`, `.find`, `.is_some_and`, `.iter`, `.name`, `.push`, `MC::name`, `VML::name`, `W::name`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `text_box_name`  (L1313-1332)
`fn text_box_name(dom: &Dom, node: NodeId) -> String`

- **Does:** Prefer the shape's own name, with its stable element path as a fallback.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1314; `Dom::name` (xmllinq/mod.rs:526) L1315,1324; `Dom::descendants` (xmllinq/mod.rs:668) L1316; `Dom::attribute` (xmllinq/mod.rs:874) L1319,1325; `XName::get` (xmllinq/mod.rs:180) L1319,1325; `element_path` (validate.rs:1418) L1331
- **Calls (std/external/unresolved):** `.filter`, `.find_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.to_string`, `Some`, `VML::name`, `W::name`, `WP::name`
- **Called by:** `check_nested_text_boxes` (validate.rs:1269)

#### `check_style_names`  (L1338-1364)
`fn check_style_names(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** A style's `w:name` holds no comma: the schema's pattern is `[^,]*` (Word reads a comma there as the start of an alias list), while its `w:styleId` may hold one.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1339; `parse_part` (validate.rs:1442) L1340; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1343,1350; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1346; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L1347; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1351; `Finding::new` (validate.rs:133) L1355; `element_path` (validate.rs:1418) L1358
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_none_or`, `.is_some_and`, `.name`, `.push`, `Some`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `check_runs_inside_paragraphs`  (L1371-1414)
`fn check_runs_inside_paragraphs(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** A run outside any paragraph: Word loads the document empty.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1385; `parse_part` (validate.rs:1442) L1386; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1389; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L1390,1402; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L1393,1398; `Finding::new` (validate.rs:133) L1405; `element_path` (validate.rs:1418) L1408
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.push`, `Some`, `W::name`, `W::p`, `W::r`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `element_path`  (L1418-1440, pub(crate))
`pub(crate) fn element_path(dom: &Dom, node: NodeId) -> String`

- **Does:** `w:body[0]/w:p[3]/w:r[2]`: conventional prefix and local name with the index among same-named siblings, root excluded.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1422,1423; `Dom::name` (xmllinq/mod.rs:526) L1426; `Dom::elements` (xmllinq/mod.rs:640) L1427; `well_known_prefix` (xmllinq/serialize.rs:181) L1432; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1432; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1435
- **Calls (std/external/unresolved):** `.is_none`, `.iter`, `.join`, `.map`, `.position`, `.push`, `.reverse`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Findings::add_at` (debug.rs:368); `check_data_bindings_have_stores` (validate.rs:1189); `check_move_range_counts` (validate.rs:1229); `check_nested_text_boxes` (validate.rs:1269); `text_box_name` (validate.rs:1313); `check_style_names` (validate.rs:1338); `check_runs_inside_paragraphs` (validate.rs:1371); `check_picture_bullets` (validate.rs:1645); `collect_ids` (validate.rs:1718); `check_del_text_under_del` (validate.rs:1831); `check_deleted_text_has_deletion` (validate.rs:1897); `check_bookmarks_outside_single_value_controls` (validate.rs:1961); +3 more

#### `parse_part`  (L1442-1448)
`fn parse_part(pkg: &PartFs, name: &str) -> Option<(Dom, NodeId)>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L1443; `Dom::new` (xmllinq/mod.rs:387) L1444; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1445; `Dom::root` (xmllinq/mod.rs:822) L1446
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Package::read` (audit.rs:354) (?); `Anchors::place` (validate.rs:593); `check_data_bindings_have_stores` (validate.rs:1189); `check_move_range_counts` (validate.rs:1229); `check_nested_text_boxes` (validate.rs:1269); `check_style_names` (validate.rs:1338); `check_runs_inside_paragraphs` (validate.rs:1371); `check_picture_bullets` (validate.rs:1645); `check_revision_and_drawing_ids` (validate.rs:1669); `check_del_text_under_del` (validate.rs:1831); `check_deleted_text_has_deletion` (validate.rs:1897); `check_comment_graph` (validate.rs:2067); +6 more

#### `xml_parts`  (L1450-1455)
`fn xml_parts(pkg: &PartFs) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1451
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`, `.into_iter`
- **Called by:** `check_data_bindings_have_stores` (validate.rs:1189); `check_move_range_counts` (validate.rs:1229); `check_nested_text_boxes` (validate.rs:1269); `check_style_names` (validate.rs:1338); `check_runs_inside_paragraphs` (validate.rs:1371); `check_namespace_qname_contexts` (validate.rs:1462); `check_revision_and_drawing_ids` (validate.rs:1669); `check_para_text_id_bounds` (validate.rs:1807); `check_del_text_under_del` (validate.rs:1831); `check_deleted_text_has_deletion` (validate.rs:1897); `check_bookmarks_outside_single_value_controls` (validate.rs:1961); `renumber_out_of_range_para_ids` (validate.rs:2886); +1 more

#### `check_namespace_qname_contexts`  (L1462-1466)
`fn check_namespace_qname_contexts(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Every prefix an MC prefix list names (`mc:Choice/@Requires`, `mc:Ignorable`, …) is bound where it is named, in every XML part: Word rejects a header whose `mc:Choice Requires="wps"` names an unbound `wps`.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1463; `check_namespace_qname_context` (validate.rs:2562) L1464
- **Called by:** `ring1` (validate.rs:1165)

#### `check_content_types_and_xml`  (L1468-1544)
`fn check_content_types_and_xml(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1469; `PartFs::content_type_for` (opc/mod.rs:837) L1471; `Finding::new` (validate.rs:133) L1475,1485,1520,1531; `PartFs::part_string` (opc/mod.rs:636) L1484; `Dom::attributes` (xmllinq/mod.rs:899) (by name) L1501
- **Calls (std/external/unresolved):** `.as_ref`, `.clear`, `.config_mut`, `.ends_with`, `.flatten`, `.insert`, `.into_owned`, `.is_none`, `.map_or_else`, `.name`, `.push`, `.read_event_into`, `.resolve_attribute`, `.resolver`, `.starts_with`, `.to_string`, `.trim_text`, `.with_checks`, `HashSet::new`, `String::new`, `Vec::new`, `quick_xml::NsReader::from_str`, `quick_xml::escape::unescape`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `check_relationship_integrity`  (L1548-1641)
`fn check_relationship_integrity(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Every `r:id` / `r:embed` / `r:link` in a part resolves in that part's `.rels`; no duplicate rIds; no dangling internal targets.
- **Calls (crate):** `PartFs::repaired_duplicate_ids` (opc/mod.rs:621) L1551; `Finding::new` (validate.rs:133) L1557,1578,1602,1623; `PartFs::parts` (opc/mod.rs:733) L1564; `PartFs::part_string` (opc/mod.rs:636) L1568; `PartFs::read_rels_for` (opc/mod.rs:991) L1571; `PartFs::resolve_rel_target` (opc/mod.rs:829) L1611; `PartFs::part_bytes` (opc/mod.rs:628) L1612,1613
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.clone`, `.contains`, `.ends_with`, `.find`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.len`, `.push`, `.starts_with`, `.trim_start_matches`, `HashMap::new`, `HashSet::new`, `Some`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165); `validation_owner_boundary_tests::relationship_target_diagnostics_distinguish_owned_payloads_from_unresolved_and_external_targets` (validate.rs:3904)

#### `check_picture_bullets`  (L1645-1667)
`fn check_picture_bullets(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Every `w:lvlPicBulletId` names a `w:numPicBullet` of the same numbering part: Word refuses the whole package otherwise ("document loaded empty").
- **Calls (crate):** `parse_part` (validate.rs:1442) L1647; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1650; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1653,1656; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1655; `Finding::new` (validate.rs:133) L1659; `element_path` (validate.rs:1418) L1662
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.push`, `Some`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `check_revision_and_drawing_ids`  (L1669-1678)
`fn check_revision_and_drawing_ids(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1670; `parse_part` (validate.rs:1442) L1671; `collect_ids` (validate.rs:1718) L1676
- **Calls (std/external/unresolved):** `HashSet::new`
- **Called by:** `ring1` (validate.rs:1165)

#### `has_unique_revision_id`  (L1684-1702, pub(crate))
`pub(crate) fn has_unique_revision_id(name: &XName) -> bool`

- **Does:** Revision elements whose `w:id` must be unique among elements of the same record type within a part (the SDK's `Sem_UniqueAttributeValue`).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L1685; `XName::local_name` (xmllinq/mod.rs:208) L1686; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1688
- **Calls (std/external/unresolved):** `.ends_with`
- **Macros:** `matches!`
- **Called by:** `RevisionIds::new` (revision_processor.rs:3777); `collect_ids` (validate.rs:1718); `renumber_duplicate_revision_ids` (validate.rs:2831)

#### `revision_id_key`  (L1708-1716)
`fn revision_id_key(dom: &Dom, node: NodeId, name: &XName, id: &str) -> String`

- **Does:** The SDK distinguishes run revisions from row/paragraph-mark revisions, even though both use `ins`, `del`, `moveFrom` or `moveTo`.
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) (by name) L1709,1712,1715; `Dom::parent` (xmllinq/mod.rs:597) L1710; `Dom::name` (xmllinq/mod.rs:526) L1711; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1712
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `format!`, `matches!`
- **Called by:** `collect_ids` (validate.rs:1718); `renumber_duplicate_revision_ids` (validate.rs:2831)

#### `collect_ids`  (L1718-1767)
`fn collect_ids( dom: &Dom, root: NodeId, rev_ids: &mut HashSet<String>, docpr_ids: &mut HashSet<String>, out: &mut Vec<Finding>, part: &str, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1726; `Dom::name` (xmllinq/mod.rs:526) L1727; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1730,1752; `has_unique_revision_id` (validate.rs:1684) L1733; `Dom::attribute` (xmllinq/mod.rs:874) L1734,1749; `revision_id_key` (validate.rs:1708) L1735; `Finding::new` (validate.rs:133) L1737,1758; `element_path` (validate.rs:1418) L1740,1761; `XName::get` (xmllinq/mod.rs:180) L1745; `Dom::attributes` (xmllinq/mod.rs:899) L1750
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.insert`, `.into_iter`, `.map`, `.or_else`, `.push`, `W::name`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520) (?); `check_revision_and_drawing_ids` (validate.rs:1669)

#### `para_id_values`  (L1772-1786)
`fn para_id_values(xml: &str) -> Vec<u32>`

- **Does:** `(attribute, value)` pairs of `w14:paraId` / `w14:textId` values at or above Word's `0x80000000` bound, in `xml`.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push`, `Vec::new`, `u32::from_str_radix`
- **Called by:** `renumber_out_of_range_para_ids` (validate.rs:2886); `validation_boundary_tests::paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes` (validate.rs:3730)

#### `out_of_range_para_ids`  (L1788-1805)
`fn out_of_range_para_ids(xml: &str) -> Vec<(&'static str, String)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push`, `.to_string`, `Vec::new`, `u32::from_str_radix`
- **Called by:** `check_para_text_id_bounds` (validate.rs:1807); `renumber_out_of_range_para_ids` (validate.rs:2886); `validation_boundary_tests::paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes` (validate.rs:3730)

#### `check_para_text_id_bounds`  (L1807-1823)
`fn check_para_text_id_bounds(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1808; `PartFs::part_string` (opc/mod.rs:636) L1809; `out_of_range_para_ids` (validate.rs:1788) L1812; `Finding::new` (validate.rs:133) L1813
- **Calls (std/external/unresolved):** `.push`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `check_del_text_under_del`  (L1831-1866)
`fn check_del_text_under_del(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** `w:del` must carry `w:delText` (never `w:t`).
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1832; `parse_part` (validate.rs:1442) L1833; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1840,1852,1853; `under_deletion` (validate.rs:1870) L1841; `Finding::new` (validate.rs:133) L1842,1857; `element_path` (validate.rs:1418) L1845,1860; `ancestor_has` (validate.rs:1987) L1854
- **Calls (std/external/unresolved):** `.push`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `under_deletion`  (L1870-1875)
`fn under_deletion(dom: &Dom, node: NodeId) -> bool`

- **Does:** Any enclosing deletion counts, even through an insertion, but only within the run's own story (#1134, Word probe 2026-10-17).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1871; `Dom::name` (xmllinq/mod.rs:526) L1873,1874; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1873
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.take_while`, `W::del`
- **Called by:** `check_del_text_under_del` (validate.rs:1831); `fix_deleted_text` (validate.rs:2733)

#### `covered_by_deletion`  (L1879-1889)
`fn covered_by_deletion(dom: &Dom, node: NodeId) -> bool`

- **Does:** True when a `w:del` or `w:moveFrom` covers `node` in its own story (a text box's `w:txbxContent` is a separate story).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1880,1885; `Dom::name` (xmllinq/mod.rs:526) L1882; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1882
- **Calls (std/external/unresolved):** `.as_ref`, `.map`
- **Called by:** `check_deleted_text_has_deletion` (validate.rs:1897); `fix_deleted_text` (validate.rs:2733); `validation_boundary_tests::deletion_scope_stops_at_textboxes_and_covers_nested_insertions` (validate.rs:3747)

#### `check_deleted_text_has_deletion`  (L1897-1917)
`fn check_deleted_text_has_deletion(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** `w:delText` / `w:delInstrText` must sit under a `w:del` (or `w:moveFrom`) in its OWN story.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1898; `parse_part` (validate.rs:1442) L1899; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1903; `covered_by_deletion` (validate.rs:1879) L1904; `Finding::new` (validate.rs:133) L1905; `element_path` (validate.rs:1418) L1908
- **Calls (std/external/unresolved):** `.push`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `bookmarks_in_single_value_controls`  (L1932-1953)
`fn bookmarks_in_single_value_controls( dom: &Dom, root: NodeId, ) -> Vec<(NodeId, &'static str, String)>`

- **Does:** `(bookmark mark, kind, control)` for every bookmark start or end inside a single-value content control.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1938; `Dom::ancestors` (xmllinq/mod.rs:770) L1939; `Dom::element` (xmllinq/mod.rs:655) L1940; `Dom::elements` (xmllinq/mod.rs:640) L1943; `Dom::name` (xmllinq/mod.rs:526) L1944; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1944
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.filter_map`, `.find`, `.flat_map`, `.into_iter`, `.map`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::name`
- **Called by:** `check_bookmarks_outside_single_value_controls` (validate.rs:1961); `drop_bookmarks_in_single_value_controls` (validate.rs:2767); `validation_boundary_tests::bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text` (validate.rs:3773)

#### `check_bookmarks_outside_single_value_controls`  (L1961-1985)
`fn check_bookmarks_outside_single_value_controls(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** No bookmark start or end in a plain-text, dropdown, combo box, date, picture or checkbox content control.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L1962; `PartFs::part_string` (opc/mod.rs:636) L1963; `Dom::new` (xmllinq/mod.rs:387) L1969; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1970; `Dom::root` (xmllinq/mod.rs:822) L1971; `bookmarks_in_single_value_controls` (validate.rs:1932) L1974; `Finding::new` (validate.rs:133) L1975; `element_path` (validate.rs:1418) L1978
- **Calls (std/external/unresolved):** `.contains`, `.push`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165)

#### `ancestor_has`  (L1987-2001)
`fn ancestor_has(dom: &Dom, node: NodeId, stop: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1988,1998; `Dom::name` (xmllinq/mod.rs:526) L1993; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1994
- **Called by:** `check_del_text_under_del` (validate.rs:1831); `fix_deleted_text` (validate.rs:2733)

#### `comments_part`  (L2034-2053)
`fn comments_part(pkg: &PartFs, main: &str) -> Option<String>`

- **Does:** The comments part as Word finds it: the first relationship of the comments type whose target part exists (`word/comments.xml` is only the conventional name — Word follows the relationship, so a differently named part is the comments part), else the conventional name when it exists unlinked.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L2035; `PartFs::resolve_rel_target` (opc/mod.rs:829) L2043; `PartFs::part_bytes` (opc/mod.rs:628) L2047,2051
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.find`, `.flat_map`, `.into_iter`, `.is_some`, `.map`, `.to_string`, `.trim_start_matches`, `Some`
- **Called by:** `check_comment_graph` (validate.rs:2067)

#### `main_part`  (L2055-2060)
`fn main_part(pkg: &PartFs) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L2056; `PartFs::part_bytes` (opc/mod.rs:628) L2057
- **Calls (std/external/unresolved):** `.map`, `.or_else`, `.to_string`
- **Called by:** `check_data_bindings_have_stores` (validate.rs:1189); `check_comment_graph` (validate.rs:2067); `repair` (validate.rs:2623)

#### `comment_finding`  (L2063-2065)
`fn comment_finding(out: &mut Vec<Finding>, code: &str, part: &str, message: String)`

- **Does:** A finding of the comment family: `code` on `part`, no path.
- **Calls (crate):** `Finding::new` (validate.rs:133) L2064
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `check_comment_graph` (validate.rs:2067); `check_comment_family_packaging` (validate.rs:2246); `check_hex_id` (validate.rs:2303); `check_comments_extended` (validate.rs:2342); `check_comments_ids` (validate.rs:2389); `check_comments_extensible` (validate.rs:2442); `check_exact_key_set` (validate.rs:2500); `check_parent_cycles` (validate.rs:2525)

#### `check_comment_graph`  (L2067-2225)
`fn check_comment_graph(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `main_part` (validate.rs:2055) L2068; `parse_part` (validate.rs:1442) L2071,2105; `comment_id_counts` (validate.rs:2230) L2074,2075,2076,2109; `comments_part` (validate.rs:2034) L2078; `comment_finding` (validate.rs:2063) L2085,2095,2125,2142,2159,2187...; `PartFs::part_bytes` (opc/mod.rs:628) L2094,2172; `check_comment_family_packaging` (validate.rs:2246) L2168; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2175; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L2176,2181; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L2180; `check_hex_id` (validate.rs:2303) L2185; `check_comments_extended` (validate.rs:2342) L2209; `check_comments_ids` (validate.rs:2389) L2210; `check_comments_extensible` (validate.rs:2442) L2211; `check_parent_cycles` (validate.rs:2525) L2223
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.as_str`, `.clone`, `.contains_key`, `.contains`, `.filter`, `.insert`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.last`, `.push`, `.to_ascii_uppercase`, `.unwrap_or`, `.values`, `HashSet::new`, `Some`, `Vec::new`, `W14::name`, `W::id`, `W::name`, `W::p`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:1165); `authored_comment_family_validation_contract_tests::auxiliary_comment_parts_without_their_source_definition_are_reported_individually` (validate.rs:4330)

#### `comment_id_counts`  (L2230-2244)
`fn comment_id_counts(dom: &Dom, root: NodeId, local: &str) -> HashMap<String, usize>`

- **Does:** How often each id occurs on `local` elements.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2232; `Dom::ancestors` (xmllinq/mod.rs:770) L2233; `Dom::attribute` (xmllinq/mod.rs:874) L2239
- **Calls (std/external/unresolved):** `.entry`, `.is_empty`, `.or_default`, `.to_string`, `HashMap::new`, `MC::name`, `Some`, `W::id`, `W::name`
- **Called by:** `check_comment_graph` (validate.rs:2067); `repair` (validate.rs:2623)

#### `check_comment_family_packaging`  (L2246-2301)
`fn check_comment_family_packaging(pkg: &PartFs, main: &str, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L2247; `PartFs::part_bytes` (opc/mod.rs:628) L2249; `PartFs::content_type_for` (opc/mod.rs:837) L2256; `comment_finding` (validate.rs:2063) L2257,2265,2281,2293; `PartFs::resolve_rel_target` (opc/mod.rs:829) L2276
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.to_string`, `.trim_start_matches`, `Some`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:2067); `authored_comment_family_validation_contract_tests::each_comment_family_relationship_requires_its_exact_owned_part_and_content_type` (validate.rs:4354)

#### `check_hex_id`  (L2303-2335)
`fn check_hex_id( label: &str, value: &str, word_para_bound: bool, part: &str, out: &mut Vec<Finding>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_finding` (validate.rs:2063) L2319,2328
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.contains`, `.is_ascii_hexdigit`, `.len`, `.ok`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:2067); `check_comments_extended` (validate.rs:2342); `check_comments_ids` (validate.rs:2389); `check_comments_extensible` (validate.rs:2442); `validation_boundary_tests::metadata_ids_check_width_hex_digits_and_the_word_boundary_independently` (validate.rs:3693)

#### `check_comments_extended`  (L2342-2387)
`fn check_comments_extended( pkg: &PartFs, last_para_ids: &HashSet<String>, out: &mut Vec<Finding>, ) -> Option<ExtendedGraph>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:1442) L2348; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2351; `attribute_by_local` (validate.rs:2490) L2352,2371; `comment_finding` (validate.rs:2063) L2353,2364,2375; `check_hex_id` (validate.rs:2303) L2362,2373; `check_exact_key_set` (validate.rs:2500) L2385
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.to_ascii_uppercase`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:2067); `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (validate.rs:3794)

#### `check_comments_ids`  (L2389-2440)
`fn check_comments_ids( pkg: &PartFs, last_para_ids: &HashSet<String>, out: &mut Vec<Finding>, ) -> Option<HashSet<String>>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:1442) L2395; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2398; `attribute_by_local` (validate.rs:2490) L2399,2418; `comment_finding` (validate.rs:2063) L2400,2411,2419,2430; `check_hex_id` (validate.rs:2303) L2409,2428; `check_exact_key_set` (validate.rs:2500) L2438
- **Calls (std/external/unresolved):** `.insert`, `.to_ascii_uppercase`, `.to_string`, `HashSet::new`, `Some`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:2067); `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (validate.rs:3794)

#### `check_comments_extensible`  (L2442-2488)
`fn check_comments_extensible( pkg: &PartFs, expected_durable_ids: Option<&HashSet<String>>, out: &mut Vec<Finding>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:1442) L2448; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2452; `attribute_by_local` (validate.rs:2490) L2453; `comment_finding` (validate.rs:2063) L2454,2465,2481; `check_hex_id` (validate.rs:2303) L2463; `check_exact_key_set` (validate.rs:2500) L2474
- **Calls (std/external/unresolved):** `.insert`, `.to_ascii_uppercase`, `.to_string`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:2067); `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (validate.rs:3794)

#### `attribute_by_local`  (L2490-2498)
`fn attribute_by_local<'a>(dom: &'a Dom, element: NodeId, local: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:909) L2491; `Dom::attr_at` (xmllinq/mod.rs:920) L2492; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2493
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `check_comments_extended` (validate.rs:2342); `check_comments_ids` (validate.rs:2389); `check_comments_extensible` (validate.rs:2442)

#### `check_exact_key_set`  (L2500-2523)
`fn check_exact_key_set( label: &str, actual: &HashSet<String>, expected: &HashSet<String>, part: &str, out: &mut Vec<Finding>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_finding` (validate.rs:2063) L2508,2516
- **Calls (std/external/unresolved):** `.difference`
- **Macros:** `format!`
- **Called by:** `check_comments_extended` (validate.rs:2342); `check_comments_ids` (validate.rs:2389); `check_comments_extensible` (validate.rs:2442)

#### `check_parent_cycles`  (L2525-2542)
`fn check_parent_cycles(parents: &HashMap<String, String>, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_finding` (validate.rs:2063) L2531
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.insert`, `.keys`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:2067); `validation_boundary_tests::graph_cycles_are_reported_without_confusing_acyclic_chains` (validate.rs:3834)

#### `is_namespace_qname_list`  (L2546-2560)
`fn is_namespace_qname_list(element: &XName, name: &XName) -> bool`

- **Does:** An unqualified `Requires` is a prefix list only on `mc:Choice`; on any other element (a custom XML part) it is application data.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2547,2549,2551; `XName::local_name` (xmllinq/mod.rs:208) L2548,2550; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2553
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `matches!`
- **Called by:** `check_namespace_qname_context` (validate.rs:2562); `validation_boundary_tests::namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes` (validate.rs:3854)

#### `check_namespace_qname_context`  (L2562-2593)
`fn check_namespace_qname_context(pkg: &PartFs, part: &str, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:1442) L2563; `Dom::descendants_and_self` (xmllinq/mod.rs:746) (by name) L2566; `Dom::attributes_iter` (xmllinq/mod.rs:886) (by name) L2570; `is_namespace_qname_list` (validate.rs:2546) L2571; `namespace_in_scope` (validate.rs:2595) L2576; `Finding::new` (validate.rs:133) L2577; `element_path` (validate.rs:1418) L2580; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2583; `well_known_namespace` (xmllinq/serialize.rs:190) L2587
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.map_or`, `.name`, `.push`, `.split_once`, `.split_whitespace`
- **Macros:** `format!`
- **Called by:** `check_namespace_qname_contexts` (validate.rs:1462); `validation_boundary_tests::namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes` (validate.rs:3854)

#### `namespace_in_scope`  (L2595-2607)
`fn namespace_in_scope<'a>(dom: &'a Dom, element: NodeId, prefix: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:909) L2598; `Dom::attr_at` (xmllinq/mod.rs:920) L2599; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L2600; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2600; `Dom::parent` (xmllinq/mod.rs:597) L2604
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `check_namespace_qname_context` (validate.rs:2562)

#### `repair`  (L2623-2714, pub)
`pub fn repair(docx: &[u8]) -> Result<Repaired, ValidateError>`

- **Does:** The package with every repairable finding fixed, in a fixed order: unbound MC prefixes bound on the part root, `w:t`/`w:delText` swapped, bookmarks dropped from single-value controls, dangling relationship attributes dropped, duplicate drawing and revision ids renumbered, paragraph ids brought into Word's range, cells...
- **Calls (crate):** `validate` (validate.rs:1140) L2624,2702; `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121) L2646; `PartFs::open` (opc/mod.rs:585) L2650; `edit_part` (validate.rs:2717) L2652,2662,2665,2672,2677,2685...; `bind_compatibility_prefixes` (comparer/finalize.rs:1026) L2653; `PartFs::read_rels_for` (opc/mod.rs:991) L2668; `drop_dangling_relationship_attributes` (validate.rs:2802) L2673; `renumber_duplicate_revision_ids` (validate.rs:2831) L2678; `renumber_out_of_range_para_ids` (validate.rs:2886) L2682; `main_part` (validate.rs:2055) L2688; `parse_part` (validate.rs:1442) L2690; `comment_id_counts` (validate.rs:2230) L2692; `drop_orphan_comment_anchors` (validate.rs:2949) L2698; `PartFs::to_zip` (opc/mod.rs:749) L2701; `detach_breaks::{fn same}` (markdown/unified.rs:876) (by name) L2707
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.dedup`, `.filter`, `.into_iter`, `.into_keys`, `.iter`, `.map`, `.sort`, `.to_vec`, `.unwrap_or_default`, `Ok`, `Vec::new`, `has`, `parts_with`
- **Called by:** `run_validate` (bin/jubarte.rs:3812); `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (validate.rs:4028); `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (validate.rs:4067); `retained_namespace_finding_source_contract_tests::repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content` (validate.rs:4448); `duplicate_drawing_owned_repair_contract_tests::duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner` (validate.rs:4551)

#### `edit_part`  (L2717-2728)
`fn edit_part(pkg: &mut PartFs, part: &str, f: impl FnOnce(&mut Dom, NodeId))`

- **Does:** Parse `part`, let `f` change it, write it back.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L2718; `Dom::new` (xmllinq/mod.rs:387) L2721; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L2722; `Dom::root` (xmllinq/mod.rs:822) L2723; `PartFs::set_part` (opc/mod.rs:661) L2727; `Dom::serialize_document` (xmllinq/mod.rs:1338) L2727
- **Calls (std/external/unresolved):** `.into_bytes`, `f`
- **Called by:** `repair` (validate.rs:2623); `validation_boundary_tests::absent_or_unreadable_parts_are_not_modified_by_edit_part` (validate.rs:3883)

#### `fix_deleted_text`  (L2733-2763)
`fn fix_deleted_text(dom: &mut Dom, root: NodeId)`

- **Does:** `w:t` and `w:instrText` under a deletion become `w:delText` and `w:delInstrText`; `w:delText` under a move source, and deleted text no deletion covers, become live text again.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2736,2742,2743,2754; `under_deletion` (validate.rs:1870) L2737; `ancestor_has` (validate.rs:1987) L2744; `Dom::set_name` (xmllinq/mod.rs:539) L2750,2761; `covered_by_deletion` (validate.rs:1879) L2755
- **Calls (std/external/unresolved):** `.push`, `Some`, `Vec::new`, `W::name`, `W::t`
- **Called by:** `validation_boundary_tests::deletion_scope_stops_at_textboxes_and_covers_nested_insertions` (validate.rs:3747)

#### `drop_bookmarks_in_single_value_controls`  (L2767-2783)
`fn drop_bookmarks_in_single_value_controls(dom: &mut Dom, root: NodeId)`

- **Does:** Every bookmark whose start or end sits in a single-value content control is dropped whole (both marks, wherever the other one is).
- **Calls (crate):** `bookmarks_in_single_value_controls` (validate.rs:1932) L2768; `Dom::attribute` (xmllinq/mod.rs:874) L2770,2776; `Dom::descendants` (xmllinq/mod.rs:668) L2774; `Dom::remove` (xmllinq/mod.rs:1118) L2781
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W::id`, `W::name`
- **Called by:** `validation_boundary_tests::bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text` (validate.rs:3773)

#### `drop_dangling_relationship_attributes`  (L2802-2826)
`fn drop_dangling_relationship_attributes(dom: &mut Dom, root: NodeId, ids: &HashSet<String>)`

- **Does:** Relationship attributes (`r:id`, `r:embed`, `r:link`, …) naming no relationship of the part are dropped, which is what stops Word's "unreadable content" repair.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L2805; `Dom::attributes` (xmllinq/mod.rs:899) L2806; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2807; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2808,2811; `Dom::name` (xmllinq/mod.rs:526) L2809; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2821; `Dom::remove` (xmllinq/mod.rs:1118) L2824
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.is_some_and`, `.push`, `Vec::new`
- **Called by:** `repair` (validate.rs:2623); `paragraph_id_collision_owner_tests::dangling_relationship_repair_keeps_empty_known_and_nonrelationship_owners` (validate.rs:4273)

#### `renumber_duplicate_revision_ids`  (L2831-2881, pub(crate))
`pub(crate) fn renumber_duplicate_revision_ids( dom: &mut Dom, root: NodeId, ) -> Vec<(String, String)>`

- **Does:** A second use of a revision id on the same record type is renumbered from the part's highest `w:id` upwards; a move range start takes its end along.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2836; `Dom::attribute` (xmllinq/mod.rs:874) L2839,2853,2871; `Dom::name` (xmllinq/mod.rs:526) L2848,2870; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2849,2870; `has_unique_revision_id` (validate.rs:1684) L2850; `revision_id_key` (validate.rs:1708) L2856; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2878
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.enumerate`, `.filter_map`, `.find`, `.insert`, `.is_some_and`, `.iter`, `.map`, `.max`, `.ok`, `.parse`, `.push`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Some`, `Vec::new`, `W::name`
- **Called by:** `unique_revision_ids` (markdown/package.rs:378); `repair` (validate.rs:2623); `literal_revision_repair_contract_tests::duplicate_move_ids_keep_exact_range_pairings_and_every_unrelated_source_node` (validate.rs:4167); `literal_revision_repair_contract_tests::repair_declines_missing_ids_and_unmatched_ranges_without_inventing_content` (validate.rs:4188)

#### `renumber_out_of_range_para_ids`  (L2886-2931)
`fn renumber_out_of_range_para_ids(pkg: &mut PartFs)`

- **Does:** Every `w14:paraId` / `w14:textId` at or above `0x80000000` is masked into Word's range, and every reference to it (`paraIdParent`, the comment parts) follows.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L2887; `PartFs::part_string` (opc/mod.rs:636) L2890,2897,2918; `para_id_values` (validate.rs:1772) L2893; `out_of_range_para_ids` (validate.rs:1788) L2900; `PartFs::set_part` (opc/mod.rs:661) L2928
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.contains`, `.extend`, `.insert`, `.into_bytes`, `.is_empty`, `.replace`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `repair` (validate.rs:2623); `paragraph_id_collision_owner_tests::paragraph_id_collision_repair_preserves_all_owners_and_shared_references` (validate.rs:4224)

#### `end_cells_with_a_paragraph`  (L2935-2945)
`fn end_cells_with_a_paragraph(dom: &mut Dom, root: NodeId)`

- **Does:** A table cell whose last child (its properties aside) is not a paragraph gets an empty one.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2936; `cell_ends_in_paragraph` (debug.rs:392) L2939; `Dom::new_element` (xmllinq/mod.rs:478) L2942; `Dom::add` (xmllinq/mod.rs:1074) L2943
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::name`, `W::p`
- **Called by:** none found in src

#### `drop_orphan_comment_anchors`  (L2949-2981)
`fn drop_orphan_comment_anchors(dom: &mut Dom, root: NodeId, defined: &HashSet<String>)`

- **Does:** Comment range marks and references naming no comment definition are dropped; a run left holding only its properties goes with its reference.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L2951; `Dom::descendants` (xmllinq/mod.rs:668) L2956,2962; `Dom::remove` (xmllinq/mod.rs:1118) L2960,2971,2978; `Dom::parent` (xmllinq/mod.rs:597) L2968; `Dom::name_is` (xmllinq/mod.rs:535) L2970,2976; `Dom::elements` (xmllinq/mod.rs:640) L2973
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.into_iter`, `.is_some_and`, `.iter`, `Some`, `W::id`, `W::name`, `orphan`
- **Called by:** `repair` (validate.rs:2623); `validation_owner_boundary_tests::orphan_comment_cleanup_preserves_live_references_and_neighbor_run_properties` (validate.rs:3953)

#### `all_paragraphs`  (L2986-2992)
`fn all_paragraphs(docx: &[u8]) -> Result<Vec<crate::inspect::Paragraph>, ValidateError>`

- **Does:** The body's paragraphs followed by every story's (`header1`, ...).
- **Calls (crate):** `paragraphs` (inspect.rs:312) L2987; `stories` (inspect.rs:353) L2988
- **Calls (std/external/unresolved):** `.extend`, `Ok`
- **Called by:** `audit_tracked` (validate.rs:3027)

#### `story_part`  (L2997-3005)
`fn story_part(id: &str) -> String`

- **Does:** The part a paragraph or change id lives in: the body's (`p3`, `r12`) is the main document, any other story's (`header1.p1`, `footnotes.r2`) `word/{story}.xml`.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L2998; `change_id` (ids.rs:187) L3000
- **Calls (std/external/unresolved):** `.as_deref`, `.map`, `.or_else`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `<Finding as From>::from` (validate.rs:1112); `audit_tracked` (validate.rs:3027)

#### `audit_tracked`  (L3027-3071, pub)
`pub fn audit_tracked( original: &[u8], edited: &[u8], author: &str, ) -> Result<Vec<Finding>, ValidateError>`

- **Does:** `validate.py --original --author` without an XSD: every visible-text difference between `original` and `edited` must sit in a revision by `author`.
- **Calls (crate):** `check_dropped_permissions` (validate.rs:3622) L3033; `PartFs::open` (opc/mod.rs:585) L3033; `reject_changes` (changes.rs:699) L3038; `check_foreign_revisions` (validate.rs:3541) L3039; `all_paragraphs` (validate.rs:2986) L3042,3043; `Finding::new` (validate.rs:133) L3045,3059; `as_edits` (validate.rs:3666) L3055,3070; `story_part` (validate.rs:2997) L3061
- **Calls (std/external/unresolved):** `.clone`, `.iter`, `.len`, `.push`, `.to_string`, `.zip`, `ChangeFilter::default`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `validate_with` (validate.rs:316)

#### `ordered_pairs`  (L3077-3154)
`fn ordered_pairs<T: Eq + std::hash::Hash>(a: &[T], b: &[T]) -> Vec<(usize, usize)>`

- **Does:** Order-stable alignment.
- **Calls (crate):** `bounded_myers_pairs` (validate.rs:3158) L3098
- **Calls (std/external/unresolved):** `.checked_sub`, `.collect`, `.copied`, `.count`, `.entry`, `.enumerate`, `.extend`, `.flatten`, `.fold`, `.get`, `.into_iter`, `.iter`, `.last`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.or_default`, `.partition_point`, `.push`, `.rev`, `.saturating_add`, `.saturating_mul`, `.skip`, `.take_while`, `.take`, `.zip`, `HashMap::new`, `Vec::new`
- **Called by:** `aligned_pairs` (validate.rs:3213); `missing_foreign_markers` (validate.rs:3454); `report_foreign_carriers` (validate.rs:3510); `check_foreign_revisions` (validate.rs:3541); `foreign_alignment_budget_tests::bounded_alignment` (validate.rs:4631); `foreign_alignment_budget_tests::foreign_alignment_dense_fallback_is_order_stable` (validate.rs:4682)

#### `bounded_myers_pairs`  (L3158-3209)
`fn bounded_myers_pairs<T: Eq>(a: &[T], b: &[T]) -> Option<Vec<(usize, usize)>>`

- **Does:** Myers' shortest edit path, bounded independently of input length.
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.push`, `.rev`, `.reverse`, `.step_by`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `ordered_pairs` (validate.rs:3077)

#### `aligned_pairs`  (L3213-3226)
`fn aligned_pairs<T: Eq + std::hash::Hash>(a: &[T], b: &[T]) -> Vec<(usize, usize)>`

- **Does:** Pair replacement gaps between the LCS anchors.
- **Calls (crate):** `ordered_pairs` (validate.rs:3077) L3214
- **Calls (std/external/unresolved):** `.chain`, `.extend`, `.into_iter`, `.len`, `.push`, `.zip`, `Vec::new`, `std::iter::once`
- **Called by:** `foreign_paragraphs` (validate.rs:3252); `check_foreign_revisions` (validate.rs:3541)

#### `foreign_paragraphs`  (L3252-3304)
`fn foreign_paragraphs( before: &[crate::changes::AuditStory], after: &[crate::changes::AuditStory], ) -> HashMap<(String, String), usize>`

- **Does:** Story-local paragraph identities.
- **Calls (crate):** `aligned_pairs` (validate.rs:3213) L3297
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.enumerate`, `.find`, `.get`, `.insert`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.or_default`, `.push`, `.rev`, `.then_some`, `HashMap::new`, `Some`
- **Macros:** `vec!`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `foreign_revisions`  (L3308-3358)
`fn foreign_revisions<'a>( stories: &'a [crate::changes::AuditStory], paragraphs: &HashMap<(String, String), usize>, edited: bool, ) -> Vec<ForeignGroup<'a>>`

- **Does:** Keep each carrier for attribution.
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.get`, `.iter`, `.map`, `.push`, `.unwrap_or`, `Vec::new`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `relax_foreign_boundaries`  (L3362-3416)
`fn relax_foreign_boundaries( groups: &mut [ForeignGroup<'_>], present: &[bool], mut missing: HashMap<String, Vec<(usize, usize)>>, )`

- **Does:** Remove only the document spans of revisions with no sequence counterpart.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.get`, `.insert`, `.into_iter`, `.iter`, `.last_mut`, `.map`, `.max`, `.min`, `.or_default`, `.partition_point`, `.push`, `.sort_unstable`, `.to_string`, `.zip`, `HashMap::new`, `Vec::new`, `position`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `foreign_window`  (L3420-3436)
`fn foreign_window(groups: &[ForeignGroup<'_>], indices: &[usize]) -> Vec<(Vec<usize>, String)>`

- **Does:** A comparison window may lose boundaries only where a missing separator explains them.
- **Calls (std/external/unresolved):** `.clone`, `.is_some_and`, `.last_mut`, `.push_str`, `.push`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `foreign_finding`  (L3438-3447)
`fn foreign_finding(group: &ForeignGroup<'_>, state: &str) -> Finding`

- **Does:** _no doc comment_
- **Calls (crate):** `Finding::new` (validate.rs:133) L3446
- **Calls (std/external/unresolved):** `.as_deref`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `report_foreign_carriers` (validate.rs:3510)

#### `missing_foreign_markers`  (L3454-3506)
`fn missing_foreign_markers( before: &[crate::changes::AuditStory], after: &[crate::changes::AuditStory], paragraphs: &HashMap<(String, String), usize>, ) -> (ForeignSpans, ForeignSpans)`

- **Does:** Match omitted markers within aligned paragraphs, ignoring revision ids.
- **Calls (crate):** `ordered_pairs` (validate.rs:3077) L3486; `missing` (ambiguous: `missing` (edit/anchor.rs:217); `edit_core::{fn missing}` (markdown/package.rs:1566)) L3505
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.entry`, `.enumerate`, `.flat_map`, `.get`, `.iter`, `.map`, `.or_default`, `.push`, `HashMap::new`, `keys`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `report_foreign_carriers`  (L3510-3539)
`fn report_foreign_carriers( before: &[ForeignGroup<'_>], after: &[ForeignGroup<'_>], old: &[usize], new: &[usize], findings: &mut ForeignFindings, )`

- **Does:** Exact carriers within an unequal combined window precede replacements.
- **Calls (crate):** `ordered_pairs` (validate.rs:3077) L3520; `foreign_finding` (validate.rs:3438) L3531,3534
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.enumerate`, `.into_iter`, `.iter`, `.len`, `.map`, `.min`, `.push`, `std::iter::once`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `check_foreign_revisions`  (L3541-3618)
`fn check_foreign_revisions( original: &[u8], reverted: &[u8], author: &str, out: &mut Vec<Finding>, ) -> Result<(), ValidateError>`

- **Does:** _no doc comment_
- **Calls (crate):** `audit_stories` (changes.rs:168) L3547,3548; `foreign_paragraphs` (validate.rs:3252) L3549; `foreign_revisions` (validate.rs:3308) L3550,3551; `aligned_pairs` (validate.rs:3213) L3564; `missing_foreign_markers` (validate.rs:3454) L3569; `relax_foreign_boundaries` (validate.rs:3362) L3570,3571; `ordered_pairs` (validate.rs:3077) L3580,3589; `foreign_window` (validate.rs:3420) L3584,3585; `report_foreign_carriers` (validate.rs:3510) L3601
- **Calls (std/external/unresolved):** `.as_deref`, `.chain`, `.clone`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.extend`, `.flat_map`, `.into_iter`, `.iter`, `.len`, `.map`, `.or_default`, `.push`, `.sort_by_key`, `.values`, `HashMap::new`, `Ok`, `Some`, `Vec::new`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `audit_tracked` (validate.rs:3027)

#### `check_dropped_permissions`  (L3622-3663)
`fn check_dropped_permissions(original: &PartFs, edited: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Permission exceptions are edit fidelity, not Word validity.
- **Calls (crate):** `xml_parts` (validate.rs:1450) L3623; `PartFs::part_string` (opc/mod.rs:636) L3624; `parse_part` (validate.rs:1442) L3630,3633; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L3638,3644; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3639,3645; `Finding::new` (validate.rs:133) L3654; `element_path` (validate.rs:1418) L3657
- **Calls (std/external/unresolved):** `.contains`, `.entry`, `.get_mut`, `.is_none_or`, `.or_default`, `.push`, `.to_string`, `HashMap::new`, `Some`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `audit_tracked` (validate.rs:3027)

#### `as_edits`  (L3666-3672)
`fn as_edits(mut findings: Vec<Finding>) -> Vec<Finding>`

- **Does:** [`audit_tracked`]'s findings belong to the `edits` set and fail a run.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `audit_tracked` (validate.rs:3027)

## Test functions (44)

- `validation_boundary_tests::pkg` (L3679): _no doc comment_
- `validation_boundary_tests::xml` (L3683): _no doc comment_
- `validation_boundary_tests::metadata_ids_check_width_hex_digits_and_the_word_boundary_independently` (L3693): _no doc comment_
- `validation_boundary_tests::paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes` (L3730): _no doc comment_
- `validation_boundary_tests::deletion_scope_stops_at_textboxes_and_covers_nested_insertions` (L3747): _no doc comment_
- `validation_boundary_tests::bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text` (L3773): _no doc comment_
- `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (L3794): _no doc comment_
- `validation_boundary_tests::graph_cycles_are_reported_without_confusing_acyclic_chains` (L3834): _no doc comment_
- `validation_boundary_tests::namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes` (L3854): _no doc comment_
- `validation_boundary_tests::absent_or_unreadable_parts_are_not_modified_by_edit_part` (L3883): _no doc comment_
- `validation_owner_boundary_tests::package` (L3900): _no doc comment_
- `validation_owner_boundary_tests::relationship_target_diagnostics_distinguish_owned_payloads_from_unresolved_and_external_targets` (L3904): _no doc comment_
- `validation_owner_boundary_tests::orphan_comment_cleanup_preserves_live_references_and_neighbor_run_properties` (L3953): _no doc comment_
- `public_memory_validator_owner_boundary_tests::package` (L4009): _no doc comment_
- `public_memory_validator_owner_boundary_tests::snapshot` (L4015): _no doc comment_
- `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (L4028): _no doc comment_
- `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (L4067): _no doc comment_
- `literal_revision_repair_contract_tests::tree` (L4156): _no doc comment_
- `literal_revision_repair_contract_tests::duplicate_move_ids_keep_exact_range_pairings_and_every_unrelated_source_node` (L4167): _no doc comment_
- `literal_revision_repair_contract_tests::repair_declines_missing_ids_and_unmatched_ranges_without_inventing_content` (L4188): _no doc comment_
- `paragraph_id_collision_owner_tests::snapshot` (L4211): _no doc comment_
- `paragraph_id_collision_owner_tests::paragraph_id_collision_repair_preserves_all_owners_and_shared_references` (L4224): _no doc comment_
- `paragraph_id_collision_owner_tests::dangling_relationship_repair_keeps_empty_known_and_nonrelationship_owners` (L4273): _no doc comment_
- `authored_comment_family_validation_contract_tests::package` (L4311): _no doc comment_
- `authored_comment_family_validation_contract_tests::frozen` (L4317): _no doc comment_
- `authored_comment_family_validation_contract_tests::auxiliary_comment_parts_without_their_source_definition_are_reported_individually` (L4330): _no doc comment_
- `authored_comment_family_validation_contract_tests::each_comment_family_relationship_requires_its_exact_owned_part_and_content_type` (L4354): _no doc comment_
- `retained_namespace_finding_source_contract_tests::repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content` (L4448): _no doc comment_
- `duplicate_drawing_owned_repair_contract_tests::duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner` (L4551): _no doc comment_
- `<Counted as PartialEq>::eq` (L4614): _no doc comment_
- `<Counted as Hash>::hash` (L4626): _no doc comment_
- `foreign_alignment_budget_tests::bounded_alignment` (L4631): _no doc comment_
- `foreign_alignment_budget_tests::foreign_alignment_ten_thousand_empty_paragraphs_is_bounded` (L4653): _no doc comment_
- `foreign_alignment_budget_tests::foreign_alignment_ten_thousand_same_key_deletions_is_bounded` (L4664): _no doc comment_
- `foreign_alignment_budget_tests::foreign_alignment_ten_thousand_alternating_paragraphs_is_bounded` (L4673): _no doc comment_
- `foreign_alignment_budget_tests::foreign_alignment_dense_fallback_is_order_stable` (L4682): _no doc comment_
- `foreign_alignment_budget_tests::foreign_alignment_dense_myers_matches_short_edit_lcs` (L4702): _no doc comment_
- `baseline_tests::finding` (L4722): _no doc comment_
- `baseline_tests::a_chain_resolves_to_the_element_it_names` (L4727): _no doc comment_
- `baseline_tests::identity_drops_element_indices_and_paragraph_ids_from_the_message` (L4744): _no doc comment_
- `baseline_tests::identity_keeps_where_the_finding_sits` (L4784): _no doc comment_
- `baseline_tests::baseline_matches_one_for_one_by_identity` (L4794): _no doc comment_
- `baseline_tests::baselined_text_and_json_name_each_list` (L4818): _no doc comment_
- `baseline_tests::input_notice_counts_errors_and_warnings_but_not_info` (L4852): _no doc comment_
