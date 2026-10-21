# Functions and methods in `src/util/words.rs`

3 non-test, 9 test. Part of the [symbol manifest](../README.md).

#### `is_continuation`  (L25-29)
`fn is_continuation(c: char) -> bool`

- **Does:** A combining mark (Unicode category Mn, Mc or Me), an emoji skin tone modifier (U+1F3FB to U+1F3FF, a symbol that behaves like a mark) or the zero width joiner: it continues the token of the character before it.
- **Calls (std/external/unresolved):** `.contains`, `.general_category_group`
- **Called by:** `word_tokens` (util/words.rs:65)

#### `is_symbol`  (L32-34)
`fn is_symbol(c: char) -> bool`

- **Does:** A symbol (an emoji, `+`, `$`): a base that marks and joiners attach to.
- **Calls (std/external/unresolved):** `.general_category_group`
- **Called by:** `word_tokens` (util/words.rs:65)

#### `word_tokens`  (L65-120, pub)
`pub fn word_tokens(text: &str) -> Vec<&str>`

- **Does:** Split `text` into a `Vec<&str>` of tokens and return them.
- **Calls (crate):** `is_continuation` (util/words.rs:25) L73,90,104; `is_symbol` (util/words.rs:32) L76,80
- **Calls (std/external/unresolved):** `.char_indices`, `.is_alphanumeric`, `.is_whitespace`, `.len`, `.push`, `Kind::Other`, `Some`, `Vec::new`
- **Called by:** `units` (edit/rewrite.rs:34); `tests::words_spaces_and_marks` (util/words.rs:128); `tests::composed_and_decomposed_text_tokenize_alike` (util/words.rs:141); `tests::a_decomposed_word_is_one_token` (util/words.rs:166); `tests::a_mark_without_a_base_is_a_token_of_its_own` (util/words.rs:189); `tests::a_zero_width_joiner_stays_inside_its_token` (util/words.rs:204); `tests::a_skin_tone_modifier_stays_in_its_emoji_sequence` (util/words.rs:223); `tests::an_alphabetic_mark_with_no_base_does_not_start_a_word` (util/words.rs:234); `tests::ascii_and_cjk_tokens_do_not_change` (util/words.rs:241); `tests::tokens_always_concatenate_to_the_text` (util/words.rs:258)

## Test functions (9)

- `tests::words_spaces_and_marks` (L128): _no doc comment_
- `tests::composed_and_decomposed_text_tokenize_alike` (L141): Same text, composed and decomposed: the same number of tokens.
- `tests::a_decomposed_word_is_one_token` (L166): _no doc comment_
- `tests::a_mark_without_a_base_is_a_token_of_its_own` (L189): A mark with no letter or digit before it (it follows whitespace, punctuation or the start of the text) never joins that punctuation or whitespace: a run of such marks is a token of its own, and the next word starts a new token.
- `tests::a_zero_width_joiner_stays_inside_its_token` (L204): _no doc comment_
- `tests::a_skin_tone_modifier_stays_in_its_emoji_sequence` (L223): _no doc comment_
- `tests::an_alphabetic_mark_with_no_base_does_not_start_a_word` (L234): _no doc comment_
- `tests::ascii_and_cjk_tokens_do_not_change` (L241): _no doc comment_
- `tests::tokens_always_concatenate_to_the_text` (L258): _no doc comment_
