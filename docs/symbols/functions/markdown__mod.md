# Functions and methods in `src/markdown/mod.rs`

25 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `TrackChanges::parse`  (L91-98, pub)
`pub fn parse(value: &str) -> Option<Self>`

- **Does:** Parses `"all"`, `"accept"` or `"reject"`, the values of pandoc's `--track-changes`, and the `"accepted"` and `"rejected"` the CLI takes for them too.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `docx_options_from_json` (markdown/mod.rs:264); `tests::track_changes_takes_pandocs_values` (markdown/mod.rs:1157)

#### `PageSize::parse`  (L121-127, pub)
`pub fn parse(value: &str) -> Option<Self>`

- **Does:** Parses `"letter"` or `"a4"`, the values of the CLI's `--page`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `PageSize::as_str`  (L130-135, pub)
`pub fn as_str(self) -> &'static str`

- **Does:** Returns the name [`Self::parse`] reads (`"letter"` or `"a4"`).
- **Calls:** none
- **Called by:** none found in src

#### `resolve_critic`  (L150-156, pub)
`pub fn resolve_critic(markdown: &str, track_changes: TrackChanges) -> String`

- **Does:** Resolves the CriticMarkup in `markdown` as `track_changes` says: `Accept` keeps insertions and drops deletions, `Reject` the other way round; both keep highlighted text (`{==…==}`) and drop comments (`{>>…<<}`).
- **Calls (crate):** `resolve` (markdown/critic.rs:1972) L153,154
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `tests::round_trip` (markdown/diff.rs:1580); `tests::resolve_critic_all_is_the_markdown_as_it_is` (markdown/mod.rs:1207)

#### `accepted_clauses`  (L159-161, pub(crate))
`pub(crate) fn accepted_clauses(markdown: &str) -> String`

- **Does:** Accepted text snapshots remove wholly deleted clause lines.
- **Calls (crate):** `accept_clauses` (markdown/critic.rs:1987) L160
- **Called by:** `accepted_text` (text_diff.rs:137)

#### `<DocxOptions as Default>::default`  (L221-235, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `tests::source` (edit/inline.rs:378); `document_to_docx` (legacy_doc.rs:242); `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (markdown/from_docx/critic.rs:2219); `docx_options_from_json` (markdown/mod.rs:264); `tests::errors_and_options_describe_themselves` (markdown/mod.rs:1213); `markdown_authored_boundary_tests::front_matter_quote_variants_preserve_decoded_authored_metadata` (markdown/write.rs:6836); `markdown_authored_boundary_tests::comment_links_and_non_attribution_suffixes_keep_the_complete_comment` (markdown/write.rs:6867); `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (markdown/write.rs:6972); `tests::word` (text_diff.rs:763)

#### `docx_options_from_json`  (L264-288, pub)
`pub fn docx_options_from_json<'a>( json: &str, reference: Option<&'a [u8]>, ) -> Result<DocxOptions<'a>, String>`

- **Does:** [`DocxOptions`] from the JSON the Python and WebAssembly bindings pass: `page`, `author`, `date`, `critic`, `track_changes` (`trackChanges`), `tracking` and `scholar`, each optional.
- **Calls (crate):** `TrackChanges::parse` (markdown/mod.rs:91) L271; `<DocxOptions as Default>::default` (markdown/mod.rs:221) L274
- **Calls (std/external/unresolved):** `.as_deref`, `.is_some`, `.map_err`, `.ok_or_else`, `.to_string`, `.unwrap_or`, `Ok`, `serde_json::from_str`
- **Macros:** `format!`
- **Called by:** `tests::the_bindings_options_read_from_json` (markdown/mod.rs:1166)

#### `<DocxOptions as std::fmt::Debug>::fmt`  (L291-303, impl std::fmt::Debug)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.debug_struct`, `.field`, `.finish`, `.is_some`, `.map`
- **Called by:** none found in src

