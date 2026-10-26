## Summary

```diff
-        .filter(|&e| {
-            dom.name(e).is_some_and(|n| n.local_name() == "docPr")
-                || dom.name_is(e, &WP::name("docPr"))
-        })
+    let wp_doc_pr = WP::name("docPr");
+    // ...
+        .filter(|&e| {
+            dom.name(e).is_some_and(|n| n.local_name() == "docPr")
+                || dom.name_is(e, &wp_doc_pr)
+        })
```

Hoists `XName` generation (which triggers string allocation and interning checks) out of `filter` closures and loops in hot DOM processing paths.

## Evidence

- **Before:** `WP::name("docPr")` and `O::name("OLEObject")` recomputed their names on every node iteration inside `filter()` and `for` loops in `src/comparer/fixups.rs`.
- **After:** `let wp_doc_pr = WP::name("docPr");` and `let o_ole_object = O::name("OLEObject");` run once, preventing O(n) re-interning. Tests pass correctly.

**Findings Fixed:**
- `src/comparer/fixups.rs:48` - Repeated recomputation of `WP::name` in `.filter()` hoisted.
- `src/comparer/fixups.rs:107` - Repeated recomputation of `O::name` in `for` loop hoisted.
- `src/comparer/fixups.rs:143` - Repeated recomputation of `O::name` in `for` loop hoisted.

## Merge Danger

**Door:** two-way

**Blast Radius:** Isolated (just internal drawing fixups algorithm speedup, no behavioral change)
