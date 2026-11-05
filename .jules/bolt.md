## 2024-01-01 - [Avoid Composite String Tuples]
**Learning:** Deduplicating items by multi-part keys in hot loops (e.g., validation checks) allocating composite string tuples `(String, String)` on every iteration causes massive allocation overhead.
**Action:** Use nested collections `HashMap<String, HashSet<String>>` to hoist the first part of the lookup and minimize string cloning allocations.
