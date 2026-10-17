# Functions and methods in `src/util/mod.rs`

5 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `sha1_fingerprint`  (L34-36, pub)
`pub fn sha1_fingerprint(s: &str) -> u64`

- **Does:** Former name of [`sha1::fnv1a_64`] at this path.
- **Calls (std/external/unresolved):** `sha1::fnv1a_64`
- **Called by:** none found in src

#### `universal_measure_twips`  (L42-44, pub(crate))
`pub(crate) fn universal_measure_twips(value: &str) -> Option<f64>`

- **Does:** A universal measure (`"612pt"`, `"1in"`, `"2.54cm"`, `"1mm"`, `"6pc"`) as twips, `None` for a plain number or an unknown unit.
- **Calls (crate):** `universal_measure` (util/mod.rs:51) L43
- **Called by:** `normalize_measures::{fn to_twips}` (comparer/finalize.rs:11543); `id_line` (markdown/from_docx/agent.rs:1514); `twips` (markdown/from_docx/header.rs:561); `tests::decodes_every_suffix_as_twips` (util/mod.rs:98); `tests::a_plain_number_or_garbage_is_not_a_universal_measure` (util/mod.rs:123); `tests::an_absurd_or_out_of_range_measure_is_not_a_measure` (util/mod.rs:134)

#### `universal_measure_half_points`  (L47-49, pub(crate))
`pub(crate) fn universal_measure_half_points(value: &str) -> Option<f64>`

- **Does:** The same measure as half-points (`ST_HpsMeasure`, `w:sz`): points × 2.
- **Calls (crate):** `universal_measure` (util/mod.rs:51) L48
- **Called by:** `apply` (markdown/from_docx/header.rs:744); `tests::an_absurd_or_out_of_range_measure_is_not_a_measure` (util/mod.rs:134); `tests::half_points_are_points_times_two` (util/mod.rs:157)

#### `universal_measure`  (L51-72)
`fn universal_measure(value: &str, per_point: f64) -> Option<f64>`

- **Does:** _no doc comment_
- **Calls (crate):** `schema_number` (util/mod.rs:76) L65
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.is_finite`, `.strip_suffix`, `.then_some`, `.trim`
- **Called by:** `universal_measure_twips` (util/mod.rs:42); `universal_measure_half_points` (util/mod.rs:47)

#### `schema_number`  (L76-89)
`fn schema_number(s: &str) -> Option<f64>`

- **Does:** `ST_UniversalMeasure`'s numeric part, `-?[0-9]+(\.[0-9]+)?`: digits on both sides of the point, a leading `-` as the only sign, no exponent.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.ok`, `.parse`, `.split_once`, `.strip_prefix`, `.unwrap_or`, `Some`
- **Called by:** `universal_measure` (util/mod.rs:51)

## Test functions (4)

- `tests::decodes_every_suffix_as_twips` (L98): Every suffix, signed, with the rounding the callers do: `12.95pt` is 259 twips once rounded.
- `tests::a_plain_number_or_garbage_is_not_a_universal_measure` (L123): A plain number is not a universal measure, and neither is anything the schema does not spell: callers keep their own integer path.
- `tests::an_absurd_or_out_of_range_measure_is_not_a_measure` (L134): The schema's numeric part is `-?digits(.digits)?`: no exponent, no `nan`/`inf`, no `+`, no bare or dangling point — Rust's f64 grammar is wider, and a magnitude past the twips domain's i64 saturates when the callers cast.
- `tests::half_points_are_points_times_two` (L157): The half-point domain `w:sz` reads: `11pt` is 22 half-points, not the twips the layout attributes carry.