#### `<MarkdownOptions as Default>::default`  (L347-360, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `<InputLimits as Default>::default` (admission.rs:64) L349
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `read` (markdown/mod.rs:767); `changed_view` (markdown/mod.rs:1016); `patch_redline` (markdown/unified.rs:287); `patch_documents` (markdown/unified.rs:337)

#### `author`  (L404-412, pub)
`pub fn author(by: &str) -> Result<&str, String>`

- **Does:** Validates the author `--changed --by` names: a handle such as `AC` or `@AC` (one `@`, the handle right after it), or the full name.
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.starts_with`, `.strip_prefix`, `.trim`, `.unwrap_or`, `Err`, `Ok`
- **Called by:** `author_filter` (cli/edit_template.rs:130); `parse_author` (cli.rs:2793)

#### `Select::from_flags`  (L429-452, pub)
`pub fn from_flags( paragraphs: Option<&str>, head: Option<usize>, tail: Option<usize>, changed: bool, by: Option<&str>, ) -> Result<Option<Self>, String>`

- **Does:** Selects the right [`Select`] variant from `read`'s CLI flags: `-p` (`paragraphs`), `--head`, `--tail`, `--changed` with `--by`, in that order of precedence.
- **Calls (crate):** `Select::parse` (markdown/mod.rs:461) L447
- **Calls (std/external/unresolved):** `.is_some`, `.map`, `.to_string`, `.transpose`, `Err`, `Ok`, `Self::Head`, `Self::Tail`, `Some`
- **Called by:** `selection` (bin/jubarte.rs:1109)

#### `Select::parse`  (L461-567, pub)
`pub fn parse(spec: &str) -> Result<Self, String>`

- **Does:** `p2, p5-p7, p17-, -p1, t0`: comma-separated picks.
- **Calls (crate):** `refused` (ambiguous, 4 candidates) L474,475,495,541,545,560; `hint_where` (ids.rs:276) L477,509; `tests::body` (ids.rs:508) L493,554; `settle` (ambiguous: `settle` (edit/hint.rs:301); `settle` (edit.rs:1617)) L530; `tests::end` (markdown/from_docx/mod.rs:5744) (by name) L540
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.is_none`, `.is_some_and`, `.map_or_else`, `.map`, `.parse`, `.push`, `.split_once`, `.split`, `.to_string`, `.trim`, `.unwrap_or`, `Err`, `Ok`, `Pick::Table`, `Self::Picks`, `Some`, `Vec::new`, `settled_range`, `spell`
- **Macros:** `format!`, `matches!`
- **Called by:** `Select::from_flags` (markdown/mod.rs:429)

#### `docx_stamp`  (L580-582, pub)
`pub fn docx_stamp(docx: &[u8]) -> (Option<String>, Option<String>)`

- **Does:** The author and timestamp a `.docx` gives the changes and comments that name none, when no parameter does: its first `dc:creator` and its `dcterms:created`.
- **Calls (crate):** `creator_created` (markdown/from_docx/mod.rs:1386) L581
- **Called by:** none found in src

#### `markdown_stamp`  (L598-605, pub)
`pub fn markdown_stamp( markdown: &str, reference: Option<&[u8]>, ) -> (Option<String>, Option<String>)`

- **Does:** The author and timestamp a Markdown document gives the changes and comments that name none, when no parameter does: its front matter (`author:`; `last_modified:`, else `modified:`, `date:`, `created:`), else the reference `.docx`'s creator and creation time ([`docx_stamp`]), each on its own.
- **Calls (crate):** `file_defaults` (markdown/view_notes.rs:152) L602
- **Calls (std/external/unresolved):** `.map`, `.or`, `.unwrap_or_default`
- **Called by:** none found in src

#### `docx_to_markdown`  (L651-688, pub)
`pub fn docx_to_markdown(docx: &[u8], options: &MarkdownOptions) -> Result<ReadDocx, MarkdownError>`

