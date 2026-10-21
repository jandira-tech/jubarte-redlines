# Functions and methods in `src/edit/prepared.rs`

22 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `finding_key`  (L32-39)
`fn finding_key(f: &Finding) -> (String, String, String, String)`

- **Does:** A finding identity the publication policy compares on: rule set, code, part and the entity/range the finding points at.
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `introduced_findings` (edit/prepared.rs:43); `check_candidates` (edit/prepared.rs:397)

#### `introduced_findings`  (L43-52, pub)
`pub fn introduced_findings(baseline: &[Finding], candidate: &[Finding]) -> Vec<Finding>`

- **Does:** The findings in `candidate` that `baseline` does not carry, first occurrence each, in `candidate` order.
- **Calls (crate):** `finding_key` (edit/prepared.rs:32) L48,49
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.contains`, `.filter`, `.insert`, `.iter`, `.map`, `HashSet::new`
- **Called by:** `check_candidates` (edit/prepared.rs:397)

#### `blocking`  (L61-63)
`fn blocking(f: &Finding) -> bool`

- **Does:** Whether `f` blocks publication when it is newly introduced.
- **Calls (std/external/unresolved):** `.as_str`, `.contains`
- **Called by:** `check_candidates` (edit/prepared.rs:397)

#### `CandidateChecks::verification`  (L94-104, pub)
`pub fn verification(&self) -> crate::result::Verification`

- **Does:** Report only completed checks and their computed findings.
- **Calls (crate):** `outcome` (ambiguous: `outcome` (edit/v2_apply.rs:1634); `deeper_boundary_tests::outcome` (edit.rs:8796)) L97,98
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `Default::default`, `Some`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `tests::verification_reports_only_completed_candidate_checks` (edit/prepared.rs:1172) (?)

#### `<PrepareOptions as Default>::default`  (L158-166, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `run_edit` (bin/jubarte.rs:1341); `tests::final_text` (edit/inline.rs:403); `tests::inline_insert_edges_and_paragraph_sides_use_explicit_kinds` (edit/inline.rs:535); `tests::inline_decisions_freeze_original_ids_and_working_rewrite_hash` (edit/inline.rs:612); `tests::verification_reports_only_completed_candidate_checks` (edit/prepared.rs:1172); `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (edit/prepared.rs:1292); `rename_positions` (edit/skip.rs:394); `apply_plan_json` (edit.rs:2271); `apply_plan` (edit.rs:2322); `preview_plan` (edit.rs:2373)

#### `prepare`  (L211-279, pub)
`pub fn prepare( source: &[u8], plan: &EditPlan, options: &PrepareOptions, ) -> Result<PreparedEdit, EditError>`

