## 2024-11-10 - Avoid O(N) allocations in empty_placeholder
**Learning:** `empty_placeholder` used `document.blocks.iter().map(...).collect::<Vec<_>>()` which eagerly allocated a massive string vector for all paragraph and table texts across the whole document just to find a unique empty placeholder string.
**Action:** Use `.any()` and lazy iterators directly on the `.blocks` to verify a placeholder string without allocating `Vec<String>`. Avoid `.flatten().cloned().collect()` on large iterables like tables, replacing it with `.flatten().any(...)`.