- **Does:** Reads a `.docx` as Markdown: headings, emphasis, links, lists, tables, footnotes and equations (as LaTeX), with tracked changes and comments as CriticMarkup (`{++new++}`, `{--old--}`, `{~~old~>new~~}`, `{==text==}{>>note<<}`), each change followed by its author and date.
- **Calls (crate):** `sniff` (admission.rs:259) L654; `PartFs::open_within` (opc/mod.rs:593) L659; `refusal_message` (opc/mod.rs:88) L660; `convert` (markdown/from_docx/mod.rs:96) L666
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.into_iter`, `.map_err`, `.to_string`, `MarkdownError::Docx`, `Ok`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `read` (markdown/mod.rs:767); `changed_view` (markdown/mod.rs:1016); `patch_documents` (markdown/unified.rs:337); `patch_word` (markdown/unified.rs:375)

#### `<ReadOptions as Default>::default`  (L713-722, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `snapshot_options` (bin/jubarte.rs:1096)

#### `read`  (L767-808, pub)
`pub fn read(docx: &[u8], options: &ReadOptions) -> Result<ReadView, MarkdownError>`

- **Does:** Reads a `.docx` as the agent view every surface prints for `read` (alias `text`): a YAML header, then Markdown with an `<!-- pN -->` id line before every paragraph, tracked changes and comments with their ids, and `<!-- page N of M -->` markers from the layout pass.
- **Calls (crate):** `page_texts` (markdown/mod.rs:1067) L770; `docx_to_markdown` (markdown/mod.rs:651) L787; `<MarkdownOptions as Default>::default` (markdown/mod.rs:347) L799
- **Calls (std/external/unresolved):** `.clone`, `.map_err`, `.ok`, `.push`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Ok`, `Vec::new`, `crate::convert::RevisionStyle::default`
- **Macros:** `format!`
- **Called by:** `agent_view` (bin/jubarte.rs:1140); `run_convert_any` (bin/jubarte.rs:3458); `run_debug` (bin/jubarte.rs:3779) (?); `run_validate` (bin/jubarte.rs:3812) (?); `border_clauses` (markdown/from_docx/agent.rs:2204); `read_snapshot` (markdown/mod.rs:821); `read_tables` (markdown/mod.rs:912)

#### `read_snapshot`  (L821-873, pub)
`pub fn read_snapshot( docx: &[u8], options: &ReadOptions, ) -> Result<crate::inspect::Snapshot, MarkdownError>`

- **Does:** The snapshot `read --json` prints: [`crate::inspect::snapshot`], kept to the body paragraphs `options.select` picks (and the changes and comments in them, and under `--changed` those of the stories it changed) when it picks some.
- **Calls (crate):** `snapshot` (inspect.rs:403) L828; `settled_snapshot` (inspect.rs:442) L829,830; `read` (markdown/mod.rs:767) L834,851; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50395); `named` (markdown/write.rs:6250)) L868; `Snapshot::retain_with` (inspect.rs:189) (by name) L870
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.is_none_or`, `.is_some`, `.iter`, `.map_err`, `.retain`, `.to_string`, `.unwrap_or_default`, `MarkdownError::Docx`, `Ok`, `Pick::Table`, `Select::Picks`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `write_output` (bin/jubarte.rs:353); `read_json` (bin/jubarte.rs:3082)

#### `<TablesError as std::fmt::Display>::fmt`  (L887-892, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fmt`, `.write_str`
- **Called by:** none found in src

#### `read_tables`  (L912-994, pub)
`pub fn read_tables(docx: &[u8], options: &ReadOptions) -> Result<ReadView, TablesError>`

