# Functions and methods in `src/markdown/mod.rs`

22 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `TrackChanges::parse`  (L87-94, pub)
`pub fn parse(value: &str) -> Option<Self>`

- **Does:** Parses `"all"`, `"accept"` or `"reject"`, the values of pandoc's `--track-changes`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `docx_options_from_json` (markdown/mod.rs:260); `tests::track_changes_takes_pandocs_values` (markdown/mod.rs:1007)

#### `PageSize::parse`  (L117-123, pub)
`pub fn parse(value: &str) -> Option<Self>`

- **Does:** Parses `"letter"` or `"a4"`, the values of the CLI's `--page`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `PageSize::as_str`  (L126-131, pub)
`pub fn as_str(self) -> &'static str`

- **Does:** Returns the name [`Self::parse`] reads (`"letter"` or `"a4"`).
- **Calls:** none
- **Called by:** none found in src

#### `resolve_critic`  (L146-152, pub)
`pub fn resolve_critic(markdown: &str, track_changes: TrackChanges) -> String`

- **Does:** Resolves the CriticMarkup in `markdown` as `track_changes` says: `Accept` keeps insertions and drops deletions, `Reject` the other way round; both keep highlighted text (`{==…==}`) and drop comments (`{>>…<<}`).
- **Calls (crate):** `resolve` (markdown/critic.rs:1744) L149,150
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `tests::round_trip` (markdown/diff.rs:1571); `tests::resolve_critic_all_is_the_markdown_as_it_is` (markdown/mod.rs:1057)

#### `accepted_clauses`  (L155-157, pub(crate))
`pub(crate) fn accepted_clauses(markdown: &str) -> String`

- **Does:** Accepted text snapshots remove wholly deleted clause lines.
- **Calls (crate):** `accept_clauses` (markdown/critic.rs:1758) L156
- **Called by:** `accepted_text` (text_diff.rs:137)

#### `<DocxOptions as Default>::default`  (L217-231, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `document_to_docx` (legacy_doc.rs:242); `docx_options_from_json` (markdown/mod.rs:260); `tests::errors_and_options_describe_themselves` (markdown/mod.rs:1063); `markdown_authored_boundary_tests::front_matter_quote_variants_preserve_decoded_authored_metadata` (markdown/write.rs:5956); `markdown_authored_boundary_tests::comment_links_and_non_attribution_suffixes_keep_the_complete_comment` (markdown/write.rs:5987); `memory_picture_import_contract_tests::rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures` (markdown/write.rs:6092); `tests::word` (text_diff.rs:763)

#### `docx_options_from_json`  (L260-284, pub)
`pub fn docx_options_from_json<'a>( json: &str, reference: Option<&'a [u8]>, ) -> Result<DocxOptions<'a>, String>`

- **Does:** [`DocxOptions`] from the JSON the Python and WebAssembly bindings pass: `page`, `author`, `date`, `critic`, `track_changes` (`trackChanges`), `tracking` and `scholar`, each optional.
- **Calls (crate):** `TrackChanges::parse` (markdown/mod.rs:87) L267; `<DocxOptions as Default>::default` (markdown/mod.rs:217) L270
- **Calls (std/external/unresolved):** `.as_deref`, `.is_some`, `.map_err`, `.ok_or_else`, `.to_string`, `.unwrap_or`, `Ok`, `serde_json::from_str`
- **Macros:** `format!`
- **Called by:** `tests::the_bindings_options_read_from_json` (markdown/mod.rs:1016)

#### `<DocxOptions as std::fmt::Debug>::fmt`  (L287-299, impl std::fmt::Debug)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.debug_struct`, `.field`, `.finish`, `.is_some`, `.map`
- **Called by:** none found in src

#### `<MarkdownOptions as Default>::default`  (L340-352, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `changed_view` (markdown/mod.rs:866); `patch_documents` (markdown/unified.rs:337); `patch_word` (markdown/unified.rs:371)

#### `author`  (L396-404, pub)
`pub fn author(by: &str) -> Result<&str, String>`

- **Does:** Validates the author `--changed --by` names: a handle such as `AC` or `@AC` (one `@`, the handle right after it), or the full name.
- **Calls (std/external/unresolved):** `.into`, `.is_empty`, `.starts_with`, `.strip_prefix`, `.trim`, `.unwrap_or`, `Err`, `Ok`
- **Called by:** `parse_author` (cli.rs:2368)

#### `Select::from_flags`  (L421-444, pub)
`pub fn from_flags( paragraphs: Option<&str>, head: Option<usize>, tail: Option<usize>, changed: bool, by: Option<&str>, ) -> Result<Option<Self>, String>`

