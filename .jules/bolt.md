## 2026-11-09 - Avoid `Option<XName>.clone()` when comparing inside hot loops
**Learning:** `XName` objects contain `Arc<str>` reference counts. Calling `.clone()` when passing `XName` objects to methods like `dom.name_is(&XName)` inside hot loops causes unnecessary atomic increments that hurt performance.
**Action:** Use borrowed references `nm` directly, avoid using `.clone()` on `XName` instances, e.g. use `dom.name_is(ae, &p_name)` instead of `dom.name_is(ae, &p_name.clone())`.
