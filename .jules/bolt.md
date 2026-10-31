## 2025-01-20 - [Legacy Document Processing Optimization]
**Learning:** In Rust document processing, using `flatten().cloned().collect()` on table rows during AST traversal (like generating placeholders) causes eager memory allocation and copies of all cell contents. For large legacy documents, this results in significant `O(N)` memory overhead.
**Action:** Always prefer lazy iterators (`.any()`, `.find()`, `.all()`) when inspecting or searching an AST (e.g. `document.blocks.iter().any()`) to avoid instantiating temporary `Vec`s and duplicating strings.

## 2025-01-20 - [Legacy Document Processing Optimization Rejected]
**Learning:** Replaced eager `Vec<String>` collection with lazy `.any()` evaluation to avoid memory allocation when searching for empty placeholders. However, the optimization was rejected because `paragraph.text()` rebuilds the string by concatenating spans. The lazy evaluation caused `paragraph.text()` to be recomputed for every candidate string, which was significantly worse than the original eager `O(N)` allocation that computed and cached the texts once.
**Action:** When replacing eager collections with lazy iterators, verify that the condition being evaluated doesn't perform expensive operations (like string concatenation). If it does, computing and caching the results once is preferable to recomputing them in a loop.