- **Does:** Selects the right [`Select`] variant from `read`'s CLI flags: `-p` (`paragraphs`), `--head`, `--tail`, `--changed` with `--by`, in that order of precedence.
- **Calls (crate):** `Select::parse` (markdown/mod.rs:453) L439
- **Calls (std/external/unresolved):** `.is_some`, `.map`, `.to_string`, `.transpose`, `Err`, `Ok`, `Self::Head`, `Self::Tail`, `Some`
- **Called by:** `selection` (bin/jubarte.rs:832)

#### `Select::parse`  (L453-559, pub)
`pub fn parse(spec: &str) -> Result<Self, String>`

- **Does:** `p2, p5-p7, p17-, -p1, t0`: comma-separated picks.
- **Calls (crate):** `refused` (ambiguous: `refused` (bin/jubarte.rs:1943); `refused` (opc/mod.rs:76)) L466,467,487,533,537,552; `hint_where` (ids.rs:276) L469,501; `tests::body` (ids.rs:508) L485,546; `settle` (edit.rs:1260) (by name) L522; `tests::end` (markdown/from_docx/mod.rs:5384) (by name) L532
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.is_none`, `.is_some_and`, `.map_or_else`, `.map`, `.parse`, `.push`, `.split_once`, `.split`, `.to_string`, `.trim`, `.unwrap_or`, `Err`, `Ok`, `Pick::Table`, `Self::Picks`, `Some`, `Vec::new`, `settled_range`, `spell`
- **Macros:** `format!`, `matches!`
- **Called by:** `Select::from_flags` (markdown/mod.rs:421)

#### `docx_stamp`  (L572-574, pub)
`pub fn docx_stamp(docx: &[u8]) -> (Option<String>, Option<String>)`

- **Does:** The author and timestamp a `.docx` gives the changes and comments that name none, when no parameter does: its first `dc:creator` and its `dcterms:created`.
- **Calls (crate):** `creator_created` (markdown/from_docx/mod.rs:1302) L573
- **Called by:** none found in src

#### `markdown_stamp`  (L590-597, pub)
`pub fn markdown_stamp( markdown: &str, reference: Option<&[u8]>, ) -> (Option<String>, Option<String>)`

- **Does:** The author and timestamp a Markdown document gives the changes and comments that name none, when no parameter does: its front matter (`author:`; `last_modified:`, else `modified:`, `date:`, `created:`), else the reference `.docx`'s creator and creation time ([`docx_stamp`]), each on its own.
- **Calls (crate):** `file_defaults` (markdown/view_notes.rs:144) L594
- **Calls (std/external/unresolved):** `.map`, `.or`, `.unwrap_or_default`
- **Called by:** none found in src

#### `docx_to_markdown`  (L641-671, pub)
`pub fn docx_to_markdown(docx: &[u8], options: &MarkdownOptions) -> Result<ReadDocx, MarkdownError>`

- **Does:** Reads a `.docx` as Markdown: headings, emphasis, links, lists, tables, footnotes and equations (as LaTeX), with tracked changes and comments as CriticMarkup (`{++new++}`, `{--old--}`, `{~~old~>new~~}`, `{==text==}{>>note<<}`), each change followed by its author and date.
- **Calls (crate):** `sniff` (admission.rs:216) L644; `convert` (markdown/from_docx/mod.rs:96) L650
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.into_iter`, `.map_err`, `.to_string`, `MarkdownError::Docx`, `Ok`
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `read` (markdown/mod.rs:749); `changed_view` (markdown/mod.rs:866); `patch_documents` (markdown/unified.rs:337); `patch_word` (markdown/unified.rs:371)

#### `<ReadOptions as Default>::default`  (L695-704, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `write_output` (bin/jubarte.rs:278); `run_text` (bin/jubarte.rs:785)

#### `read`  (L749-789, pub)
`pub fn read(docx: &[u8], options: &ReadOptions) -> Result<ReadView, MarkdownError>`

