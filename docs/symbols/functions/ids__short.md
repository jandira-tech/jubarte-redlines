# Functions and methods in `src/ids/short.rs`

16 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `Step::parse`  (L73-91)
`fn parse(text: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `word` (ambiguous, 5 candidates) L75,78,81,84,87,90
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.strip_prefix`, `Self::Cell`, `Self::ContentControl`, `Self::Paragraph`, `Self::Row`, `Self::TextBox`, `Some`
- **Called by:** `ShortId::block` (ids/short.rs:311); `ShortId::range` (ids/short.rs:342)

#### `Step::number`  (L96-105, pub)
`pub fn number(self) -> usize`

- **Does:** The step's number.
- **Calls:** none
- **Called by:** `audit_report_with` (audit.rs:177) (?); `stale_fields` (audit.rs:869) (?); `ShortId::range` (ids/short.rs:342) (?)

#### `Step::renumbered`  (L108-117)
`fn renumbered(self, n: usize) -> Self`

- **Does:** The same kind of step, numbered `n`.
- **Calls (std/external/unresolved):** `Self::Cell`, `Self::ContentControl`, `Self::Paragraph`, `Self::Row`, `Self::Table`, `Self::TextBox`
- **Called by:** `<ShortId as fmt::Display>::fmt` (ids/short.rs:405) (?)

#### `settled_spelling`  (L128-182)
`fn settled_spelling(token: &str) -> Option<String>`

- **Does:** The settled spelling of an expired id, when one is clear (spec §2): `12` → `p12, r12 or c12`, `#12` → `r12`, `#c27` → `c27`, `t0.r1.c2` → `t0.row1.cell2`, `body:rev:12` → `r12`, `header1:rev:4` → `header1.r4`, `body:p:12` → `p12`, `header1:p:2` → `header1.p2`, `1+2` → `r1+2` (only a change id joins numbers), `t0.row1....
- **Calls (crate):** `number` (ids.rs:211) L130; `ShortId::parse` (ids/short.rs:356) L134,137
- **Calls (std/external/unresolved):** `.all`, `.as_slice`, `.bytes`, `.collect`, `.contains`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.next`, `.push_str`, `.push`, `.rsplit`, `.split_once`, `.split`, `.starts_with`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.then`, `.to_string`, `settled`
- **Macros:** `format!`
- **Called by:** `<ShortId as FromStr>::from_str` (ids/short.rs:387)

#### `<Step as fmt::Display>::fmt`  (L185-194, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `part_stem`  (L203-208)
`fn part_stem(head: &str) -> bool`

- **Does:** Whether `head` names a part (grok review of #24a: any lowercase word was a part, so `row`, `page` or `body1` parsed).
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.contains`, `.is_some`, `.iter`, `.strip_prefix`
- **Called by:** `ShortId::block` (ids/short.rs:311); `ShortId::change_of` (ids/short.rs:327)

#### `valid_path`  (L220-254)
`fn valid_path(story: &Story, path: &[Step]) -> bool`

- **Does:** Whether `path` is a block the grammar can name: a paragraph, table or content control of the story; a table's rows, a row's cells; a cell's paragraphs and nested tables; a paragraph's text boxes (each holding paragraphs and tables) and inline content controls.
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `Story::Part`
- **Macros:** `matches!`
- **Called by:** `ShortId::block` (ids/short.rs:311)

#### `ShortId::paragraph`  (L296-301, pub)
`pub fn paragraph(n: usize) -> Self`

- **Does:** A body paragraph, `p{n}`.
- **Calls (std/external/unresolved):** `Step::Paragraph`
- **Macros:** `vec!`
- **Called by:** `tests::the_constructors_print_what_the_view_prints` (ids/short.rs:569)

#### `ShortId::change`  (L304-309, pub)
`pub fn change(wid: u32) -> Self`

- **Does:** A body change, `r{wid}`.
- **Macros:** `vec!`
- **Called by:** `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?); `tests::the_constructors_print_what_the_view_prints` (ids/short.rs:569)

#### `ShortId::block`  (L311-325)
`fn block(token: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `Step::parse` (ids/short.rs:73) L314; `part_stem` (ids/short.rs:203) L316; `valid_path` (ids/short.rs:220) L324
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.into_iter`, `.map`, `.next`, `.split`, `.then_some`, `.to_string`, `Some`, `Story::Part`
- **Called by:** `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?); `ShortId::range` (ids/short.rs:342); `ShortId::parse` (ids/short.rs:356)

#### `ShortId::change_of`  (L327-339)
`fn change_of(token: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `part_stem` (ids/short.rs:203) L329
- **Calls (std/external/unresolved):** `.collect`, `.map`, `.rsplit_once`, `.split`, `.strip_prefix`, `.to_string`, `Some`, `Story::Part`
- **Called by:** `ShortId::parse` (ids/short.rs:356)

#### `ShortId::range`  (L342-354)
`fn range(first: &str, last: &str) -> Option<Self>`

- **Does:** `FIRST-N`: a block, then the number of its last step's last sibling.
- **Calls (crate):** `ShortId::block` (ids/short.rs:311) L343; `Step::parse` (ids/short.rs:73) L348; `.number` (ambiguous: `Field::number` (audit.rs:862); `Step::number` (ids/short.rs:96)) L350; `Step::number` (ids/short.rs:96) L350,352
- **Calls (std/external/unresolved):** `.clone`, `.last`, `.then`, `Box::new`, `std::mem::discriminant`
- **Called by:** `ShortId::parse` (ids/short.rs:356); `Levels::of` (revision_processor/style_records.rs:356) (?)

#### `ShortId::parse`  (L356-381)
`fn parse(token: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `ShortId::range` (ids/short.rs:342) L358; `number` (ids.rs:211) L364,367,375,378; `ShortId::change_of` (ids/short.rs:327) L380; `ShortId::block` (ids/short.rs:311) L380
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.filter`, `.is_ascii_digit`, `.map`, `.or_else`, `.split_once`, `.strip_prefix`, `.strip_suffix`, `digits`
- **Called by:** `settled_spelling` (ids/short.rs:128); `<ShortId as FromStr>::from_str` (ids/short.rs:387)

#### `<ShortId as FromStr>::from_str`  (L387-392, impl FromStr)
`fn from_str(token: &str) -> Result<Self, IdError>`

- **Does:** _no doc comment_
- **Calls (crate):** `ShortId::parse` (ids/short.rs:356) L388; `settled_spelling` (ids/short.rs:128) L389; `IdError::new` (ids.rs:102) L390
- **Calls (std/external/unresolved):** `.ok_or_else`
- **Called by:** none found in src

#### `<Story as fmt::Display>::fmt`  (L396-401, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.write_str`
- **Called by:** none found in src

#### `<ShortId as fmt::Display>::fmt`  (L405-436, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `Step::renumbered` (ids/short.rs:108) (by name) L423
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.is_empty`, `.iter`, `.join`, `.last`, `.map_or`, `.map`, `.write_str`, `Ok`, `Step::Paragraph`, `prefix`
- **Macros:** `write!`
- **Called by:** none found in src

## Test functions (4)

- `tests::every_settled_spelling_parses_and_prints_back` (L444): _no doc comment_
- `tests::expired_and_malformed_spellings_are_refused` (L494): _no doc comment_
- `tests::the_constructors_print_what_the_view_prints` (L569): _no doc comment_
- `tests::an_expired_id_is_refused_with_its_settled_spelling` (L575): _no doc comment_
