# Functions and methods in `src/convert/word_subst.rs`

11 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `parse_table`  (L23-61, pub(crate))
`pub(crate) fn parse_table(src: &str) -> Vec<SubstRow>`

- **Does:** _no doc comment_
- **Calls (crate):** `flush` (markdown/view_stories.rs:1334) (by name) L48,59; `unquote` (convert/word_subst.rs:63) L52,54; `parse_string_array` (convert/word_subst.rs:89) L56
- **Calls (std/external/unresolved):** `.clear`, `.is_empty`, `.lines`, `.push`, `.starts_with`, `.strip_prefix`, `.take`, `.trim`, `Some`, `Vec::new`, `std::mem::take`
- **Called by:** none found in src

#### `unquote`  (L63-69)
`fn unquote(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `unescape` (convert/word_subst.rs:72) L66
- **Calls (std/external/unresolved):** `.and_then`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.trim`
- **Called by:** `parse_table` (convert/word_subst.rs:23)

#### `unescape`  (L72-85)
`fn unescape(s: &str) -> String`

- **Does:** TOML basic-string escapes the table uses (`\"`, `\\`).
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.next`, `.push`, `String::with_capacity`
- **Called by:** `unquote` (convert/word_subst.rs:63)

#### `parse_string_array`  (L89-121)
`fn parse_string_array(s: &str) -> Vec<String>`

- **Does:** A one-line TOML array of basic strings.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.map`, `.push`, `.trim_end_matches`, `.trim_start_matches`, `.trim`, `String::new`, `Vec::new`, `std::mem::take`
- **Called by:** `parse_table` (convert/word_subst.rs:23); `tests::stems_keep_commas_and_escaped_quotes_inside_a_string` (convert/word_subst.rs:221)

#### `normalize`  (L123-128)
`fn normalize(name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.to_ascii_lowercase`, `.trim`
- **Called by:** `lookup_physical` (convert/word_subst.rs:136)

#### `rows`  (L130-132, pub(crate))
`pub(crate) fn rows() -> &'static [SubstRow]`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `CommentFamily::read_meta` (comments.rs:485) (?); `lookup_physical` (convert/word_subst.rs:136); `unknown_physical` (convert/word_subst.rs:144); `tests::stems_keep_commas_and_escaped_quotes_inside_a_string` (convert/word_subst.rs:221); `tests::table_contains_required_rows` (convert/word_subst.rs:327)

#### `lookup_physical`  (L136-142, pub(crate))
`pub(crate) fn lookup_physical(requested: &str) -> Option<String>`

- **Does:** Look up an evidence-table row for `requested` (after comma-split).
- **Calls (crate):** `normalize` (convert/word_subst.rs:123) L137; `rows` (convert/word_subst.rs:130) L138
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.iter`, `.map`
- **Called by:** `Fonts::resolve_walk` (convert/font.rs:1818); `tests::table_contains_required_rows` (convert/word_subst.rs:327)

#### `unknown_physical`  (L144-149, pub(crate))
`pub(crate) fn unknown_physical() -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `rows` (convert/word_subst.rs:130) L145
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.into`, `.iter`, `.map_or_else`
- **Called by:** `Fonts::resolve_walk` (convert/font.rs:1818); `tests::table_contains_required_rows` (convert/word_subst.rs:327)

#### `entry_generic`  (L157-169, pub(crate))
`pub(crate) fn entry_generic(entry: &FontEntry) -> &'static str`

- **Does:** The catalogue face Word draws an absent font-table entry in (Word 16 probes my6/my7, unique names, both orders).
- **Calls (std/external/unresolved):** `.as_deref`, `.eq_ignore_ascii_case`, `.is_some_and`
- **Called by:** `Fonts::resolve_walk` (convert/font.rs:1818); `tests::an_absent_entry_is_cambria_when_roman_and_calibri_otherwise` (convert/word_subst.rs:244)

#### `script_face`  (L179-187, pub(crate))
`pub(crate) fn script_face(entry: &FontEntry) -> Option<&'static str>`

- **Does:** The installed face an absent entry's script coverage picks, whatever its family says: a `w:sig` claiming Armenian is Sylfaen (even beside Hebrew and Arabic bits, as Arial's own sig has them), one claiming Thaana is MV Boli (55fcbe9086's Faruma).
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `add_installed_faces` (convert/font.rs:3395); `tests::a_sig_claiming_armenian_or_thaana_picks_that_script_face` (convert/word_subst.rs:283)

#### `word_name_substitute`  (L193-213, pub(crate))
`pub(crate) fn word_name_substitute(name: &str) -> Option<&'static str>`

- **Does:** Word's own stand-in for a family it knows by name, ahead of the font table's altName (Word 16 probes my2–my5, fresh sessions): the family itself or the family and a style word ("Myriad Pro Light"), never a PostScript name ("MyriadPro-Regular" is unknown).
- **Calls (std/external/unresolved):** `.any`, `.find`, `.is_empty`, `.is_some_and`, `.iter`, `.split`, `.starts_with`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim`, `Some`
- **Called by:** `add_installed_faces` (convert/font.rs:3395); `tests::word_knows_some_absent_families_by_name_only` (convert/word_subst.rs:302)

## Test functions (6)

- `tests::stems_keep_commas_and_escaped_quotes_inside_a_string` (L221): _no doc comment_
- `tests::entry` (L236): _no doc comment_
- `tests::an_absent_entry_is_cambria_when_roman_and_calibri_otherwise` (L244): _no doc comment_
- `tests::a_sig_claiming_armenian_or_thaana_picks_that_script_face` (L283): _no doc comment_
- `tests::word_knows_some_absent_families_by_name_only` (L302): _no doc comment_
- `tests::table_contains_required_rows` (L327): _no doc comment_