- **Does:** Run the shared preparation: resolve, apply, choose the primary bytes, verify the serialized candidate, then apply `options` last.
- **Calls (crate):** `prepare_skipping` (edit/skip.rs:43) L219; `check_update_fields` (edit.rs:2329) (by name) L221; `lower` (edit/visible.rs:95) L222; `Transaction::start` (edit.rs:2680) L224; `Transaction::resolve` (edit.rs:3267) L226; `check` (edit/tracked.rs:107) L227; `Transaction::emits_directly` (edit.rs:2828) L228; `Transaction::edits_beside_a_range_marker` (edit.rs:2872) L231; `Transaction::apply` (edit.rs:6430) L232; `check_dom_realized` (edit/prepared.rs:712) L233; `produce` (edit/prepared.rs:928) L234; `primary_effect` (edit/prepared.rs:284) L235; `check_candidates` (edit/prepared.rs:397) L243; `check_rejected_ranges` (edit/prepared.rs:330) L246; `err` (edit/prepared.rs:1047) L261; `source_sha256` (inspect.rs:299) (by name) L269,270,271; `EditPlan::to_json` (edit.rs:1846) L270
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.as_ref`, `.clone`, `.map_err`, `.to_vec`, `.unwrap_or`, `Err`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::verification_reports_only_completed_candidate_checks` (edit/prepared.rs:1172); `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (edit/prepared.rs:1292); `prepare_skipping` (edit/skip.rs:43) (?); `rename_positions` (edit/skip.rs:394) (?); `apply_plan_json_with` (edit.rs:2284); `apply_plan` (edit.rs:2322); `preview_plan_with` (edit.rs:2380)

#### `primary_effect`  (L284-322)
`fn primary_effect(tx: &Transaction<'_>, result: &EditResult) -> Effect`

- **Does:** The primary transaction's document effect.
- **Calls (crate):** `expected_texts` (edit/prepared.rs:482) L316
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.is_some`
- **Macros:** `matches!`
- **Called by:** `prepare` (edit/prepared.rs:211)

#### `check_rejected_ranges`  (L330-357)
`fn check_rejected_ranges(source: &[u8], redline: &[u8]) -> Result<(), EditError>`

- **Does:** Reject All of `redline` gives the source back, so it keeps each of the source's bookmarks (by name; Word's own `_GoBack` aside), comment ranges and permission ranges.
- **Calls (crate):** `reject_revisions` (document_comparer.rs:4985) L331; `err` (edit/prepared.rs:1047) L332,349; `range_markers` (edit/prepared.rs:368) L338
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.difference`, `.is_empty`, `.join`, `.map_err`, `.map`, `.then`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `prepare` (edit/prepared.rs:211)

#### `range_markers`  (L368-392)
`fn range_markers(docx: &[u8]) -> RangeMarkers`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::open` (inspect.rs:735) L370; `Opened::story_parts` (inspect.rs:861) (by name) L374; `parse_part` (inspect.rs:926) L376; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L379,386,389; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L380
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.insert`, `.into_iter`, `.len`, `.map`, `.to_string`, `RangeMarkers::default`, `Some`, `W::name`
- **Macros:** `vec!`
- **Called by:** `check_rejected_ranges` (edit/prepared.rs:330)

#### `check_candidates`  (L397-450)
`fn check_candidates(source: &[u8], result: &EditResult) -> Result<CandidateChecks, EditError>`

- **Does:** Findings on the input and what each serialized artifact adds.
- **Calls (crate):** `validate` (validate.rs:1140) L402,406; `err` (edit/prepared.rs:1047) L407,426; `finding_key` (edit/prepared.rs:32) L416,437; `introduced_findings` (edit/prepared.rs:43) L424; `blocking` (edit/prepared.rs:61) L425
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.iter`, `.map_err`, `.ok`, `.push`, `.unwrap_or_default`, `Err`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `prepare` (edit/prepared.rs:211); `prepare_v2` (edit/prepared.rs:1058); `tests::unvalidatable_source_never_claims_findings_introduced` (edit/prepared.rs:1157)

#### `expected_texts`  (L482-670)
`fn expected_texts(tx: &Transaction<'_>, exact: bool) -> Vec<(usize, ExpectedText)>`

- **Does:** The exact text each paragraph must read as in the candidate: resolution composed over the base projection, or, under `exact`, the `rewrite`'s requested text itself — so a requested change the algorithm normalizes away is detected instead of passing as a no-op.
- **Calls (crate):** `rewrite_text` (edit/prepared.rs:675) L549; `comment_text_of` (edit.rs:7024) (by name) L619; `outcome_paragraph` (edit/prepared.rs:692) L622
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.extend`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.or_insert_with`, `.push_str`, `.push`, `.remove`, `.sort_by_key`, `.to_string`, `.unwrap_or_else`, `BTreeMap::new`, `ExpectedScope::Control`, `HashSet::new`, `Some`, `String::with_capacity`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `primary_effect` (edit/prepared.rs:284); `check_dom_realized` (edit/prepared.rs:712)

#### `rewrite_text`  (L675-688)
`fn rewrite_text(tx: &Transaction<'_>, para: usize) -> Option<(usize, String)>`

- **Does:** The first `rewrite` op on `para` and the text it asks for, or `None` when no rewrite op resolved onto it (`cN.text` rewrites excluded: their target is the covered span, not the paragraph).
- **Calls (crate):** `comment_text_of` (edit.rs:7024) (by name) L683; `outcome_paragraph` (edit/prepared.rs:692) L686
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.find_map`, `.is_some`, `.iter`, `.then`, `Some`
- **Called by:** `expected_texts` (edit/prepared.rs:482)

#### `outcome_paragraph`  (L692-699)
`fn outcome_paragraph(tx: &Transaction<'_>, op: usize) -> Option<usize>`

- **Does:** The paragraph index an outcome names (`p3`, `header1.p1`, …) inside the transaction's story list.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L694
- **Calls (std/external/unresolved):** `.as_deref`, `.iter`, `.position`
- **Called by:** `expected_texts` (edit/prepared.rs:482); `rewrite_text` (edit/prepared.rs:675)

#### `attached`  (L702-707)
`fn attached(tx: &Transaction<'_>, para: usize) -> bool`

- **Does:** `para`'s node still belongs to its story in the mutated DOM.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L706
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `check_dom_realized` (edit/prepared.rs:712)

#### `check_dom_realized`  (L712-750, pub(super))
`pub(super) fn check_dom_realized(tx: &Transaction<'_>, exact: bool) -> Result<(), EditError>`

- **Does:** Each expected paragraph reads exactly so after apply, in the DOM the candidate is serialized from.
- **Calls (crate):** `expected_texts` (edit/prepared.rs:482) L713; `attached` (edit/prepared.rs:702) L714; `project_paragraph` (inspect.rs:1265) (by name) L721; `control_text` (edit/prepared.rs:753) L723; `err` (edit/prepared.rs:1047) L734; `paragraph_id` (ids.rs:410) L739
- **Calls (std/external/unresolved):** `.clone`, `.replace`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `prepare` (edit/prepared.rs:211); `apply_bound` (edit/v2_apply.rs:121)

#### `control_text`  (L753-757)
`fn control_text(dom: &crate::xmllinq::Dom, sdt: crate::xmllinq::NodeId) -> String`

- **Does:** The projected text of a content control (`w:sdtContent` content).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L754; `project_paragraph` (inspect.rs:1265) (by name) L755
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`, `crate::namespaces::W::sdt_content`
- **Called by:** `check_dom_realized` (edit/prepared.rs:712)

#### `check_serialized_stories`  (L763-841, pub(super))
`pub(super) fn check_serialized_stories( tx: &Transaction<'_>, clean: &[u8], ) -> Result<(), EditError>`

- **Does:** The serialized paragraphs of every story the plan wrote read exactly as the DOM they were written from — the body always, plus each header, footer and notes story an operation touched: a character the writer dropped or restated is `POSTCONDITION_FAILED`.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) (by name) L767; `paragraphs` (inspect.rs:312) L768; `err` (edit/prepared.rs:1047) L769,771,784,804,809,816...; `project_paragraph` (inspect.rs:1265) (by name) L782,827; `stories` (inspect.rs:353) L803; `story_paragraph_nodes` (inspect.rs:1178) L807
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.find`, `.is_empty`, `.iter`, `.len`, `.map_err`, `.to_string`, `.zip`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `produce` (edit/prepared.rs:928); `apply_bound` (edit/v2_apply.rs:121)

#### `check_redline_stories`  (L844-924)
`fn check_redline_stories( tx: &Transaction<'_>, clean: &[u8], redline: &[u8], ) -> Result<(), EditError>`

- **Does:** Compare the published redline's accepted text with the final clean copy.
- **Calls (crate):** `accept_revisions` (document_comparer.rs:4962) L884; `err` (edit/prepared.rs:1047) L885,891,903,915; `move_range_bookmark_regressions::accept` (revision_processor/bookmarks.rs:181) (by name) L887,888; `paragraphs` (inspect.rs:312) L890; `check_redline_stories::{fn compare}` (edit/prepared.rs:849) L893,921; `paragraphs` (ambiguous, 11 candidates) L895,896; `stories` (inspect.rs:353) L902; `stories` (inspect.rs:353) (by name) L905,906
- **Calls (std/external/unresolved):** `.all`, `.find`, `.iter`, `.map_err`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `produce` (edit/prepared.rs:928); `tests::redline_only_defects_name_the_story_and_paragraph` (edit/prepared.rs:1190)

#### `check_redline_stories::{fn compare}`  (L849-879)
`fn compare( part: &str, clean: &[crate::inspect::Paragraph], redline: &[crate::inspect::Paragraph], ) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit/prepared.rs:1047) L855,868
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.min`, `.zip`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::the_same_pixels_in_different_png_bytes_are_unchanged` (convert/diff_render.rs:264) (?); `tests::one_changed_pixel_is_boxed_exactly` (convert/diff_render.rs:283) (?); `check_redline_stories` (edit/prepared.rs:844)

#### `produce`  (L928-1044)
`fn produce( tx: &mut Transaction<'_>, emit: bool, role: PrimaryRole, ) -> Result<EditResult, EditError>`

- **Does:** Serialize and check the prepared DOM before field updates, then check the published redline against the final clean candidate.
- **Calls (crate):** `Transaction::finish` (edit.rs:6645) L934; `check_serialized_stories` (edit/prepared.rs:763) L935; `Transaction::commented_base` (edit.rs:6518) (by name) L937; `result` (edit/tracked.rs:199) L939; `Transaction::check_redactions` (edit.rs:5846) L940,1029; `check_redline_stories` (edit/prepared.rs:844) L942,1037; `update_fields` (fields.rs:115) L949; `err` (edit/prepared.rs:1047) L950,968,1026; `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493) L962; `Transaction::commented_base` (edit.rs:6518) L965; `compare_documents_for_edit` (document_comparer.rs:6432) L967; `record_mid_changes` (edit/sections.rs:453) L974; `restore` (edit/ranges.rs:432) L976; `rewrite` (edit/whole.rs:130) L978; `comment_holds_kept_text` (edit.rs:6932) (by name) L993; `Transaction::conflict` (edit.rs:5870) L1010; `lift_deletion_comments` (edit.rs:6914) (by name) L1022; `apply_settings_to_docx` (settings.rs:424) L1025; `typing_order` (edit/replacement_order.rs:82) L1028; `Transaction::report` (edit.rs:3161) L1030; `paragraphs` (inspect.rs:312) L1031; `revision_counts` (ambiguous: `revision_counts` (edit/tracked.rs:404); `revision_counts` (edit.rs:2396)) L1034
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.collect`, `.filter`, `.find`, `.into`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::new`, `refresh`, `std::borrow::Cow::Borrowed`
- **Macros:** `format!`, `matches!`
- **Called by:** `prepare` (edit/prepared.rs:211)

#### `err`  (L1047-1049)
`fn err(code: &str, operation: Option<&str>, message: impl Into<String>) -> EditError`

- **Does:** An [`EditError`] as `super::err` builds it.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1048
- **Called by:** `AnchorChanges::records` (edit/comment_settlement.rs:78) (?); `cursor_offset` (edit/for_edit.rs:503) (?); `Selection::build` (edit/for_edit.rs:547) (?); `text_selection` (edit/for_edit.rs:663) (?); `changed_selection` (edit/for_edit.rs:703) (?); `for_edit` (edit/for_edit.rs:1581) (?); `prepare` (edit/prepared.rs:211); `check_rejected_ranges` (edit/prepared.rs:330); `check_candidates` (edit/prepared.rs:397); `check_dom_realized` (edit/prepared.rs:712); `check_serialized_stories` (edit/prepared.rs:763); `check_redline_stories` (edit/prepared.rs:844); +39 more

#### `prepare_v2`  (L1058-1111, pub)
`pub fn prepare_v2( source: &[u8], plan: &super::v2::PlanV2, options: &PrepareOptions, ) -> Result<PreparedEdit, EditError>`

- **Does:** Prepare a PlanV2 through the existing source/working-baseline adapter and the same serialized candidate checks as v1.
- **Calls (crate):** `apply_quoting` (edit/v2_apply.rs:86) L1069; `check_candidates` (edit/prepared.rs:397) L1082; `err` (edit/prepared.rs:1047) L1085; `source_sha256` (inspect.rs:299) (by name) L1092,1093,1098
- **Calls (std/external/unresolved):** `.as_bytes`, `.clone`, `.expect`, `Err`, `Ok`, `Some`, `Vec::new`, `serde_json::to_string`
- **Called by:** `tests::final_text` (edit/inline.rs:403) (?); `tests::inline_insert_edges_and_paragraph_sides_use_explicit_kinds` (edit/inline.rs:535) (?); `tests::inline_decisions_freeze_original_ids_and_working_rewrite_hash` (edit/inline.rs:612) (?); `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (edit/prepared.rs:1292)

## Test functions (7)

- `tests::result_of` (L1121): _no doc comment_
- `tests::unvalidatable_source_never_claims_findings_introduced` (L1157): A source the full validator cannot read must not become an empty baseline that calls every candidate finding introduced: with no trustworthy baseline nothing is claimed introduced, while the candidate's own validation still runs (fail-safe disclosure, not a fabricated verdict).
- `tests::verification_reports_only_completed_candidate_checks` (L1172): _no doc comment_
- `tests::redline_only_defects_name_the_story_and_paragraph` (L1190): _no doc comment_
- `tests::serialized_candidate` (L1287): A deterministic serializer fake: only the current test thread sees it. _(from a code comment)_
- `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (L1292): _no doc comment_
- `<Reset as Drop>::drop` (L1295): _no doc comment_
