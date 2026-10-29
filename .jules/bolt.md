## 2024-10-27 - [Avoid per-paragraph string allocation on case-insensitive search]
**Learning:** [Using text.to_lowercase().contains() in loops causes massive allocations because to_lowercase allocates a new String on every call. Precompiling regex escapes avoids it.]
**Action:** [When searching iteratively (e.g. per-paragraph or per-line) case-insensitively, lift the pattern setup out of the closure and utilize regex pattern compilation rather than mutating subject strings.]

## 2024-10-27 - [Regex `(?i)` vs `to_lowercase` semantics]
**Learning:** [Precompiling a regex with `(?i)` for case-insensitive matching instead of using `to_lowercase().contains()` fails because `(?i)` folds case differently (e.g., `ß` and other folds that change length) changing match results, and long literals hit regex size limits.]
**Action:** [Avoid replacing `to_lowercase()` with `(?i)` regex matching unless you can guarantee the input size is small and the application explicitly accepts regex case-folding semantics instead of Unicode lowercase folding.]
