## 2025-01-20 - [Legacy Document Processing Optimization]
**Learning:** In Rust document processing, using `flatten().cloned().collect()` on table rows during AST traversal (like generating placeholders) causes eager memory allocation and copies of all cell contents. For large legacy documents, this results in significant `O(N)` memory overhead.
**Action:** Always prefer lazy iterators (`.any()`, `.find()`, `.all()`) when inspecting or searching an AST (e.g. `document.blocks.iter().any()`) to avoid instantiating temporary `Vec`s and duplicating strings.