- **Does:** Reads a `.docx` as the agent view every surface prints for `read` (alias `text`): a YAML header, then Markdown with an `<!-- pN -->` id line before every paragraph, tracked changes and comments with their ids, and `<!-- page N of M -->` markers from the layout pass.
- **Calls (crate):** `page_texts` (markdown/mod.rs:917) L752; `docx_to_markdown` (markdown/mod.rs:641) L769
- **Calls (std/external/unresolved):** `.clone`, `.map_err`, `.ok`, `.push`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Ok`, `Vec::new`, `crate::convert::RevisionStyle::default`
- **Macros:** `format!`
- **Called by:** `agent_view` (bin/jubarte.rs:844); `run_convert_any` (bin/jubarte.rs:2477); `run_debug` (bin/jubarte.rs:2746) (?); `run_validate` (bin/jubarte.rs:2793) (?); `cloud_folder_names_with` (convert/font.rs:2401) (?); `family_faces_in_with` (convert/font.rs:2473) (?); `scan_cjk_family_faces_with` (convert/font.rs:2881) (?); `endnote_num_format` (convert/mod.rs:8644) (?); `face_unicode_map` (convert/pdf.rs:1656) (?); `tests::symbol_font_text_reads_back_as_word_writes_it` (convert/pdf.rs:2283) (?); `Geometry::of` (edit/sections.rs:151) (?); `text_width` (edit/structural.rs:70) (?); +2 more

#### `read_snapshot`  (L798-844, pub)
`pub fn read_snapshot( docx: &[u8], options: &ReadOptions, ) -> Result<crate::inspect::Snapshot, MarkdownError>`

- **Does:** The snapshot `read --json` prints: [`crate::inspect::snapshot`], kept to the body paragraphs `options.select` picks (and the changes and comments in them, and under `--changed` those of the stories it changed) when it picks some.
- **Calls (crate):** `snapshot` (inspect.rs:362) L803; `read` (markdown/mod.rs:749) L805,822; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50359); `named` (markdown/write.rs:5387)) L839; `Snapshot::retain_with` (inspect.rs:161) (by name) L841
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.is_none_or`, `.is_some`, `.iter`, `.map_err`, `.retain`, `.to_string`, `.unwrap_or_default`, `MarkdownError::Docx`, `Ok`, `Pick::Table`, `Select::Picks`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `write_output` (bin/jubarte.rs:278); `run_text` (bin/jubarte.rs:785)

#### `changed_view`  (L866-893, pub)
`pub fn changed_view( docx: &[u8], author: &str, accepted: bool, source: Option<&str>, ) -> Result<String, MarkdownError>`

- **Does:** The view `jubarte edit` and `jubarte add` print after applying: the blocks of `docx` (the redline, or with `accepted` its accepted text with `rev` tags on the id lines) that carry `author`'s marks, as the agent view without page markers.
- **Calls (crate):** `<MarkdownOptions as Default>::default` (markdown/mod.rs:340) L882; `docx_to_markdown` (markdown/mod.rs:641) L884,890
- **Calls (std/external/unresolved):** `.is_empty`, `.is_none_or`, `.map`, `.nth`, `.splitn`, `.to_string`, `.trim`, `Ok`, `Some`, `options`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `resolved_or_view` (bin/jubarte.rs:1635)

#### `page_texts`  (L917-943, pub)
`pub fn page_texts( docx: &[u8], track_changes: TrackChanges, revisions: crate::convert::RevisionStyle, ) -> Result<Vec<String>, String>`

- **Does:** The text the layout pass paints on each page, with the document's changes kept (`TrackChanges::All`), accepted (`TrackChanges::Accept`) or rejected (`TrackChanges::Reject`).
- **Calls (crate):** `accept_revisions` (document_comparer.rs:4959) L924; `reject_revisions` (document_comparer.rs:4976) L927; `render` (convert/mod.rs:592) L932
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.map_err`, `.map`, `Ok`, `crate::convert::PdfOptions::default`, `crate::convert::RenderRequest::default`, `std::borrow::Cow::Borrowed`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `read` (markdown/mod.rs:749); `with_page_markers` (markdown/mod.rs:951)

#### `with_page_markers`  (L951-964, pub)
`pub fn with_page_markers( docx: &[u8], markdown: &str, track_changes: TrackChanges, revisions: crate::convert::RevisionStyle, ) -> (String, Option<String>)`

- **Does:** `markdown` (read from `docx`) with `<!-- page N of M -->` lines, as `convert --to md` writes it: `docx` is laid out as its PDF would be, with its changes kept, accepted or rejected as the Markdown has them, and each block found on its page.
- **Calls (crate):** `page_texts` (markdown/mod.rs:917) L957; `paginate` (markdown/pages.rs:59) L960
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `paginated` (bin/jubarte.rs:2463)

#### `<MarkdownError as std::fmt::Display>::fmt`  (L990-996, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

## Test functions (4)

- `tests::track_changes_takes_pandocs_values` (L1007): _no doc comment_
- `tests::the_bindings_options_read_from_json` (L1016): _no doc comment_
- `tests::resolve_critic_all_is_the_markdown_as_it_is` (L1057): _no doc comment_
- `tests::errors_and_options_describe_themselves` (L1063): _no doc comment_
