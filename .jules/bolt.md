## 2024-10-25 - Performance pattern: String joining
**Learning:** Known performance pattern in Rust to avoid `.collect::<Vec<_>>().join(" ")` in favor of iterator loops or `itertools::Itertools::join`. We manually unrolled it into a `String::new()` loop.
**Action:** When manipulating strings, iterate and push to `String` (with capacity if known) instead of allocating intermediate vectors.
