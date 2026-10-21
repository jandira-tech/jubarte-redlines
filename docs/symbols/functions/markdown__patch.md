# Functions and methods in `src/markdown/patch.rs`

14 non-test, 9 test. Part of the [symbol manifest](../README.md).

#### `Block::text`  (L78-80)
`fn text(&self) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `rewritten` (markdown/patch.rs:491) (?); `blocks` (markdown/unified.rs:1059) (?)

#### `apply_markdown`  (L121-236, pub)
`pub fn apply_markdown(docx: &[u8], markdown: &str) -> Result<Patched, MarkdownError>`

- **Does:** Edits `docx` so its body reads as `markdown`, keeping the paragraphs' own runs, formatting, fields and properties, and reporting what the document could not take in [`Patched::warnings`].
- **Calls (crate):** `summary` (inspect.rs:339) L123; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:458); `invalid` (markdown/from_docx/ooxml.rs:31)) L123,126,131; `accept_revisions` (document_comparer.rs:4962) L126; `paragraphs` (inspect.rs:312) L130; `blocks` (markdown/patch.rs:662) L145; `align` (markdown/patch.rs:399) L150; `paragraph_styles` (markdown/patch.rs:239) L151; `.text` (ambiguous, 10 candidates) L158; `operations` (markdown/patch.rs:502) L168; `apply_plan` (edit.rs:2322) L188; `describe` (markdown/patch.rs:290) L226
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.chain`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.nth`, `.ok`, `.parse`, `.push`, `.rev`, `.split`, `.to_string`, `.to_vec`, `.trim`, `.unwrap_or_else`, `Default::default`, `Edit::Pair`, `Err`, `HashSet::new`, `Kind::Heading`, `MarkdownError::Package`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `redline` (markdown/redline.rs:109)

#### `paragraph_styles`  (L239-275)
`fn paragraph_styles(docx: &[u8]) -> HashSet<String>`

- **Does:** The paragraph styles a document defines, by lowercase name.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L240; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L243; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L246; `PartFs::resolve_rel_target` (opc/mod.rs:829) (by name) L251; `PartFs::part_string` (opc/mod.rs:636) (by name) L253; `Dom::new` (xmllinq/mod.rs:387) L257; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L258; `Dom::root` (xmllinq/mod.rs:822) L259; `Dom::elements` (xmllinq/mod.rs:640) L263; `Dom::attribute` (xmllinq/mod.rs:874) L267,272; `Dom::element` (xmllinq/mod.rs:655) L271
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.is_none_or`, `.iter`, `.to_lowercase`, `HashSet::new`, `Some`, `W::name`, `W::val`
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `describe`  (L290-312)
`fn describe(edit: &Edit, paragraphs: &[Paragraph], blocks: &[Block]) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `excerpt` (edit.rs:7161) L303,305,309; `.text` (ambiguous, 10 candidates) L303,305
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.len`, `.take`
- **Macros:** `format!`
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `key`  (L316-320)
`fn key(text: &str) -> String`

- **Does:** Text compared for alignment: whitespace collapsed, symbols and leading typed numbering left out.
- **Calls (crate):** `collapse_whitespace` (util/mod.rs:46) L318; `enumerator_len` (markdown/patch.rs:324) L319
- **Calls (std/external/unresolved):** `.replace`, `.to_string`
- **Called by:** `earliest` (changes/logical.rs:383) (?); `fold_twins` (changes.rs:1280) (?); `step_h` (comparer/lcs.rs:3621) (?); `has_common_run_ge` (comparer/lcs.rs:5736) (?); `group_by_key_stable` (comparer/produce.rs:692) (?); `align` (markdown/patch.rs:399); `operations` (markdown/patch.rs:502); `tests::keys_ignore_numbering_spacing_and_symbols` (markdown/patch.rs:857)

#### `enumerator_len`  (L324-384)
`fn enumerator_len(text: &str) -> usize`

- **Does:** The length of typed numbering at the start of `text` with the whitespace after it: `1.`, `1.2`, `(a)`, `b)`, `iv.`, `A.`.
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.contains`, `.count`, `.first`, `.get`, `.is_ascii_alphabetic`, `.is_ascii_digit`, `.is_ascii_whitespace`, `.iter`, `.len`, `.take_while`, `Some`, `digits`
- **Called by:** `key` (markdown/patch.rs:316); `rewritten` (markdown/patch.rs:491); `tests::typed_numbering_is_found` (markdown/patch.rs:835); `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (markdown/patch.rs:1020)

#### `heading_level`  (L386-389)
`fn heading_level(style: &str) -> Option<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.ok`, `.parse`, `.strip_prefix`, `.then_some`
- **Called by:** none found in src

#### `compatible`  (L392-394)
`fn compatible(paragraph: &Paragraph, block: &Block) -> bool`

- **Does:** Whether a paragraph and a block can be the same: cells pair with cells.
- **Calls:** none
- **Called by:** `align` (markdown/patch.rs:399)

#### `align`  (L399-448)
`fn align(paragraphs: &[Paragraph], blocks: &[Block]) -> Vec<Edit>`

