## 2024-11-05 - Avoid `.clone()` when using `name_is` on `Dom`
**Learning:** The `dom.name_is(node_id, &XName)` method takes a reference to the `XName`. Previously, some hot loops were doing `dom.name_is(c, &props.clone())` or similar. Since `XName` is a struct containing an `Arc<str>` and an `XNamespace`, cloning it is relatively cheap, but doing so in inner hot loops (like DOM traversal) still adds up to measurable CPU/memory overhead.
**Action:** When querying the DOM name using `name_is`, pass the `XName` reference directly without calling `.clone()`.
