# Functions and methods in `src/markdown/patch.rs`

14 non-test, 9 test. Part of the [symbol manifest](../README.md).

#### `Block::text`  (L78-80)
`fn text(&self) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `rewritten` (markdown/patch.rs:485) (?); `blocks` (markdown/unified.rs:851) (?)

#### `apply_markdown`  (L121-234, pub)
`pub fn apply_markdown(docx: &[u8], markdown: &str) -> Result<Patched, MarkdownError>`

- **Does:** Edits `docx` so its body reads as `markdown`, keeping the paragraphs' own runs, formatting, fields and properties, and reporting what the document could not take in [`Patched::warnings`].
- **Calls (crate):** `summary` (inspect.rs:298) L123; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:447); `invalid` (markdown/from_docx/ooxml.rs:27)) L123,126,131; `accept_revisions` (document_comparer.rs:4959) L126; `paragraphs` (inspect.rs:284) L130; `blocks` (markdown/patch.rs:643) L145; `align` (markdown/patch.rs:393) L150; `paragraph_styles` (markdown/patch.rs:237) L151; `.text` (ambiguous, 10 candidates) L158; `operations` (markdown/patch.rs:496) L168; `apply_plan` (edit.rs:1621) L186; `describe` (markdown/patch.rs:284) L224
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.chain`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.nth`, `.ok`, `.parse`, `.push`, `.rev`, `.split`, `.to_string`, `.to_vec`, `.trim`, `.unwrap_or_else`, `Edit::Pair`, `Err`, `HashSet::new`, `Kind::Heading`, `MarkdownError::Package`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `redline` (markdown/redline.rs:108)

#### `paragraph_styles`  (L237-269)
`fn paragraph_styles(docx: &[u8]) -> HashSet<String>`

- **Does:** The paragraph styles a document defines, by lowercase name.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:456) L238; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L241; `PartFs::read_rels_for` (opc/mod.rs:756) (by name) L244; `PartFs::resolve_rel_target` (opc/mod.rs:600) (by name) L249; `PartFs::part_string` (opc/mod.rs:498) (by name) L251; `Dom::new` (xmllinq/mod.rs:387) L255; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L256; `Dom::root` (xmllinq/mod.rs:822) L257; `Dom::elements` (xmllinq/mod.rs:640) L261; `Dom::attribute` (xmllinq/mod.rs:874) L263,266; `Dom::element` (xmllinq/mod.rs:655) L265
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.iter`, `.to_lowercase`, `HashSet::new`, `Some`, `W::name`, `W::val`
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `describe`  (L284-306)
`fn describe(edit: &Edit, paragraphs: &[Paragraph], blocks: &[Block]) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `excerpt` (edit.rs:5435) L297,299,303; `.text` (ambiguous, 10 candidates) L297,299
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.len`, `.take`
- **Macros:** `format!`
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `key`  (L310-314)
`fn key(text: &str) -> String`

- **Does:** Text compared for alignment: whitespace collapsed, symbols and leading typed numbering left out.
- **Calls (crate):** `enumerator_len` (markdown/patch.rs:318) L313
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.replace`, `.split_whitespace`, `.to_string`
- **Called by:** `fold_twins` (changes.rs:568) (?); `step_h` (comparer/lcs.rs:3621) (?); `has_common_run_ge` (comparer/lcs.rs:5736) (?); `group_by_key_stable` (comparer/produce.rs:692) (?); `align` (markdown/patch.rs:393); `operations` (markdown/patch.rs:496); `tests::keys_ignore_numbering_spacing_and_symbols` (markdown/patch.rs:838)

#### `enumerator_len`  (L318-378)
`fn enumerator_len(text: &str) -> usize`

- **Does:** The length of typed numbering at the start of `text` with the whitespace after it: `1.`, `1.2`, `(a)`, `b)`, `iv.`, `A.`.
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.contains`, `.count`, `.first`, `.get`, `.is_ascii_alphabetic`, `.is_ascii_digit`, `.is_ascii_whitespace`, `.iter`, `.len`, `.take_while`, `Some`, `digits`
- **Called by:** `key` (markdown/patch.rs:310); `rewritten` (markdown/patch.rs:485); `tests::typed_numbering_is_found` (markdown/patch.rs:816); `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (markdown/patch.rs:1001)

#### `heading_level`  (L380-383)
`fn heading_level(style: &str) -> Option<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.ok`, `.parse`, `.strip_prefix`, `.then_some`
- **Called by:** none found in src

#### `compatible`  (L386-388)
`fn compatible(paragraph: &Paragraph, block: &Block) -> bool`

- **Does:** Whether a paragraph and a block can be the same: cells pair with cells.
- **Calls:** none
- **Called by:** `align` (markdown/patch.rs:393)

#### `align`  (L393-442)
`fn align(paragraphs: &[Paragraph], blocks: &[Block]) -> Vec<Edit>`

- **Does:** Aligns paragraphs and blocks: equal text first, then, in each stretch that differs, the most similar compatible pairs in order, then leftovers by position.
- **Calls (crate):** `key` (markdown/patch.rs:310) L394,395; `.text` (ambiguous, 10 candidates) L395; `pair` (markdown/patch.rs:445) L409; `compatible` (markdown/patch.rs:386) L410,421; `similarity` (markdown/diff.rs:850) L413
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.chain`, `.collect`, `.into_iter`, `.iter`, `.len`, `.map`, `.push`, `.then_some`, `.zip`, `Edit::Delete`, `Edit::Insert`, `Edit::Pair`, `Some`, `Vec::new`, `capture_diff_slices`, `std::iter::once`
- **Called by:** `field_tests::display_math_paragraph_takes_the_equation_alignment` (convert/mod.rs:37769) (?); `apply_markdown` (markdown/patch.rs:121); `markdown_pairing_boundary_tests::ordered_pairing_boundaries_keep_every_authored_index_once` (markdown/patch.rs:915)

