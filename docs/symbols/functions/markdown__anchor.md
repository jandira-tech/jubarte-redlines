# Functions and methods in `src/markdown/anchor.rs`

9 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `hide`  (L26-28)
`fn hide(c: char) -> char`

- **Does:** A private-use stand-in for an escaped character, so mark stripping never sees it (Supplementary Private Use Area-A, U+F0000 onward).
- **Calls (std/external/unresolved):** `.unwrap_or`, `char::from_u32`, `u32::from`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528); `blank` (markdown/anchor.rs:130); `plain_anchor` (markdown/anchor.rs:251)

#### `reveal`  (L30-35)
`fn reveal(c: char) -> char`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.unwrap_or`, `char::from_u32`, `u32::from`
- **Called by:** none found in src

#### `space_reference`  (L39-45)
`fn space_reference(rest: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool`

- **Does:** Whether `&` (just read) starts `&#32;`, the agent view's leading space; the reference is consumed when it does.
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.eq`, `.nth`, `.take`
- **Called by:** `unescape_markdown` (markdown/anchor.rs:50); `plain_anchor` (markdown/anchor.rs:251)

#### `unescape_markdown`  (L50-70, pub)
`pub fn unescape_markdown(text: &str) -> String`

- **Does:** Removes a backslash escape before a Markdown punctuation character (`\*` → `*`), as CommonMark reads it; backslashes that do not precede one of those characters are kept verbatim.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L55; `space_reference` (markdown/anchor.rs:39) L63
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.len`, `.next`, `.peekable`, `.push`, `String::with_capacity`
- **Called by:** `tests::unescape_markdown_resolves_punctuation_escapes_only` (markdown/anchor.rs:425)

#### `strip_between`  (L81-102)
`fn strip_between(text: &str, open: &str, close: &str, keep: Keep) -> String`

- **Does:** `open … close` spans replaced by what `keep` says; an unclosed `open` stays as text.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map_or`, `.push_str`, `.rsplit_once`, `String::new`
- **Called by:** `plain_anchor` (markdown/anchor.rs:251)

#### `without_block_mark`  (L108-127)
`fn without_block_mark(text: &str) -> &str`

- **Does:** The text without a leading block mark: `# ` (one to six), `> `, a bullet (`- `, `+ `, `* `) or an ordered marker (`1.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.get`, `.strip_prefix`, `.take_while`, `.trim_start`, `Some`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `blank`  (L130-132)
`fn blank(c: char) -> bool`

- **Does:** Whitespace, or a `&#32;` [`plain_anchor`] hid.
- **Calls (crate):** `hide` (markdown/anchor.rs:26) L131
- **Calls (std/external/unresolved):** `.is_whitespace`
- **Called by:** `peel_story_final_groups` (comparer/lcs.rs:3533) (?); `collapse_ws` (convert/mod.rs:16220) (?); `without_emphasis` (markdown/anchor.rs:139); `mark_close` (markdown/critic.rs:1556) (?); `mark_closes` (markdown/critic.rs:1580) (?)

#### `without_emphasis`  (L139-233)
`fn without_emphasis(text: &str) -> String`

- **Does:** The text with its emphasis marks dropped: `**`, `~~`, `==`, `<u>`, `<mark>` (with or without the `class="hl-…"` colour the view prints for a highlight that is not yellow), `<s>` and their closers always; single `*` and `_` runs only when they pair up (`*x*`, `_x_`, `__x__`), so `2 * 3`, `snake_case`, `snake__case` and...
- **Calls (crate):** `blank` (markdown/anchor.rs:130) L158,190,191; `Anchors::fill` (comments.rs:342) (by name) L208,209
- **Calls (std/external/unresolved):** `.chars`, `.checked_sub`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.find`, `.get`, `.is_multiple_of`, `.is_some_and`, `.iter`, `.len`, `.map_or`, `.map`, `.pop`, `.push`, `.replace_range`, `.replace`, `.rposition`, `.take_while`, `.to_string`, `.truncate`, `.zip`, `Vec::new`, `spaced`, `star`
- **Macros:** `vec!`
- **Called by:** `plain_anchor` (markdown/anchor.rs:251)

#### `plain_anchor`  (L251-299, pub)
`pub fn plain_anchor(text: &str) -> String`

- **Does:** The document text an agent-view fragment stands for: - attribution notes and comment text dropped (`{>>r0 @AC<<}` etc.); - insertions unwrapped (`{++x++}` → `x`) and comment scopes too (`{==x==}`); - deletions removed (`{--x--}`); - substitutions become their new side (`{~~old~>new~~}` → `new`); - a block mark that le...
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L257,263,269; `hide` (markdown/anchor.rs:26) L258,272,278; `space_reference` (markdown/anchor.rs:39) L277; `strip_between` (markdown/anchor.rs:81) L290,291,292,293,294; `without_emphasis` (markdown/anchor.rs:139) L297
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.join`, `.len`, `.map`, `.next`, `.peekable`, `.push`, `.split`, `.trim_matches`, `Some`, `String::with_capacity`
- **Called by:** `tests::plain_anchor_drops_the_marks_the_agent_view_adds` (markdown/anchor.rs:306); `tests::underscore_emphasis_reads_as_the_markdown_reader_reads_it` (markdown/anchor.rs:392)

## Test functions (4)

- `tests::plain_anchor_drops_the_marks_the_agent_view_adds` (L306): _no doc comment_
- `tests::reader_text` (L382): The text pulldown-cmark, the crate's Markdown reader, keeps of `s`.
- `tests::underscore_emphasis_reads_as_the_markdown_reader_reads_it` (L392): _no doc comment_
- `tests::unescape_markdown_resolves_punctuation_escapes_only` (L425): _no doc comment_
