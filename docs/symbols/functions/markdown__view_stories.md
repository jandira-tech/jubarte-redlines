# Functions and methods in `src/markdown/view_stories.rs`

30 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `ParaFormat::read`  (L81-103)
`fn read(entries: &[(String, Value)]) -> Option<Self>`

- **Does:** The `center`/`right`, `color:` and `border:` of a `pN: {…}` entry; `None` when its border does not read.
- **Calls (crate):** `text_of` (changes.rs:806) (by name) L89,98; `parse` (markdown/borders.rs:232) L90; `align_of` (markdown/view_stories.rs:108) L97
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.chars`, `.filter`, `.find_map`, `.is_ascii_alphanumeric`, `.iter`, `.map`, `Some`
- **Called by:** `story` (markdown/view_stories.rs:384)

#### `align_of`  (L108-112)
`fn align_of(flag: impl Fn(&str) -> bool) -> Option<&'static str>`

- **Does:** The alignment the entry names: `center`, `right`, or a stated `left` or `justify`; `None` when it names none (the style's applies).
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `flag`
- **Called by:** `ParaFormat::read` (markdown/view_stories.rs:81); `story` (markdown/view_stories.rs:384)

#### `Marks::mark`  (L144-147)
`fn mark(&mut self, name: &str, tag: Option<&super::view_notes::Tag>, at: usize) -> String`

- **Does:** `w:ins w:id="5" w:author="…" w:date="…"`: the start tag's name and attributes, for `tag` or the document's author.
- **Calls (crate):** `Marks::mark_id` (markdown/view_stories.rs:152) L146
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.get`
- **Called by:** `story_part` (markdown/view_stories.rs:1118) (?)

#### `Marks::mark_id`  (L152-193)
`fn mark_id( &mut self, name: &str, tag: Option<&super::view_notes::Tag>, id: Option<u32>, ) -> String`

- **Does:** The mark `name` with `id`, by the tag's author; a side the tag numbers none for (`r5` after a substitution names its deleted side) takes a fresh id.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L187,191
- **Calls (std/external/unresolved):** `.borrow_mut`, `.clone`, `.get`, `.insert`, `.is_empty`, `.map`, `.saturating_add`, `.unwrap_or_else`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `Marks::mark` (markdown/view_stories.rs:144); `format_changes` (markdown/view_stories.rs:199); `tracked_piece` (markdown/view_stories.rs:850) (?)

#### `format_changes`  (L199-228, pub(super))
`pub(super) fn format_changes( people: &People, tags: &[&super::view_notes::Tag], first_id: u32, ) -> Vec<String>`

- **Does:** The start tags of the `w:pPrChange`s `tags` name, by their people: one per tag, a tag that numbers no `w:id` taking a fresh one above `first_id`.
- **Calls (crate):** `Marks::mark_id` (markdown/view_stories.rs:152) L225
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.first`, `.flat_map`, `.insert`, `.iter`, `.map`, `.unwrap_or_else`, `Some`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `flow_text`  (L243-251, pub(super))
`pub(super) fn flow_text(text: &str, key: &str) -> Option<String>`

- **Does:** A text clause from the shared flow-map parser, honoring both YAML quoting forms rather than looking for clause names inside strings.
- **Calls (crate):** `flow_map` (markdown/view_stories.rs:258) L244
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `Some`
- **Called by:** `note_formats` (markdown/view_notes.rs:730)

#### `flow_map`  (L258-297)
`fn flow_map(text: &str, depth: usize) -> Option<(Vec<(String, Value)>, &str)>`

- **Does:** The map `text` opens with (`{id: header3, text: "{PAGE}", inactive}`), and what follows it.
- **Calls (crate):** `scalar` (markdown/view_stories.rs:322) L269,282; `flow_map` (markdown/view_stories.rs:258) L274; `flow_list` (markdown/view_stories.rs:300) L278
- **Calls (std/external/unresolved):** `.push`, `.starts_with`, `.strip_prefix`, `.trim_start`, `Some`, `Value::List`, `Value::Map`, `Value::Text`, `Vec::new`
- **Called by:** `flow_text` (markdown/view_stories.rs:243); `flow_map` (markdown/view_stories.rs:258); `Stories::entry` (markdown/view_stories.rs:487); `section` (markdown/view_stories.rs:563)

#### `flow_list`  (L300-316)
`fn flow_list(text: &str) -> Option<(Vec<String>, &str)>`

- **Does:** The scalars of a flow sequence after its `[`, and what follows its `]`.
- **Calls (crate):** `scalar` (markdown/view_stories.rs:322) L307
- **Calls (std/external/unresolved):** `.push`, `.strip_prefix`, `.trim_start`, `Some`, `Vec::new`
- **Called by:** `flow_map` (markdown/view_stories.rs:258)

#### `scalar`  (L322-361)
`fn scalar(text: &str) -> Option<(String, &str)>`

- **Does:** A flow scalar: a JSON string (`"{PAGE}"`), a single-quoted one, which keeps backslashes and reads `''` as `'` (`'{REF _R \h}'`), or plain text up to the next `,`, `:` followed by a space, or `}` (a plain flow scalar holds none of `,[]{}` and no `: `).
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.is_empty`, `.is_some`, `.len`, `.map_or`, `.next_if`, `.next`, `.ok`, `.peekable`, `.push`, `.skip`, `.starts_with`, `.strip_prefix`, `.then`, `.to_string`, `.trim`, `Some`, `String::new`, `serde_json::from_str`
- **Macros:** `matches!`
- **Called by:** `flow_map` (markdown/view_stories.rs:258); `flow_list` (markdown/view_stories.rs:300)

#### `story_id`  (L365-377)
`fn story_id(id: &str) -> Option<(String, usize)>`

- **Does:** The part stem and paragraph of a story id: `header2` → (`header2`, 0), `header2.p1` → (`header2`, 1).
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.ok`, `.or_else`, `.parse`, `.split_once`, `.strip_prefix`, `.then`, `.to_string`
- **Called by:** `story` (markdown/view_stories.rs:384); `story_part` (markdown/view_stories.rs:1118)

#### `story`  (L384-474)
`fn story(kind: &'static str, ty: &str, entries: &[(String, Value)]) -> Option<Story>`

- **Does:** The story a `first: {…}` entry names; `None` for `none` or what does not read.
- **Calls (crate):** `story_id` (markdown/view_stories.rs:365) L398; `text_of` (changes.rs:806) (by name) L398,440,448,463; `restore` (markdown/critic.rs:1662) L425,438,440; `ParaFormat::read` (markdown/view_stories.rs:81) L426; `align_of` (markdown/view_stories.rs:108) L469
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.filter_map`, `.find_map`, `.find`, `.first`, `.is_empty`, `.iter`, `.last`, `.map`, `.ok`, `.parse`, `.pop`, `.push`, `.sort_by_key`, `.split_off`, `.split`, `.strip_prefix`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `ParaFormat::default`, `Some`, `String::new`, `Vec::new`, `flag`, `tags`
- **Macros:** `vec!`
- **Called by:** `Stories::entry` (markdown/view_stories.rs:487); `section` (markdown/view_stories.rs:563)

#### `Stories::entry`  (L487-503)
`fn entry(&mut self, kind: &'static str, ty: &str, value: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `flow_map` (markdown/view_stories.rs:258) L493; `story` (markdown/view_stories.rs:384) L493
- **Calls (std/external/unresolved):** `.and_then`, `.push`, `.retain`, `.trim`, `Some`
- **Macros:** `format!`
- **Called by:** `first_section` (markdown/view_stories.rs:510)

#### `first_section`  (L510-559, pub(super))
`pub(super) fn first_section(yaml: &str) -> Stories`

- **Does:** The first section's stories, from the YAML header's `headers:` and `footers:` blocks (` first: {id: header3, text: …} # comment`).
- **Calls (crate):** `Stories::entry` (markdown/view_stories.rs:487) L525,552; `without_comment` (markdown/view_styles.rs:324) L530; `flush` (markdown/view_stories.rs:755) L540,557
- **Calls (std/external/unresolved):** `.as_mut`, `.as_str`, `.collect`, `.into_iter`, `.is_empty`, `.is_some`, `.join`, `.lines`, `.map`, `.push`, `.split_once`, `.starts_with`, `.strip_suffix`, `.take`, `.to_string`, `.trim_end`, `.trim`, `Some`, `Stories::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes` (markdown/view_stories.rs:1429); `tests::the_first_sections_blocks_read` (markdown/view_stories.rs:1447); `Writer::metadata` (markdown/write.rs:4075)

#### `section`  (L563-599, pub(super))
`pub(super) fn section(entry: &str) -> Stories`

- **Does:** A later section's stories, from its `sections:` entry (`{p11-p147, headers: {default: {…}}, footers: {…}}`): only those it changes.
- **Calls (crate):** `flow_map` (markdown/view_stories.rs:258) L565; `story` (markdown/view_stories.rs:384) L582
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.push`, `.to_string`, `Some`, `Stories::default`
- **Macros:** `format!`
- **Called by:** `tests::an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes` (markdown/view_stories.rs:1429); `tests::a_later_section_reads_its_own_stories` (markdown/view_stories.rs:1468); `tests::what_does_not_read_is_left_out` (markdown/view_stories.rs:1479); `Writer::metadata` (markdown/write.rs:4075)

#### `runs`  (L612-687)
`fn runs(text: &str, marks: &mut Marks<'_>, drawings: &HashMap<String, String>) -> String`

- **Does:** A story's text as paragraph content: a field printed as the body prints it (`{PAGE \\* roman}i{/}`) becomes a simple field with that code and result, and a bare `{PAGE}` or other `{NAME}` one with the result `1`; a line break is a `w:br`.
- **Calls (crate):** `at` (markdown/from_docx/drawing.rs:1134) L629; `flush` (markdown/view_stories.rs:755) L630,638,667,685; `change` (markdown/view_stories.rs:696) L637; `tags_after` (markdown/view_stories.rs:713) L639; `tracked` (markdown/view_stories.rs:814) L649,659,660; `substitution_ids` (markdown/view_stories.rs:801) L655; `field` (markdown/view_stories.rs:733) L666; `escape` (markdown/xml.rs:426) L671; `runs` (markdown/view_stories.rs:612) L672
- **Calls (std/external/unresolved):** `.as_slice`, `.chars`, `.find`, `.first`, `.get`, `.len_utf8`, `.map_or`, `.next`, `.push_str`, `.push`, `.strip_prefix`, `.trim`, `Change::Insert`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `runs` (markdown/view_stories.rs:612); `story_part` (markdown/view_stories.rs:1118); `tests::written` (markdown/view_stories.rs:1490)

#### `change`  (L696-709)
`fn change(text: &str) -> Option<(Change<'_>, &str)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.split_once`, `.strip_prefix`, `Change::Delete`, `Change::Insert`, `Change::Substitute`, `Some`
- **Called by:** `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (convert/mod.rs:52938) (?); `track_properties` (edit/tracked.rs:633) (?); `runs` (markdown/view_stories.rs:612); `with_style_changes` (markdown/view_styles.rs:40) (?)

#### `tags_after`  (L713-723)
`fn tags_after(mut text: &str) -> (Vec<super::view_notes::Tag>, &str)`

- **Does:** The revision tags right after a change (`{>>r5 @AC<<}{>>r6 @JD<<}`), and what follows them.
- **Calls (crate):** `revision_tags` (markdown/view_notes.rs:363) L717
- **Calls (std/external/unresolved):** `.extend`, `.find`, `.strip_prefix`, `Vec::new`
- **Called by:** `runs` (markdown/view_stories.rs:612)

#### `field`  (L733-751)
`fn field(text: &str) -> Option<(Field<'_>, &str)>`

- **Does:** _no doc comment_
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1442) L735; `is_field_code` (markdown/critic.rs:1426) L736; `result_end` (markdown/view_stories.rs:913) L737
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.char_indices`, `.find`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.len`, `.strip_prefix`, `.then`, `Field::Bare`, `Field::Coded`, `Some`
- **Called by:** `runs` (markdown/view_stories.rs:612); `tracked` (markdown/view_stories.rs:814); `tracked_piece` (markdown/view_stories.rs:850)

#### `flush`  (L755-770)
`fn flush(plain: &mut String, out: &mut String, deleted: bool)`

- **Does:** Plain text as runs, a line break as a `w:br`; deleted text as `w:delText`.
- **Calls (crate):** `unescape` (markdown/view_stories.rs:774) L757; `escape` (markdown/xml.rs:426) L765
- **Calls (std/external/unresolved):** `.clear`, `.enumerate`, `.is_empty`, `.push_str`, `.split`
- **Macros:** `format!`
- **Called by:** `pair_gaps` (comparer/cross_para.rs:1204) (?); `resolve_paragraph_window` (comparer/lcs.rs:9735) (?); `count_words_and_tokenize` (comparer/moves.rs:87) (?); `revision_clusters` (comparer/textbox.rs:83) (?); `hf_tab_line_breaks` (convert/mod.rs:22114) (?); `Layout::emit_inline_pictures` (convert/mod.rs:27675) (?); `small_caps_pieces` (convert/mod.rs:32382) (?); `parse_table` (convert/word_subst.rs:23) (?); `align` (debug/diff.rs:1406) (?); `table_row_change` (markdown/diff.rs:255) (?); `first_section` (markdown/view_stories.rs:510); `runs` (markdown/view_stories.rs:612); +2 more

#### `unescape`  (L774-794)
`fn unescape(text: &str) -> String`

- **Does:** Document text the view escaped back as it was: a mark's delimiter (`{\++`, `~\>`) whole again.
- **Calls (std/external/unresolved):** `.contains`, `.fold`, `.iter`, `.replace`, `.to_string`
- **Called by:** `flush` (markdown/view_stories.rs:755)

#### `substitution_ids`  (L801-808)
`fn substitution_ids(ids: &[u32]) -> (&[u32], &[u32])`

- **Does:** A substitution's one tag (`r94+95+92+93`) lists the deleted side's ids, then the inserted side's: Word numbers marks in document order and writes the insertion first, so the inserted side begins where the numbers drop.
- **Calls (std/external/unresolved):** `.len`, `.map_or`, `.min`, `.position`, `.split_at`, `.windows`
- **Called by:** `runs` (markdown/view_stories.rs:612)

#### `tracked`  (L814-846)
`fn tracked( out: &mut String, marks: &mut Marks<'_>, name: &str, tag: Option<&super::view_notes::Tag>, ids: &[u32], inner: &str, drawings: &HashMap<String, String>, )`

- **Does:** One change's marks: a side whose tag names several ids (`r92+93`) is a mark per id, its text split as the body splits a run (after its last space, else before its last character); a side holding a field stays one mark.
- **Calls (crate):** `field` (markdown/view_stories.rs:733) L825; `split_point` (markdown/xml.rs:1679) L831; `cuts_token` (markdown/from_docx/drawing.rs:1162) L834; `tracked_piece` (markdown/view_stories.rs:850) L844
- **Calls (std/external/unresolved):** `.any`, `.char_indices`, `.copied`, `.ends_with`, `.enumerate`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.map`, `.rev`, `.splice`, `.split_at`
- **Macros:** `vec!`
- **Called by:** `runs` (markdown/view_stories.rs:612)

#### `tracked_piece`  (L850-906)
`fn tracked_piece( out: &mut String, marks: &mut Marks<'_>, name: &str, tag: Option<&super::view_notes::Tag>, id: Option<u32>, inner: &str, drawings: &HashMap<String, String>, )`

- **Does:** `inner` inside a `w:ins` or `w:del`: its text and its fields, which are complex there (a `w:fldSimple` cannot sit in a mark).
- **Calls (crate):** `Marks::mark_id` (markdown/view_stories.rs:152) (by name) L860; `at` (markdown/from_docx/drawing.rs:1134) L869; `flush` (markdown/view_stories.rs:755) L870,878,896,904; `field` (markdown/view_stories.rs:733) L877; `escape` (markdown/xml.rs:426) L893
- **Calls (std/external/unresolved):** `.chars`, `.get`, `.len_utf8`, `.next`, `.push_str`, `.push`, `.strip_prefix`, `.to_string`, `.trim`, `String::new`
- **Macros:** `write!`
- **Called by:** `tracked` (markdown/view_stories.rs:814)

#### `result_end`  (L913-938)
`fn result_end(text: &str) -> Option<usize>`

- **Does:** Where the `{/}` that closes a field's result sits in `text`, the text after its `{CODE}`: past any field nested in the result.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1442) L928; `is_field_code` (markdown/critic.rs:1426) L929
- **Calls (std/external/unresolved):** `.char_indices`, `.chars`, `.len`, `.map_or`, `.next`, `.starts_with`, `.strip_prefix`, `Some`
- **Called by:** `field` (markdown/view_stories.rs:733)

#### `SectionStories::apply`  (L953-987, pub(super))
`pub(super) fn apply(&self, number: usize, section: &str) -> String`

- **Does:** `section` (a `w:sectPr`) for section `number`, with the references and title page the view names in place of its own.
- **Calls (crate):** `without_empty_elements` (markdown/view_stories.rs:991) L959
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.filter_map`, `.find`, `.get`, `.insert_str`, `.iter`, `.map`, `.min`, `.to_string`, `.unwrap_or_default`
- **Called by:** `paragraph_properties` (markdown/xml.rs:1699)

#### `without_empty_elements`  (L991-1007)
`fn without_empty_elements(xml: &str, name: &str) -> String`

- **Does:** `xml` without its empty `<name …/>` elements.
- **Calls (std/external/unresolved):** `.ends_with`, `.find`, `.len`, `.push_str`, `.starts_with`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `SectionStories::apply` (markdown/view_stories.rs:953)

#### `write`  (L1013-1093, pub(super))
`pub(super) fn write( package: &mut PartFs, main: &str, sections: &BTreeMap<usize, Stories>, count: usize, people: &People, first_id: u32, default_style: &str, ) -> SectionStories`

- **Does:** Writes the headers and footers `sections` name, by section number from one, into `package` beside `main`: one part each, named by its id (`header2.xml`), in place of any the reference document had.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L1028; `PartFs::remove_related_part` (opc/mod.rs:694) L1039; `max_drawing_id` (markdown/package.rs:1232) L1047; `story_part` (markdown/view_stories.rs:1118) L1064; `reference_rank` (markdown/view_stories.rs:1098) L1077; `even_and_odd_headers` (markdown/view_stories.rs:1343) L1087
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.map`, `.max`, `.push`, `.sort_by_key`, `.unwrap_or_default`, `.unwrap_or`, `.values`, `HashMap::new`, `SectionStories::default`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::revised` (markdown/from_docx/mod.rs:2456) (?); `assemble` (markdown/package.rs:97)

#### `reference_rank`  (L1098-1107)
`fn reference_rank(kind: &str, ty: &str) -> usize`

- **Does:** Where Word writes a `kind` reference of type `ty` in a `w:sectPr`: even and default headers, even and default footers, then the first header and footer (every section Word wrote in the bench corpus).
- **Calls:** none
- **Called by:** `write` (markdown/view_stories.rs:1013)

#### `story_part`  (L1118-1339)
`fn story_part( package: &mut PartFs, main: &str, story: &Story, default_style: &str, marks: &mut Marks<'_>, next_doc_pr: &mut u32, ) -> String`

- **Does:** Writes `story`'s part and relates it to `main`: its relationship id.
- **Calls (crate):** `story_id` (markdown/view_stories.rs:365) L1140,1234; `revision_tags` (markdown/view_notes.rs:363) L1141,1163,1170,1238,1244; `sibling` (markdown/package.rs:345) L1181; `PartFs::parts` (opc/mod.rs:532) L1184; `at` (markdown/from_docx/drawing.rs:1134) L1199; `lookup` (markdown/from_docx/drawing.rs:1234) L1206; `materialize` (markdown/from_docx/drawing.rs:1371) L1213; `Drawing::wraps_run` (markdown/from_docx/drawing.rs:1308) (by name) L1217; `escape` (markdown/xml.rs:426) L1260; `xml` (markdown/borders.rs:254) L1268; `.mark` (ambiguous: `Marks::mark` (markdown/view_stories.rs:144); `NsScope::mark` (xmllinq/parse.rs:158)) L1286,1299,1314; `runs` (markdown/view_stories.rs:612) L1302; `PartFs::set_part` (opc/mod.rs:510) L1319; `PartFs::add_content_type_override` (opc/mod.rs:616) L1327; `PartFs::add_document_relationship` (opc/mod.rs:772) L1334; `relative_rel_target` (opc/mod.rs:129) L1337
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.as_slice`, `.as_str`, `.chain`, `.chars`, `.checked_sub`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.entry`, `.enumerate`, `.extend`, `.filter`, `.find_map`, `.find`, `.flatten`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_empty`, `.iter`, `.keys`, `.len_utf8`, `.len`, `.map_or`, `.map`, `.max`, `.next`, `.or_default`, `.push_str`, `.push`, `.remove`, `.replace`, `.resize`, `.rsplit_once`, `.sort_by_key`, `.split_once`, `.strip_prefix`, `.then_some`, `.to_string`, `.unwrap_or_default`, `HashMap::new`, `HashSet::from`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`, `vec!`, `write!`
- **Called by:** `write` (markdown/view_stories.rs:1013)

#### `even_and_odd_headers`  (L1343-1363)
`fn even_and_odd_headers(package: &mut PartFs, main: &str)`

- **Does:** Turns on Word's even and odd headers in the settings part `main` relates to.
- **Calls (crate):** `related` (markdown/package.rs:353) L1344; `PartFs::part_string` (opc/mod.rs:498) L1347; `PartFs::set_part` (opc/mod.rs:510) L1361
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.filter_map`, `.find`, `.insert_str`, `.into_bytes`, `.iter`, `.min`
- **Called by:** `write` (markdown/view_stories.rs:1013)

## Test functions (7)

- `tests::an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes` (L1429): _no doc comment_
- `tests::the_first_sections_blocks_read` (L1447): _no doc comment_
- `tests::a_later_section_reads_its_own_stories` (L1468): _no doc comment_
- `tests::what_does_not_read_is_left_out` (L1479): _no doc comment_
- `tests::written` (L1490): `text` as runs, with Ann Counsel as `@AC`.
- `tests::fields_and_changes_become_runs_and_marks` (L1514): _no doc comment_
- `tests::a_field_with_its_result_keeps_its_code_and_result` (L1558): _no doc comment_
