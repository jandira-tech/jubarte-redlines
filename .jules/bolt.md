## 2025-02-12 - [Avoid Eager Allocations in LegacyDocument Traversals]
**Learning:** [In `legacy_doc.rs`, eagerly collecting strings into large vectors from document blocks like `Block::Table` rows (e.g. using `.flatten().cloned().collect()`) causes unnecessary allocations and high peak memory overhead on large documents, especially for simple validation checks.]
**Action:** [Use lazy evaluation like `.iter().any(...)` or `.find(...)` to check for string presence incrementally without intermediate container allocations.]
