## 2025-02-27 - [Avoid allocation in attributes loop]
**Learning:** `dom.attributes(node)` allocates a `Vec` and clones strings. `dom.attributes_iter(node)` borrows, avoiding allocation, but you cannot mutate `dom` inside the loop due to the active borrow.
**Action:** Use an index-based `while` loop with `dom.attr_count(node)` and `dom.attr_at(node, i)` to retrieve attributes allocation-free, dropping the borrow on `dom` so it can be mutated within the same loop.
