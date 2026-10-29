## 2024-05-18 - Replacing `W::name("...")` with cached accessors

**Learning:** Profiling showed that frequently creating `XName` references using `W::name("string literal")` dynamically allocates and hashes into an interning table, which acts as a performance bottleneck in hot paths, such as the comparer and DOM walks. We can replace hundreds of instances of this pattern with static once-lock cached values available in `src/namespaces.rs` for XML elements and attributes (e.g. `W::p()`, `W::type_()`).

**Action:** Replaced dynamic `W::name(...)` calls with their statically allocated function equivalents from the namespace modules. This bypasses the string processing logic in tight loop iterations.
