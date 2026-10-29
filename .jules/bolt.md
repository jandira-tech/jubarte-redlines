## 2024-10-24 - Avoid intermediate Vec allocations during string join operations
**Learning:** Found several occurrences of the anti-pattern `.collect::<Vec<_>>().join()` which creates an unnecessary intermediate `Vec` allocation before performing the string concatenation. This causes a redundant memory allocation and copy.
**Action:** Replace `.collect::<Vec<_>>().join(sep)` with a direct iterator loop where we pre-allocate a `String` (or use the first element) and `.push_str()` the subsequent elements with the separator.

## 2024-10-25 - Avoid manually unrolling Iterator to avoid `Vec` allocations in `join`
**Learning:** Manually unrolling an iterator into a `.push_str()` loop instead of using `.collect::<Vec<_>>().join()` or the standard library's `Itertools::join()` (when available) reads worse and can be rejected by maintainers, particularly in non-hot cold paths. This optimization was rejected ("cold-path `.join` rewrites that read worse than what they replace").
**Action:** Avoid replacing `.collect::<Vec<_>>().join(...)` with manual string push loops unless inside a proven performance hotspot, and consider the readability tradeoff.
