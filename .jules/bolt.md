## 2024-11-03 - Lazy Evaluation on AST Elements
**Learning:** Eagerly allocating strings from all AST elements (e.g. `table.rows.iter().flatten().cloned().collect()`) just to search for a placeholder string causes significant $O(N)$ memory and time overhead on large documents, especially when this is done as part of formatting or conversion operations.
**Action:** Always prefer lazy evaluation with iterators (e.g. `any()`, `find()`) against AST blocks rather than collecting them into full strings when merely checking for existence or searching.
