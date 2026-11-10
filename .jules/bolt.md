## 2024-11-06 - Memory Check

**Learning:** `jubarte-redlines` DOM operations are memory allocations when `.elements()` and `.attributes()` are used. Use index-based iteration with `child_count`/`child_at` and `attr_count`/`attr_at` to avoid allocations in hot paths.
**Action:** Find usages of `dom.elements(node, None)` or `dom.attributes(node)` and replace them with `dom.child_count()`/`dom.child_at()` and `dom.attr_count()`/`dom.attr_at()` loops to reduce allocations in hot paths.
