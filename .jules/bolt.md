
## 2024-05-18 - [Avoid `XName::clone` inside hot loops]
**Learning:** `XName` instances in this codebase contain `Arc<str>`. Cloning `XName` inside hot loops (like in `src/comparer/lcs.rs` and `src/comparer/finalize.rs` loops `dom.name_is(c, &props.clone())`) causes expensive atomic operations (increment and decrement) on every iteration.
**Action:** When passing `XName` variables to `dom.name_is` or checking against `Option<XName>`, pass by reference (e.g., `dom.name_is(c, &props)` and `nm.as_ref() == Some(&p_name)`) to avoid the unnecessary overhead.