#### `pair`  (L445-481)
`fn pair( olds: &[usize], news: &[usize], score: impl Fn(usize, usize) -> Option<f64>, ) -> Vec<(usize, usize)>`

- **Does:** Weighted longest common subsequence of `olds` and `news` under `score`.
- **Calls (std/external/unresolved):** `.abs`, `.collect`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.push`, `.rev`, `Vec::new`, `score`
- **Macros:** `vec!`
- **Called by:** `vml_line_image` (convert/mod.rs:18609) (?); `vml_group_box` (convert/mod.rs:19263) (?); `Fib::parse` (legacy_doc.rs:895) (?); `render_emphasis` (markdown/from_docx/ooxml.rs:705) (?); `align` (markdown/patch.rs:393); `markdown_pairing_boundary_tests::weighted_pairing_skips_weaker_conflicts_without_crossing_source_owners` (markdown/patch.rs:974)

#### `rewritten`  (L485-493)
`fn rewritten(paragraph: &Paragraph, block: &Block) -> String`

- **Does:** The paragraph's new text: the block's, after the paragraph's own typed numbering when it has some.
- **Calls (crate):** `.text` (ambiguous: `Block::text` (markdown/patch.rs:78); `Block::text` (markdown/unified.rs:823)) L486; `enumerator_len` (markdown/patch.rs:318) L487,491
- **Macros:** `format!`
- **Called by:** `operations` (markdown/patch.rs:496); `tests::a_rewrite_keeps_the_paragraphs_own_numbering` (markdown/patch.rs:868); `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (markdown/patch.rs:1001)

#### `operations`  (L496-640)
`fn operations( paragraphs: &[Paragraph], blocks: &[Block], edits: &[Edit], styles: &HashSet<String>, whole: &HashSet<usize>, dropped: &HashSet<usize>, ) -> Vec<Operation>`

- **Does:** The plan's operations.
- **Calls (crate):** `nearest` (ambiguous: `nearest` (capabilities/discovery.rs:161); `nearest` (debug/render.rs:375); `nearest` (inspect/tables.rs:43)) L540,546,548,552; `at` (markdown/from_docx/drawing.rs:1134) (by name) L555,558,580,604,613,618; `key` (markdown/patch.rs:310) L573; `.text` (ambiguous, 10 candidates) L573; `rewritten` (markdown/patch.rs:485) L581
- **Calls (std/external/unresolved):** `.abs_diff`, `.and_then`, `.checked_sub`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.is_some`, `.iter`, `.len`, `.min_by_key`, `.or_else`, `.push`, `.rev`, `.then_some`, `.then`, `.unwrap_or`, `Kind::Heading`, `Some`, `String::new`, `Vec::new`, `fits`, `insert`, `stays`
- **Macros:** `format!`, `matches!`
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `blocks`  (L643-790)
`fn blocks(markdown: &str) -> (Vec<Block>, bool)`

- **Does:** The Markdown's blocks in order, and whether it has footnotes.
- **Calls (crate):** `.text` (ambiguous, 10 candidates) L660; `close` (ambiguous, 9 candidates) L690,696,707,711,715,719...; `kind` (ambiguous: `tests::kind` (admission.rs:1043); `tests::kind` (edit/flags.rs:765); `tests::kind` (markdown/from_docx/critic.rs:1490)) L764; `push_run` (markdown/patch.rs:794) L767,777,782
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.get_or_insert_with`, `.is_empty`, `.iter`, `.push`, `.replace`, `.saturating_sub`, `.split`, `.starts_with`, `.take`, `.to_ascii_lowercase`, `.trim`, `Kind::Heading`, `Parser::new_ext`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `style_lines` (debug.rs:1362) (?); `apply_markdown` (markdown/patch.rs:121); `tests::blocks_have_kinds_and_runs` (markdown/patch.rs:844); `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (markdown/patch.rs:1001)

#### `push_run`  (L794-808)
`fn push_run(block: &mut Block, text: &str, bold: bool, italic: bool)`

- **Does:** Appends text to the block, joining the last run when its formatting is the same.
- **Calls (std/external/unresolved):** `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.then_some`, `.to_string`, `RunSpec::default`
- **Called by:** `rebuild_body_free_mesh_lcs` (comparer/finalize.rs:13829) (?); `blocks` (markdown/patch.rs:643)

## Test functions (9)

- `tests::typed_numbering_is_found` (L816): _no doc comment_
- `tests::keys_ignore_numbering_spacing_and_symbols` (L838): _no doc comment_
- `tests::blocks_have_kinds_and_runs` (L844): _no doc comment_
- `tests::a_rewrite_keeps_the_paragraphs_own_numbering` (L868): _no doc comment_
- `markdown_pairing_boundary_tests::block` (L893): _no doc comment_
- `markdown_pairing_boundary_tests::edit_view` (L903): _no doc comment_
- `markdown_pairing_boundary_tests::ordered_pairing_boundaries_keep_every_authored_index_once` (L915): _no doc comment_
- `markdown_pairing_boundary_tests::weighted_pairing_skips_weaker_conflicts_without_crossing_source_owners` (L974): _no doc comment_
- `markdown_pairing_boundary_tests::malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting` (L1001): _no doc comment_