- **Does:** What `read --tables` prints ([`crate::inspect::tables_text`]): every body table as a grid, under `options.track_changes` `Reject` with every change rejected (`Accept` and `All` read the accepted text).
- **Calls (crate):** `view_tables` (inspect.rs:465) L917; `tables_text` (inspect.rs:554) L922,991; `read` (markdown/mod.rs:767) L935; `read_tables::{fn own}` (markdown/mod.rs:938) L958
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.insert`, `.into_iter`, `.map_err`, `.map`, `.range`, `.then_some`, `.to_string`, `.unwrap_or`, `Err`, `MarkdownError::Docx`, `Ok`, `TablesError::Read`, `TablesError::Usage`, `Vec::new`, `std::collections::BTreeMap::new`, `std::collections::BTreeSet::new`
- **Macros:** `format!`
- **Called by:** `run_read_tables` (bin/jubarte.rs:925)

#### `read_tables::{fn own}`  (L938-955)
`fn own( rows: &[Vec<crate::inspect::TableCell>], nested: &[crate::inspect::NestedTable], number: usize, owner: &mut std::collections::BTreeMap<usize, usize>, )`

- **Does:** The view's table that holds each paragraph (`pN` of a cell, or of a table nested in one, at any depth). _(from a code comment)_
- **Calls (crate):** `paragraph_place` (ids.rs:417) L945; `read_tables::{fn own}` (markdown/mod.rs:938) L953
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.flat_map`, `.flatten`, `.insert`, `.iter`, `.map`
- **Called by:** `revision_wrapper` (comparer/finalize.rs:6712) (?); `inherited_ref_id` (convert/mod.rs:20480) (?); `read_tables` (markdown/mod.rs:912); `read_tables::{fn own}` (markdown/mod.rs:938)

#### `changed_view`  (L1016-1043, pub)
`pub fn changed_view( docx: &[u8], author: &str, accepted: bool, source: Option<&str>, ) -> Result<String, MarkdownError>`

- **Does:** The view `jubarte edit` and `jubarte add` print after applying: the blocks of `docx` (the redline, or with `accepted` its accepted text with `rev` tags on the id lines) that carry `author`'s marks, as the agent view without page markers.
- **Calls (crate):** `<MarkdownOptions as Default>::default` (markdown/mod.rs:347) L1032; `docx_to_markdown` (markdown/mod.rs:651) L1034,1040
- **Calls (std/external/unresolved):** `.is_empty`, `.is_none_or`, `.map`, `.nth`, `.splitn`, `.to_string`, `.trim`, `Ok`, `Some`, `options`
- **Called by:** none found in src

#### `page_texts`  (L1067-1093, pub)
`pub fn page_texts( docx: &[u8], track_changes: TrackChanges, revisions: crate::convert::RevisionStyle, ) -> Result<Vec<String>, String>`

- **Does:** The text the layout pass paints on each page, with the document's changes kept (`TrackChanges::All`), accepted (`TrackChanges::Accept`) or rejected (`TrackChanges::Reject`).
- **Calls (crate):** `accept_revisions` (document_comparer.rs:4962) L1074; `reject_revisions` (document_comparer.rs:4985) L1077; `render` (convert/mod.rs:592) L1082
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.map_err`, `.map`, `Ok`, `crate::convert::PdfOptions::default`, `crate::convert::RenderRequest::default`, `std::borrow::Cow::Borrowed`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `read` (markdown/mod.rs:767); `with_page_markers` (markdown/mod.rs:1101)

#### `with_page_markers`  (L1101-1114, pub)
`pub fn with_page_markers( docx: &[u8], markdown: &str, track_changes: TrackChanges, revisions: crate::convert::RevisionStyle, ) -> (String, Option<String>)`

- **Does:** `markdown` (read from `docx`) with `<!-- page N of M -->` lines, as `convert --to md` writes it: `docx` is laid out as its PDF would be, with its changes kept, accepted or rejected as the Markdown has them, and each block found on its page.
- **Calls (crate):** `page_texts` (markdown/mod.rs:1067) L1107; `paginate` (markdown/pages.rs:59) L1110
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `paginated` (bin/jubarte.rs:3444)

#### `<MarkdownError as std::fmt::Display>::fmt`  (L1140-1146, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

## Test functions (4)

- `tests::track_changes_takes_pandocs_values` (L1157): _no doc comment_
- `tests::the_bindings_options_read_from_json` (L1166): _no doc comment_
- `tests::resolve_critic_all_is_the_markdown_as_it_is` (L1207): _no doc comment_
- `tests::errors_and_options_describe_themselves` (L1213): _no doc comment_
