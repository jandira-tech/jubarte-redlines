# Functions and methods in `src/markdown/view_stories.rs`

40 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `ParaFormat::read`  (L94-165)
`fn read(entries: &[(String, Value)]) -> Option<Self>`

- **Does:** The `center`/`right`, `indent:`, `color:`, `border:` and `frame:` of a `pN: {…}` entry; `None` when its indent, border or frame does not read.
- **Calls (crate):** `text_of` (changes.rs:1518) (by name) L102,155; `parse` (markdown/borders.rs:232) L103; `valid` (markdown/frames.rs:37) L116; `align_of` (markdown/view_stories.rs:170) L154
- **Calls (std/external/unresolved):** `.abs`, `.all`, `.any`, `.as_str`, `.chars`, `.clone`, `.collect`, `.filter`, `.find_map`, `.find`, `.is_ascii_alphanumeric`, `.is_finite`, `.is_some`, `.iter`, `.map`, `.ok`, `.parse`, `.position`, `.round`, `.strip_suffix`, `Some`
- **Called by:** `story` (markdown/view_stories.rs:713)

#### `align_of`  (L170-174)
`fn align_of(flag: impl Fn(&str) -> bool) -> Option<&'static str>`

- **Does:** The alignment the entry names: `center`, `right`, or a stated `left` or `justify`; `None` when it names none (the style's applies).
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `flag`
- **Called by:** `ParaFormat::read` (markdown/view_stories.rs:94); `story` (markdown/view_stories.rs:713)

#### `Pool::ids`  (L229-235)
`fn ids(&self, ids: &[u32], slot: usize) -> Vec<u32>`

- **Does:** The ids mark `slot` takes: one each, in order; the last mark the rest, and a mark past the ids none (a fresh one).
- **Calls (std/external/unresolved):** `.get`, `.to_vec`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** none found in src

#### `other_mark`  (L254-256)
`fn other_mark(name: &str) -> &'static str`

- **Does:** The other of `w:ins` and `w:del`: each `re` level alternates them.
- **Calls:** none
- **Called by:** `reads` (markdown/view_stories.rs:259)

#### `reads`  (L259-339)
`fn reads(text: &str, out: &mut Vec<Read>)`

- **Does:** [`Read`]s of one paragraph's text, as [`runs`] reads it.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) L273; `change` (markdown/view_stories.rs:1227) L278; `tags_after` (markdown/view_stories.rs:1281) L279; `other_mark` (markdown/view_stories.rs:254) L298,315; `field` (markdown/view_stories.rs:1312) L327; `reads` (markdown/view_stories.rs:259) L330
- **Calls (std/external/unresolved):** `.all`, `.as_slice`, `.chars`, `.clone`, `.collect`, `.copied`, `.count`, `.find`, `.first`, `.flat_map`, `.insert`, `.iter`, `.len_utf8`, `.next`, `.push`, `.strip_prefix`, `.take_while`, `.truncate`, `Change::Insert`, `Read::Own`, `Read::Parent`, `Some`
- **Macros:** `matches!`
- **Called by:** `reads` (markdown/view_stories.rs:259); `story_part` (markdown/view_stories.rs:1710)

#### `pools`  (L349-427)
`fn pools(read: &[Read]) -> HashMap<Vec<u32>, Pool>`

- **Does:** The joined ids a story part prints on more than one piece, each with the mark every piece and `re` takes.
- **Calls (std/external/unresolved):** `.as_slice`, `.clear`, `.clone`, `.entry`, `.get`, `.insert`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.len`, `.map_or`, `.or_default`, `.push_back`, `.push`, `.saturating_sub`, `HashMap::new`, `Some`, `Vec::new`, `place`, `pooled`, `spare`, `split`, `std::mem::take`
- **Called by:** `story_part` (markdown/view_stories.rs:1710)

#### `Marks::take`  (L432-436)
`fn take(&mut self, tag: &super::view_notes::Tag) -> Option<Vec<u32>>`

- **Does:** The ids the next piece of `tag` takes, when its joined id prints on several pieces; `None` when the tag's own ids apply.
- **Calls (crate):** `.ids` (ambiguous, 6 candidates) L435
- **Calls (std/external/unresolved):** `.get_mut`, `.pop_front`, `.unwrap_or`, `Some`
- **Called by:** `Marks::piece` (markdown/view_stories.rs:440); `runs` (markdown/view_stories.rs:941)

#### `Marks::piece`  (L440-446)
`fn piece(&mut self, name: &str, tag: &super::view_notes::Tag) -> String`

- **Does:** A piece's mark `name`: the next of its pooled ids, else its tag's first.
- **Calls (crate):** `Marks::take` (markdown/view_stories.rs:432) L441; `Marks::mark_id` (markdown/view_stories.rs:471) L445
- **Calls (std/external/unresolved):** `.copied`, `.first`, `Some`
- **Called by:** `story_part` (markdown/view_stories.rs:1710) (?)

#### `Marks::parent`  (L450-459)
`fn parent(&mut self, name: &str, tag: &super::view_notes::Tag) -> String`

- **Does:** The mark `name` of a change `re` names around a nested one: the mark of it [`pools`] found the nest in.
- **Calls (crate):** `.ids` (ambiguous, 6 candidates) L454; `Marks::mark_id` (markdown/view_stories.rs:471) L458
- **Calls (std/external/unresolved):** `.copied`, `.first`, `.get_mut`, `.pop_front`, `.unwrap_or`, `Some`
- **Called by:** `Package::empty_spacers` (audit.rs:661) (?); `absolute` (bin/jubarte.rs:168) (?); `run_convert` (bin/jubarte.rs:646) (?); `publish_edit` (bin/jubarte.rs:1402) (?); `run_debug_diff` (bin/jubarte.rs:3871) (?); `resolve_plan` (cli.rs:613) (?); `tests::check_boundary_revision` (comparer/bookmarks.rs:819) (?); `deeper_boundary_tests::placement_empty_start_and_past_last_boundaries` (comparer/comments.rs:2131) (?); `deeper_boundary_tests::point_after_normal_mark_does_not_clone_next_deleted_mark` (comparer/comments.rs:2183) (?); `deeper_boundary_tests::deletion_exit_requires_only_markers_after_the_end` (comparer/comments.rs:2230) (?); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540) (?); `coverage_tests::trailing_spaced_empty_requires_insert_dominance_and_deleted_predecessor` (comparer/finalize.rs:16600) (?); +45 more

#### `Marks::mark`  (L463-466)
`fn mark(&mut self, name: &str, tag: Option<&super::view_notes::Tag>, at: usize) -> String`

- **Does:** `w:ins w:id="5" w:author="…" w:date="…"`: the start tag's name and attributes, for `tag` or the document's author.
- **Calls (crate):** `Marks::mark_id` (markdown/view_stories.rs:471) L465
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.get`
- **Called by:** `story_part` (markdown/view_stories.rs:1710) (?)

#### `Marks::mark_id`  (L471-519)
`fn mark_id( &mut self, name: &str, tag: Option<&super::view_notes::Tag>, id: Option<u32>, ) -> String`

- **Does:** The mark `name` with `id`, by the tag's author; a side the tag numbers none for (`r5` after a substitution names its deleted side) takes a fresh id.
- **Calls (crate):** `date_attribute` (markdown/xml.rs:2921) L512; `escape` (markdown/xml.rs:601) L515
- **Calls (std/external/unresolved):** `.and_then`, `.borrow_mut`, `.clone`, `.get`, `.insert`, `.map`, `.saturating_add`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `Marks::piece` (markdown/view_stories.rs:440); `Marks::parent` (markdown/view_stories.rs:450); `Marks::mark` (markdown/view_stories.rs:463); `format_changes` (markdown/view_stories.rs:525); `tracked_piece` (markdown/view_stories.rs:1437) (?)

#### `format_changes`  (L525-557, pub(super))
`pub(super) fn format_changes( people: &People, tags: &[&super::view_notes::Tag], first_id: u32, ) -> Vec<String>`

- **Does:** The start tags of the `w:pPrChange`s `tags` name, by their people: one per tag, a tag that numbers no `w:id` taking a fresh one above `first_id`.
- **Calls (crate):** `Marks::mark_id` (markdown/view_stories.rs:471) L554
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.first`, `.flat_map`, `.insert`, `.iter`, `.map`, `.unwrap_or_else`, `HashMap::new`, `Some`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `flow_text`  (L572-580, pub(super))
`pub(super) fn flow_text(text: &str, key: &str) -> Option<String>`

- **Does:** A text clause from the shared flow-map parser, honoring both YAML quoting forms rather than looking for clause names inside strings.
- **Calls (crate):** `flow_map` (markdown/view_stories.rs:587) L573
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `Some`
- **Called by:** `note_formats` (markdown/view_notes.rs:760)

#### `flow_map`  (L587-626)
`fn flow_map(text: &str, depth: usize) -> Option<(Vec<(String, Value)>, &str)>`

- **Does:** The map `text` opens with (`{id: header3, text: "{PAGE}", inactive}`), and what follows it.
- **Calls (crate):** `scalar` (markdown/view_stories.rs:651) L598,611; `flow_map` (markdown/view_stories.rs:587) L603; `flow_list` (markdown/view_stories.rs:629) L607
- **Calls (std/external/unresolved):** `.push`, `.starts_with`, `.strip_prefix`, `.trim_start`, `Some`, `Value::List`, `Value::Map`, `Value::Text`, `Vec::new`
- **Called by:** `flow_text` (markdown/view_stories.rs:572); `flow_map` (markdown/view_stories.rs:587); `Stories::entry` (markdown/view_stories.rs:816); `section` (markdown/view_stories.rs:892)

#### `flow_list`  (L629-645)
`fn flow_list(text: &str) -> Option<(Vec<String>, &str)>`

- **Does:** The scalars of a flow sequence after its `[`, and what follows its `]`.
- **Calls (crate):** `scalar` (markdown/view_stories.rs:651) L636
- **Calls (std/external/unresolved):** `.push`, `.strip_prefix`, `.trim_start`, `Some`, `Vec::new`
- **Called by:** `flow_map` (markdown/view_stories.rs:587)

#### `scalar`  (L651-690)
`fn scalar(text: &str) -> Option<(String, &str)>`

- **Does:** A flow scalar: a JSON string (`"{PAGE}"`), a single-quoted one, which keeps backslashes and reads `''` as `'` (`'{REF _R \h}'`), or plain text up to the next `,`, `:` followed by a space, or `}` (a plain flow scalar holds none of `,[]{}` and no `: `).
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.is_empty`, `.is_some`, `.len`, `.map_or`, `.next_if`, `.next`, `.ok`, `.peekable`, `.push`, `.skip`, `.starts_with`, `.strip_prefix`, `.then`, `.to_string`, `.trim`, `Some`, `String::new`, `serde_json::from_str`
- **Macros:** `matches!`
- **Called by:** `flow_map` (markdown/view_stories.rs:587); `flow_list` (markdown/view_stories.rs:629)

#### `story_id`  (L694-706)
`fn story_id(id: &str) -> Option<(String, usize)>`

- **Does:** The part stem and paragraph of a story id: `header2` → (`header2`, 0), `header2.p1` → (`header2`, 1).
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.ok`, `.or_else`, `.parse`, `.split_once`, `.strip_prefix`, `.then`, `.to_string`
- **Called by:** `story` (markdown/view_stories.rs:713); `story_part` (markdown/view_stories.rs:1710)

#### `story`  (L713-803)
`fn story(kind: &'static str, ty: &str, entries: &[(String, Value)]) -> Option<Story>`

- **Does:** The story a `first: {…}` entry names; `None` for `none` or what does not read.
- **Calls (crate):** `story_id` (markdown/view_stories.rs:694) L727; `text_of` (changes.rs:1518) (by name) L727,769,777,792; `restore` (markdown/critic.rs:1861) L754,767,769; `ParaFormat::read` (markdown/view_stories.rs:94) L755; `align_of` (markdown/view_stories.rs:170) L798
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.filter_map`, `.find_map`, `.find`, `.first`, `.is_empty`, `.iter`, `.last`, `.map`, `.ok`, `.parse`, `.pop`, `.push`, `.sort_by_key`, `.split_off`, `.split`, `.strip_prefix`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `ParaFormat::default`, `Some`, `String::new`, `Vec::new`, `flag`, `tags`
- **Macros:** `vec!`
- **Called by:** `Stories::entry` (markdown/view_stories.rs:816); `section` (markdown/view_stories.rs:892)

#### `Stories::entry`  (L816-832)
`fn entry(&mut self, kind: &'static str, ty: &str, value: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `flow_map` (markdown/view_stories.rs:587) L822; `story` (markdown/view_stories.rs:713) L822
- **Calls (std/external/unresolved):** `.and_then`, `.push`, `.retain`, `.trim`, `Some`
- **Macros:** `format!`
- **Called by:** `first_section` (markdown/view_stories.rs:839)

#### `first_section`  (L839-888, pub(super))
`pub(super) fn first_section(yaml: &str) -> Stories`

- **Does:** The first section's stories, from the YAML header's `headers:` and `footers:` blocks (` first: {id: header3, text: …} # comment`).
- **Calls (crate):** `Stories::entry` (markdown/view_stories.rs:816) L854,881; `without_comment` (markdown/view_styles.rs:335) L859; `flush` (markdown/view_stories.rs:1334) L869,886
- **Calls (std/external/unresolved):** `.as_mut`, `.as_str`, `.collect`, `.into_iter`, `.is_empty`, `.is_some`, `.join`, `.lines`, `.map`, `.push`, `.split_once`, `.starts_with`, `.strip_suffix`, `.take`, `.to_string`, `.trim_end`, `.trim`, `Some`, `Stories::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes` (markdown/view_stories.rs:2092); `tests::the_first_sections_blocks_read` (markdown/view_stories.rs:2110); `Writer::metadata` (markdown/write.rs:4453)

#### `section`  (L892-928, pub(super))
`pub(super) fn section(entry: &str) -> Stories`

- **Does:** A later section's stories, from its `sections:` entry (`{p11-p147, headers: {default: {…}}, footers: {…}}`): only those it changes.
- **Calls (crate):** `flow_map` (markdown/view_stories.rs:587) L894; `story` (markdown/view_stories.rs:713) L911
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.push`, `.to_string`, `Some`, `Stories::default`
- **Macros:** `format!`
- **Called by:** `tests::an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes` (markdown/view_stories.rs:2092); `tests::a_later_section_reads_its_own_stories` (markdown/view_stories.rs:2131); `tests::what_does_not_read_is_left_out` (markdown/view_stories.rs:2142); `Writer::metadata` (markdown/write.rs:4453)

#### `runs`  (L941-1122)
`fn runs(text: &str, marks: &mut Marks<'_>, drawings: &HashMap<String, String>) -> String`

- **Does:** A story's text as paragraph content: a field printed as the body prints it (`{PAGE \\* roman}i{/}`) becomes a simple field with that code and result, and a bare `{PAGE}` or other `{NAME}` one with the result `1`; a line break is a `w:br`.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1360) L960; `flush` (markdown/view_stories.rs:1334) L963,975,1102,1120; `change` (markdown/view_stories.rs:1227) L970; `tags_after` (markdown/view_stories.rs:1281) L976; `number_noid_parent` (markdown/view_stories.rs:1132) L995; `Marks::take` (markdown/view_stories.rs:432) L1004,1054,1055; `tracked` (markdown/view_stories.rs:1393) L1010,1065,1084; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L1036,1076; `substitution_ids` (markdown/view_stories.rs:1380) L1058; `append_revision` (markdown/view_stories.rs:1246) L1095; `field` (markdown/view_stories.rs:1312) L1099; `escape` (markdown/xml.rs:601) L1106; `runs` (markdown/view_stories.rs:941) L1107
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.as_slice`, `.chars`, `.clear`, `.collect`, `.copied`, `.count`, `.find`, `.first`, `.flat_map`, `.get`, `.insert`, `.is_empty`, `.iter`, `.len_utf8`, `.len`, `.next`, `.push_str`, `.push`, `.reverse`, `.strip_prefix`, `.take_while`, `.to_string`, `.trim`, `.truncate`, `.unwrap_or`, `Change::Insert`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `runs` (markdown/view_stories.rs:941); `story_part` (markdown/view_stories.rs:1710); `tests::written` (markdown/view_stories.rs:2153)

#### `number_noid_parent`  (L1132-1189)
`fn number_noid_parent( name: &'static str, tags: &mut [super::view_notes::Tag], parents: &mut [super::view_notes::Tag], after: &str, marks: &mut Marks<'_>, current: &mut Vec<NoidParent>, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `names_noid_parent` (markdown/view_stories.rs:1193) L1166
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.filter`, `.first_mut`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.iter_mut`, `.last`, `.len`, `.push`, `.rev`, `.saturating_add`, `.truncate`, `.zip`, `Some`
- **Macros:** `vec!`
- **Called by:** `runs` (markdown/view_stories.rs:941)

#### `names_noid_parent`  (L1193-1218)
`fn names_noid_parent(mut text: &str, name: &str, tag: &super::view_notes::Tag) -> bool`

- **Does:** Only a contiguous nest shares an anonymous id.
- **Calls (crate):** `change` (markdown/view_stories.rs:1227) L1194; `tags_after` (markdown/view_stories.rs:1281) L1200
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.is_empty`
- **Called by:** `number_noid_parent` (markdown/view_stories.rs:1132)

#### `change`  (L1227-1240)
`fn change(text: &str) -> Option<(Change<'_>, &str)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.split_once`, `.strip_prefix`, `Change::Delete`, `Change::Insert`, `Change::Substitute`, `Some`
- **Called by:** `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (convert/mod.rs:52974) (?); `track_properties` (edit/tracked.rs:888) (?); `reads` (markdown/view_stories.rs:259); `runs` (markdown/view_stories.rs:941); `names_noid_parent` (markdown/view_stories.rs:1193); `with_style_changes` (markdown/view_styles.rs:45) (?)

#### `append_revision`  (L1246-1278)
`fn append_revision( out: &mut String, changed: &str, mut wrappers: Vec<RevisionWrapper>, last: &mut Vec<RevisionWrapper>, )`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.count`, `.ends_with`, `.iter_mut`, `.iter`, `.len`, `.map`, `.push_str`, `.rev`, `.sum`, `.take_while`, `.take`, `.truncate`, `.zip`
- **Called by:** `runs` (markdown/view_stories.rs:941)

#### `tags_after`  (L1281-1302)
`fn tags_after( mut text: &str, ) -> ( Vec<super::view_notes::Tag>, Vec<super::view_notes::Tag>, &str, )`

- **Does:** Revision tags after a change, all named parents innermost first, and the rest.
- **Calls (crate):** `attribution_tag` (markdown/critic.rs:1335) L1290; `nested_revision_tags` (markdown/view_notes.rs:418) L1291; `revision_tags` (markdown/view_notes.rs:371) L1294
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.extend`, `.into_iter`, `.push`, `.skip`, `Vec::new`
- **Called by:** `reads` (markdown/view_stories.rs:259); `runs` (markdown/view_stories.rs:941); `names_noid_parent` (markdown/view_stories.rs:1193)

#### `field`  (L1312-1330)
`fn field(text: &str) -> Option<(Field<'_>, &str)>`

- **Does:** _no doc comment_
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1641) L1314; `is_field_code` (markdown/critic.rs:1625) L1315; `result_end` (markdown/view_stories.rs:1499) L1316
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.char_indices`, `.find`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.len`, `.strip_prefix`, `.then`, `Field::Bare`, `Field::Coded`, `Some`
- **Called by:** `reads` (markdown/view_stories.rs:259); `runs` (markdown/view_stories.rs:941); `tracked` (markdown/view_stories.rs:1393); `tracked_piece` (markdown/view_stories.rs:1437)

#### `flush`  (L1334-1349)
`fn flush(plain: &mut String, out: &mut String, deleted: bool)`

- **Does:** Plain text as runs, a line break as a `w:br`; deleted text as `w:delText`.
- **Calls (crate):** `unescape` (markdown/view_stories.rs:1353) L1336; `escape` (markdown/xml.rs:601) L1344
- **Calls (std/external/unresolved):** `.clear`, `.enumerate`, `.is_empty`, `.push_str`, `.split`
- **Macros:** `format!`
- **Called by:** `pair_gaps` (comparer/cross_para.rs:1204) (?); `resolve_paragraph_window` (comparer/lcs.rs:9735) (?); `count_words_and_tokenize` (comparer/moves.rs:87) (?); `revision_clusters` (comparer/textbox.rs:83) (?); `hf_tab_line_breaks` (convert/mod.rs:22150) (?); `Layout::emit_inline_pictures` (convert/mod.rs:27711) (?); `small_caps_pieces` (convert/mod.rs:32418) (?); `parse_table` (convert/word_subst.rs:23) (?); `align` (debug/diff.rs:1406) (?); `table_row_change` (markdown/diff.rs:255) (?); `first_section` (markdown/view_stories.rs:839); `runs` (markdown/view_stories.rs:941); +2 more

#### `unescape`  (L1353-1373)
`fn unescape(text: &str) -> String`

- **Does:** Document text the view escaped back as it was: a mark's delimiter (`{\++`, `~\>`) whole again.
- **Calls (std/external/unresolved):** `.contains`, `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `flush` (markdown/view_stories.rs:1334)

#### `substitution_ids`  (L1380-1387, pub(super))
`pub(super) fn substitution_ids(ids: &[u32]) -> (&[u32], &[u32])`

- **Does:** A substitution's one tag (`r94+95+92+93`) lists the deleted side's ids, then the inserted side's: Word numbers marks in document order and writes the insertion first, so the inserted side begins where the numbers drop.
- **Calls (std/external/unresolved):** `.len`, `.map_or`, `.min`, `.position`, `.split_at`, `.windows`
- **Called by:** `runs` (markdown/view_stories.rs:941); `Writer::end_comment` (markdown/write.rs:3681)

#### `tracked`  (L1393-1433)
`fn tracked( out: &mut String, marks: &mut Marks<'_>, (name, deleted): (&str, bool), tag: Option<&super::view_notes::Tag>, ids: &[u32], inner: &str, drawings: &HashMap<String, String>, )`

- **Does:** One change's marks: a side whose tag names several ids (`r92+93`) is a mark per id, its text split as the body splits a run (after its last space, else before its last character); a side holding a field stays one mark.
- **Calls (crate):** `field` (markdown/view_stories.rs:1312) L1404; `split_point` (markdown/xml.rs:2019) L1410; `cuts_token` (markdown/from_docx/drawing.rs:1388) L1413; `tracked_piece` (markdown/view_stories.rs:1437) L1423
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.copied`, `.ends_with`, `.enumerate`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.map`, `.rev`, `.splice`, `.split_at`
- **Macros:** `vec!`
- **Called by:** `runs` (markdown/view_stories.rs:941)

#### `tracked_piece`  (L1437-1492)
`fn tracked_piece( out: &mut String, marks: &mut Marks<'_>, (name, deleted): (&str, bool), tag: Option<&super::view_notes::Tag>, id: Option<u32>, inner: &str, drawings: &HashMap<String, String>, )`

- **Does:** `inner` inside a `w:ins` or `w:del`: its text and its fields, which are complex there (a `w:fldSimple` cannot sit in a mark).
- **Calls (crate):** `Marks::mark_id` (markdown/view_stories.rs:471) (by name) L1446; `at` (markdown/from_docx/drawing.rs:1360) L1455; `flush` (markdown/view_stories.rs:1334) L1456,1464,1482,1490; `field` (markdown/view_stories.rs:1312) L1463; `escape` (markdown/xml.rs:601) L1479
- **Calls (std/external/unresolved):** `.chars`, `.get`, `.len_utf8`, `.next`, `.push_str`, `.push`, `.strip_prefix`, `.to_string`, `.trim`, `String::new`
- **Macros:** `write!`
- **Called by:** `tracked` (markdown/view_stories.rs:1393)

#### `result_end`  (L1499-1524)
`fn result_end(text: &str) -> Option<usize>`

- **Does:** Where the `{/}` that closes a field's result sits in `text`, the text after its `{CODE}`: past any field nested in the result.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1641) L1514; `is_field_code` (markdown/critic.rs:1625) L1515
- **Calls (std/external/unresolved):** `.char_indices`, `.chars`, `.len`, `.map_or`, `.next`, `.starts_with`, `.strip_prefix`, `Some`
- **Called by:** `field` (markdown/view_stories.rs:1312)

#### `SectionStories::apply`  (L1539-1573, pub(super))
`pub(super) fn apply(&self, number: usize, section: &str) -> String`

- **Does:** `section` (a `w:sectPr`) for section `number`, with the references and title page the view names in place of its own.
- **Calls (crate):** `without_empty_elements` (markdown/view_stories.rs:1577) L1545
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.filter_map`, `.find`, `.get`, `.insert_str`, `.iter`, `.map`, `.min`, `.to_string`, `.unwrap_or_default`
- **Called by:** `paragraph_properties` (markdown/xml.rs:2039)

#### `without_empty_elements`  (L1577-1593)
`fn without_empty_elements(xml: &str, name: &str) -> String`

- **Does:** `xml` without its empty `<name …/>` elements.
- **Calls (std/external/unresolved):** `.ends_with`, `.find`, `.len`, `.push_str`, `.starts_with`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `SectionStories::apply` (markdown/view_stories.rs:1539)

#### `write`  (L1599-1685, pub(super))
`pub(super) fn write( package: &mut PartFs, main: &str, sections: &BTreeMap<usize, Stories>, count: usize, people: &People, first_id: u32, default_style: &str, ) -> Result<SectionStories, super::MarkdownError>`

- **Does:** Writes the headers and footers `sections` name, by section number from one, into `package` beside `main`: one part each, named by its id (`header2.xml`), in place of any the reference document had.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L1614; `PartFs::remove_related_part` (opc/mod.rs:923) L1625; `max_drawing_id` (markdown/package.rs:1438) L1636; `story_part` (markdown/view_stories.rs:1710) L1653; `reference_rank` (markdown/view_stories.rs:1690) L1666; `even_and_odd_headers` (markdown/view_stories.rs:2006) L1676
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.max`, `.push`, `.sort_by_key`, `.unwrap_or_default`, `.unwrap_or`, `.values`, `Err`, `HashMap::new`, `Ok`, `SectionStories::default`, `String::new`, `Vec::new`, `super::MarkdownError::Package`
- **Macros:** `format!`
- **Called by:** `written_places` (edit/origin.rs:649) (?); `Writer::revised` (markdown/from_docx/mod.rs:2614) (?); `assemble` (markdown/package.rs:103)

#### `reference_rank`  (L1690-1699)
`fn reference_rank(kind: &str, ty: &str) -> usize`

- **Does:** Where Word writes a `kind` reference of type `ty` in a `w:sectPr`: even and default headers, even and default footers, then the first header and footer (every section Word wrote in the bench corpus).
- **Calls:** none
- **Called by:** `write` (markdown/view_stories.rs:1599)

#### `story_part`  (L1710-2002)
`fn story_part( package: &mut PartFs, main: &str, story: &Story, default_style: &str, marks: &mut Marks<'_>, next_doc_pr: &mut u32, ) -> String`

- **Does:** Writes `story`'s part and relates it to `main`: its relationship id.
- **Calls (crate):** `story_id` (markdown/view_stories.rs:694) L1734,1828; `revision_tags` (markdown/view_notes.rs:371) L1735,1757,1764,1832,1838,1855; `sibling` (markdown/package.rs:504) L1775; `PartFs::parts` (opc/mod.rs:733) L1778; `at` (markdown/from_docx/drawing.rs:1360) L1793; `lookup` (markdown/from_docx/drawing.rs:1460) L1800; `materialize` (markdown/from_docx/drawing.rs:1881) L1807; `Drawing::wraps_run` (markdown/from_docx/drawing.rs:1818) (by name) L1811; `attribution_tag` (markdown/critic.rs:1335) L1848; `reads` (markdown/view_stories.rs:259) L1876; `pools` (markdown/view_stories.rs:349) L1886; `escape` (markdown/xml.rs:601) L1899; `xml` (markdown/frames.rs:116) L1908; `xml` (markdown/borders.rs:254) L1911; `runs` (markdown/view_stories.rs:941) L1944; `Marks::piece` (markdown/view_stories.rs:440) (by name) L1950,1977; `.mark` (ambiguous: `Marks::mark` (markdown/view_stories.rs:463); `NsScope::mark` (xmllinq/parse.rs:193)) L1963; `PartFs::set_part` (opc/mod.rs:661) L1982; `PartFs::add_content_type_override` (opc/mod.rs:845) L1990; `PartFs::add_document_relationship` (opc/mod.rs:1003) L1997; `relative_rel_target` (opc/mod.rs:180) L2000
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_slice`, `.as_str`, `.chain`, `.chars`, `.checked_sub`, `.clear`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.entry`, `.enumerate`, `.extend`, `.filter`, `.find_map`, `.find`, `.flatten`, `.get_mut`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_some`, `.iter_mut`, `.iter`, `.keys`, `.len_utf8`, `.len`, `.map_or`, `.map`, `.max`, `.next`, `.or_default`, `.push_str`, `.push`, `.remove`, `.replace`, `.resize`, `.rev`, `.rsplit_once`, `.sort_by_key`, `.split_once`, `.strip_prefix`, `.then_some`, `.to_string`, `.unwrap_or_default`, `HashMap::new`, `HashSet::from`, `Read::Mark`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`, `vec!`, `write!`
- **Called by:** `write` (markdown/view_stories.rs:1599)

#### `even_and_odd_headers`  (L2006-2026)
`fn even_and_odd_headers(package: &mut PartFs, main: &str)`

- **Does:** Turns on Word's even and odd headers in the settings part `main` relates to.
- **Calls (crate):** `related` (markdown/package.rs:512) L2007; `PartFs::part_string` (opc/mod.rs:636) L2010; `PartFs::set_part` (opc/mod.rs:661) L2024
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.filter_map`, `.find`, `.insert_str`, `.into_bytes`, `.iter`, `.min`
- **Called by:** `write` (markdown/view_stories.rs:1599)

## Test functions (7)

- `tests::an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes` (L2092): _no doc comment_
- `tests::the_first_sections_blocks_read` (L2110): _no doc comment_
- `tests::a_later_section_reads_its_own_stories` (L2131): _no doc comment_
- `tests::what_does_not_read_is_left_out` (L2142): _no doc comment_
- `tests::written` (L2153): `text` as runs, with Ann Counsel as `@AC`.
- `tests::fields_and_changes_become_runs_and_marks` (L2180): _no doc comment_
- `tests::a_field_with_its_result_keeps_its_code_and_result` (L2217): _no doc comment_
