# Functions and methods in `src/markdown/anchor.rs`

11 non-test, 10 test. Part of the [symbol manifest](../README.md).

#### `hide`  (L26-28)
`fn hide(c: char) -> char`

- **Does:** A private-use stand-in for an escaped character, so mark stripping never sees it (Supplementary Private Use Area-A, U+F0000 onward).
- **Calls (std/external/unresolved):** `.unwrap_or`, `char::from_u32`, `u32::from`
- **Called by:** `blank` (markdown/anchor.rs:138); `read_anchor` (markdown/anchor.rs:273)

#### `reveal`  (L30-35)
`fn reveal(c: char) -> char`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.unwrap_or`, `char::from_u32`, `u32::from`
- **Called by:** none found in src

#### `space_reference`  (L39-45)
`fn space_reference(rest: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool`

- **Does:** Whether `&` (just read) starts `&#32;`, the agent view's leading space; the reference is consumed when it does.
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.eq`, `.nth`, `.take`
- **Called by:** `unescape_markdown` (markdown/anchor.rs:50); `read_anchor` (markdown/anchor.rs:273)

#### `unescape_markdown`  (L50-70, pub)
`pub fn unescape_markdown(text: &str) -> String`

- **Does:** Removes a backslash escape before a Markdown punctuation character (`\*` → `*`), as CommonMark reads it; backslashes that do not precede one of those characters are kept verbatim.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L55; `space_reference` (markdown/anchor.rs:39) L63
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.len`, `.next`, `.peekable`, `.push`, `String::with_capacity`
- **Called by:** `tests::unescape_markdown_resolves_punctuation_escapes_only` (markdown/anchor.rs:541)

#### `strip_between`  (L84-110)
`fn strip_between(text: &str, open: &str, close: &str, keep: Keep) -> String`

- **Does:** `open … close` spans replaced by what `keep` says; an unclosed `open` stays as text.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map_or`, `.push_str`, `.rsplit_once`, `.unwrap_or`, `String::new`
- **Called by:** `read_anchor` (markdown/anchor.rs:273)

#### `without_block_mark`  (L116-135)
`fn without_block_mark(text: &str) -> &str`

- **Does:** The text without a leading block mark: `# ` (one to six), `> `, a bullet (`- `, `+ `, `* `) or an ordered marker (`1.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.get`, `.strip_prefix`, `.take_while`, `.trim_start`, `Some`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `blank`  (L138-140)
`fn blank(c: char) -> bool`

- **Does:** Whitespace, or a `&#32;` [`plain_anchor`] hid.
- **Calls (crate):** `hide` (markdown/anchor.rs:26) L139
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `peel_story_final_groups` (comparer/lcs.rs:3533) (?); `collapse_ws` (convert/mod.rs:16256) (?); `without_emphasis` (markdown/anchor.rs:147); `mark_close` (markdown/critic.rs:1750) (?); `mark_closes` (markdown/critic.rs:1774) (?)

#### `without_emphasis`  (L147-241)
`fn without_emphasis(text: &str) -> String`

- **Does:** The text with its emphasis marks dropped: `**`, `~~`, `==`, `<u>`, `<mark>` (with or without the `class="hl-…"` colour the view prints for a highlight that is not yellow), `<s>` and their closers always; single `*` and `_` runs only when they pair up (`*x*`, `_x_`, `__x__`), so `2 * 3`, `snake_case`, `snake__case` and...
- **Calls (crate):** `blank` (markdown/anchor.rs:138) L166,198,199; `Anchors::fill` (comments.rs:352) (by name) L216,217
- **Calls (std/external/unresolved):** `.chars`, `.checked_sub`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.find`, `.get`, `.is_multiple_of`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.pop`, `.push`, `.replace_range`, `.replace`, `.rposition`, `.take_while`, `.to_string`, `.truncate`, `.zip`, `Vec::new`, `spaced`, `star`
- **Macros:** `vec!`
- **Called by:** `read_anchor` (markdown/anchor.rs:273)

#### `plain_anchor`  (L259-261, pub)
`pub fn plain_anchor(text: &str) -> String`

- **Does:** The document text an agent-view fragment stands for: - attribution notes and comment text dropped (`{r0 @AC}` etc.); - insertions unwrapped (`{++x++}` → `x`) and comment scopes too (`{==x==}`); - deletions removed (`{--x--}`); - substitutions become their new side (`{~~old~>new~~}` → `new`); - a block mark that leads...
- **Calls (crate):** `read_anchor` (markdown/anchor.rs:273) L260
- **Called by:** `tests::the_removed_reading_keeps_deletions_and_both_sides` (markdown/anchor.rs:370); `tests::plain_anchor_drops_the_marks_the_agent_view_adds` (markdown/anchor.rs:384); `tests::underscore_emphasis_reads_as_the_markdown_reader_reads_it` (markdown/anchor.rs:473); `tests::plain_anchor_keeps_mark_shaped_code_literal` (markdown/anchor.rs:506); `tests::extension_anchors_match_the_reader` (markdown/anchor.rs:511); `tests::plain_anchor_keeps_code_span_contents_literal` (markdown/anchor.rs:518); `tests::plain_anchor_keeps_all_code_span_spaces` (markdown/anchor.rs:526); `tests::plain_anchor_reads_code_span_padding` (markdown/anchor.rs:534)

#### `plain_anchor_with_removed`  (L267-269, pub(crate))
`pub(crate) fn plain_anchor_with_removed(text: &str) -> String`

- **Does:** [`plain_anchor`] with the removed text kept: a deletion unwrapped (`{--x--}` → `x`) and a substitution as both sides (`{~~old~>new~~}` → `oldnew`), the text a refusal looks for in another author's deletion.
- **Calls (crate):** `read_anchor` (markdown/anchor.rs:273) L268
- **Called by:** `tests::the_removed_reading_keeps_deletions_and_both_sides` (markdown/anchor.rs:370)

#### `read_anchor`  (L273-363)
`fn read_anchor(text: &str, deleted: Keep, substituted: Keep) -> String`

- **Does:** [`plain_anchor`]'s reading, a deletion read as `deleted` says and a substitution as `substituted` says.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L279; `hide` (markdown/anchor.rs:26) L284,317,331,337; `Parser::peek` (markdown/math.rs:804) (by name) L316,322,328; `space_reference` (markdown/anchor.rs:39) L336; `strip_between` (markdown/anchor.rs:84) L349,355,356,357,358; `without_attributions` (markdown/critic.rs:1402) L350; `without_emphasis` (markdown/anchor.rs:147) L361
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.contains`, `.extend`, `.into_offset_iter`, `.is_ascii`, `.join`, `.len`, `.map`, `.next`, `.peekable`, `.push_str`, `.push`, `.split`, `.trim_matches`, `Some`, `String::with_capacity`, `pulldown_cmark::Parser::new_ext`
- **Called by:** `plain_anchor` (markdown/anchor.rs:259); `plain_anchor_with_removed` (markdown/anchor.rs:267)

## Test functions (10)

- `tests::the_removed_reading_keeps_deletions_and_both_sides` (L370): _no doc comment_
- `tests::plain_anchor_drops_the_marks_the_agent_view_adds` (L384): _no doc comment_
- `tests::reader_text` (L460): The text pulldown-cmark, the crate's Markdown reader, keeps of `s`.
- `tests::underscore_emphasis_reads_as_the_markdown_reader_reads_it` (L473): _no doc comment_
- `tests::plain_anchor_keeps_mark_shaped_code_literal` (L506): _no doc comment_
- `tests::extension_anchors_match_the_reader` (L511): _no doc comment_
- `tests::plain_anchor_keeps_code_span_contents_literal` (L518): _no doc comment_
- `tests::plain_anchor_keeps_all_code_span_spaces` (L526): _no doc comment_
- `tests::plain_anchor_reads_code_span_padding` (L534): _no doc comment_
- `tests::unescape_markdown_resolves_punctuation_escapes_only` (L541): _no doc comment_