- **Does:** Aligns paragraphs and blocks: equal text first, then, in each stretch that differs, the most similar compatible pairs in order, then leftovers by position.
- **Calls (crate):** `key` (markdown/patch.rs:316) L400,401; `.text` (ambiguous, 10 candidates) L401; `pair` (markdown/patch.rs:451) L415; `compatible` (markdown/patch.rs:392) L416,427; `similarity` (markdown/diff.rs:859) L419
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.chain`, `.collect`, `.into_iter`, `.iter`, `.len`, `.map`, `.push`, `.then_some`, `.zip`, `Edit::Delete`, `Edit::Insert`, `Edit::Pair`, `Some`, `Vec::new`, `capture_diff_slices`, `std::iter::once`
- **Called by:** `field_tests::display_math_paragraph_takes_the_equation_alignment` (convert/mod.rs:37805) (?); `apply_markdown` (markdown/patch.rs:121); `markdown_pairing_boundary_tests::ordered_pairing_boundaries_keep_every_authored_index_once` (markdown/patch.rs:934)

#### `pair`  (L451-487)
`fn pair( olds: &[usize], news: &[usize], score: impl Fn(usize, usize) -> Option<f64>, ) -> Vec<(usize, usize)>`

- **Does:** Weighted longest common subsequence of `olds` and `news` under `score`.
- **Calls (std/external/unresolved):** `.abs`, `.collect`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.push`, `.rev`, `Vec::new`, `score`
- **Macros:** `vec!`
- **Called by:** `align` (markdown/patch.rs:399); `markdown_pairing_boundary_tests::weighted_pairing_skips_weaker_conflicts_without_crossing_source_owners` (markdown/patch.rs:993)

#### `rewritten`  (L491-499)
`fn rewritten(paragraph: &Paragraph, block: &Block) -> String`

- **Does:** The paragraph's new text: the block's, after the paragraph's own typed numbering when it has some.
- **Calls (crate):** `.text` (ambiguous: `Block::text` (markdown/patch.rs:78); `Block::text` (markdown/unified.rs:1031)) L492; `enumerator_len` (markdown/patch.rs:324) L493,497
- **Macros:** `format!`
- **Called by:** `operations` (markdown/patch.rs:502); `tests::a_rewrite_keeps_the_paragraphs_own_numbering` (markdown/patch.rs:887); `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (markdown/patch.rs:1020)

#### `operations`  (L502-659)
`fn operations( paragraphs: &[Paragraph], blocks: &[Block], edits: &[Edit], styles: &HashSet<String>, whole: &HashSet<usize>, dropped: &HashSet<usize>, ) -> Vec<Operation>`

- **Does:** The plan's operations.
- **Calls (crate):** `nearest` (edit.rs:2233) L546,552,554,558; `at` (markdown/from_docx/drawing.rs:1360) (by name) L561,564,587,614,623,629; `key` (markdown/patch.rs:316) L579; `.text` (ambiguous, 10 candidates) L579; `rewritten` (markdown/patch.rs:491) L588; `insert` (markdown/data_stores.rs:366) (by name) L602,648
- **Calls (std/external/unresolved):** `.abs_diff`, `.and_then`, `.checked_sub`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.is_some`, `.iter`, `.len`, `.min_by_key`, `.or_else`, `.push`, `.rev`, `.then_some`, `.then`, `.unwrap_or`, `Kind::Heading`, `Some`, `String::new`, `Vec::new`, `fits`, `stays`
- **Macros:** `format!`, `matches!`
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `blocks`  (L662-809)
`fn blocks(markdown: &str) -> (Vec<Block>, bool)`

- **Does:** The Markdown's blocks in order, and whether it has footnotes.
- **Calls (crate):** `.text` (ambiguous, 10 candidates) L679; `close` (ambiguous, 9 candidates) L709,715,726,730,734,738...; `kind` (ambiguous: `tests::kind` (admission.rs:1443); `tests::kind` (edit/flags.rs:894); `tests::kind` (markdown/from_docx/critic.rs:1518)) L783; `push_run` (markdown/patch.rs:813) L786,796,801
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.get_or_insert_with`, `.is_empty`, `.iter`, `.push`, `.replace`, `.saturating_sub`, `.split`, `.starts_with`, `.take`, `.to_ascii_lowercase`, `.trim`, `Kind::Heading`, `Parser::new_ext`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `style_lines` (debug.rs:1532) (?); `apply_markdown` (markdown/patch.rs:121); `tests::blocks_have_kinds_and_runs` (markdown/patch.rs:863); `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (markdown/patch.rs:1020)

#### `push_run`  (L813-827)
`fn push_run(block: &mut Block, text: &str, bold: bool, italic: bool)`

- **Does:** Appends text to the block, joining the last run when its formatting is the same.
- **Calls (std/external/unresolved):** `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.then_some`, `.to_string`, `RunSpec::default`
- **Called by:** `rebuild_body_free_mesh_lcs` (comparer/finalize.rs:13851) (?); `blocks` (markdown/patch.rs:662)

## Test functions (9)

- `tests::typed_numbering_is_found` (L835): _no doc comment_
- `tests::keys_ignore_numbering_spacing_and_symbols` (L857): _no doc comment_
- `tests::blocks_have_kinds_and_runs` (L863): _no doc comment_
- `tests::a_rewrite_keeps_the_paragraphs_own_numbering` (L887): _no doc comment_
- `markdown_pairing_boundary_tests::block` (L912): _no doc comment_
- `markdown_pairing_boundary_tests::edit_view` (L922): _no doc comment_
- `markdown_pairing_boundary_tests::ordered_pairing_boundaries_keep_every_authored_index_once` (L934): _no doc comment_
- `markdown_pairing_boundary_tests::weighted_pairing_skips_weaker_conflicts_without_crossing_source_owners` (L993): _no doc comment_
- `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (L1020): _no doc comment_
