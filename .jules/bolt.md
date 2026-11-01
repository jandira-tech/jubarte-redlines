## 2026-10-29 - [Fast path for DOM attribute copying]
**Learning:** `dom.attributes(node)` clones all attributes into a `Vec<(XName, String)>`, meaning `for (an, av) in dom.attributes(node) { dom.set_attribute_value(ne, &an, Some(&av)); }` causes massive string and struct allocations in hot rebuild paths.
**Action:** Implemented and enforce usage of `dom.copy_attributes(src, dest)` which delegates to the internal DOM vector, executing a fast direct transfer of `Vec<Attr>` without allocating intermediary owned string pairs.
