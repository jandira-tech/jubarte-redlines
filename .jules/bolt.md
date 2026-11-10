## 2024-05-24 - Zero-allocation traversal
**Learning:** Avoid dom.elements() and dom.attributes() in hot paths as they allocate vectors.
**Action:** Use index-based loops or iterators instead.
