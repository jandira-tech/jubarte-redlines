# Functions and methods in `src/perf.rs`

22 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `reset`  (L40-43, pub)
`pub fn reset()`

- **Does:** Resets all counters and stage timers to zero.
- **Calls (crate):** `active::reset` (perf.rs:240) L42
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `record_stage_ns`  (L48-55, pub)
`pub fn record_stage_ns(stage: Stage, nanos: u64)`

- **Does:** Adds `nanos` to the cumulative wall-time of `stage`.
- **Calls (crate):** `active::record_stage_ns` (perf.rs:253) L50
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `time_stage`  (L61-74, pub)
`pub fn time_stage<R>(stage: Stage, f: impl FnOnce() -> R) -> R`

- **Does:** Runs `f` and, when profiling, accumulates its wall duration into `stage`; otherwise returns `f()` directly.
- **Calls (crate):** `active::record_stage_ns` (perf.rs:253) L66
- **Calls (std/external/unresolved):** `.as_nanos`, `.elapsed`, `f`, `std::time::Instant::now`
- **Called by:** `tests::time_stage_returns_closure_result` (perf.rs:360)

#### `inc_lcs_calls`  (L77-80, pub)
`pub fn inc_lcs_calls()`

- **Does:** Increments the LCS-call counter.
- **Calls (crate):** `active::inc_lcs_calls` (perf.rs:257) L79
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `add_lcs_window_area`  (L85-92, pub)
`pub fn add_lcs_window_area(n: u64, m: u64)`

- **Does:** Accumulates LCS window area `n * m` (a saturation-safe product).
- **Calls (crate):** `active::add_lcs_window_area` (perf.rs:261) L87
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `inc_corr_run_scans`  (L97-100, pub)
`pub fn inc_corr_run_scans()`

- **Does:** Increments the correlation-run scan counter.
- **Calls (crate):** `active::inc_corr_run_scans` (perf.rs:265) L99
- **Called by:** `correlated_hash_run` (comparer/lcs.rs:9049) (?); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `inc_corr_run_hits`  (L105-108, pub)
`pub fn inc_corr_run_hits()`

- **Does:** Increments the correlation-run hit counter (a scan whose result was used).
- **Calls (crate):** `active::inc_corr_run_hits` (perf.rs:269) L107
- **Called by:** `correlated_hash_run` (comparer/lcs.rs:9049) (?); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `add_unit_clones`  (L112-119, pub)
`pub fn add_unit_clones(n: u64)`

- **Does:** Adds `n` to the unit-clone counter.
- **Calls (crate):** `active::add_unit_clones` (perf.rs:273) L114
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `set_atom_count`  (L124-131, pub)
`pub fn set_atom_count(n: u64)`

- **Does:** Records the atom count of the current compare, overwriting any earlier value.
- **Calls (crate):** `active::set_atom_count` (perf.rs:277) L126
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `set_unit_count`  (L136-143, pub)
`pub fn set_unit_count(n: u64)`

- **Does:** Records the unit count of the current compare, overwriting any earlier value.
- **Calls (crate):** `active::set_unit_count` (perf.rs:281) L138
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `Snapshot::to_json`  (L172-198, pub)
`pub fn to_json(&self) -> String`

- **Does:** Emits a single-line JSON object that names each stage in plain text (`package_open`, `preprocess`, `atomize`, `unitize`, `lcs`, `produce`, `serialize`, `zip`); no trailing newline.
- **Macros:** `concat!`, `format!`
- **Called by:** `tests::snapshot_json_is_valid_shape` (perf.rs:314)

#### `snapshot`  (L202-211, pub)
`pub fn snapshot() -> Snapshot`

- **Does:** Captures the current counters and stage timers without resetting them.
- **Calls (crate):** `active::snapshot` (perf.rs:285) L205
- **Calls (std/external/unresolved):** `Snapshot::default`
- **Called by:** `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::reset`  (L240-251, pub(super))
`pub(super) fn reset()`

- **Does:** _no doc comment_
- **Calls (crate):** `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L242,244,245,246,247,248...
- **Called by:** `reset` (perf.rs:40); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::record_stage_ns`  (L253-255, pub(super))
`pub(super) fn record_stage_ns(stage: Stage, nanos: u64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`
- **Called by:** `record_stage_ns` (perf.rs:48); `time_stage` (perf.rs:61); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::inc_lcs_calls`  (L257-259, pub(super))
`pub(super) fn inc_lcs_calls()`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`
- **Called by:** `inc_lcs_calls` (perf.rs:77); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::add_lcs_window_area`  (L261-263, pub(super))
`pub(super) fn add_lcs_window_area(n: u64, m: u64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`, `.saturating_mul`
- **Called by:** `add_lcs_window_area` (perf.rs:85); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::inc_corr_run_scans`  (L265-267, pub(super))
`pub(super) fn inc_corr_run_scans()`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`
- **Called by:** `correlated_hash_run` (comparer/lcs.rs:9049) (?); `inc_corr_run_scans` (perf.rs:97); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::inc_corr_run_hits`  (L269-271, pub(super))
`pub(super) fn inc_corr_run_hits()`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`
- **Called by:** `correlated_hash_run` (comparer/lcs.rs:9049) (?); `inc_corr_run_hits` (perf.rs:105); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::add_unit_clones`  (L273-275, pub(super))
`pub(super) fn add_unit_clones(n: u64)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`
- **Called by:** `add_unit_clones` (perf.rs:112); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::set_atom_count`  (L277-279, pub(super))
`pub(super) fn set_atom_count(n: u64)`

- **Does:** _no doc comment_
- **Calls (crate):** `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L278
- **Called by:** `set_atom_count` (perf.rs:124); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::set_unit_count`  (L281-283, pub(super))
`pub(super) fn set_unit_count(n: u64)`

- **Does:** _no doc comment_
- **Calls (crate):** `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L282
- **Called by:** `set_unit_count` (perf.rs:136); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

#### `active::snapshot`  (L285-305, pub(super))
`pub(super) fn snapshot() -> Snapshot`

- **Does:** _no doc comment_
- **Calls (crate):** `.load` (ambiguous, 8 candidates) L288,289,290,291,292,293...
- **Called by:** `snapshot` (perf.rs:202); `tests::reset_and_record_are_safe_without_feature` (perf.rs:333)

## Test functions (3)

- `tests::snapshot_json_is_valid_shape` (L314): _no doc comment_
- `tests::reset_and_record_are_safe_without_feature` (L333): _no doc comment_
- `tests::time_stage_returns_closure_result` (L360): _no doc comment_
