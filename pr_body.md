## Summary

This PR addresses all static analysis and compilation issues introduced in `fix/1287-v2-strike-restore`.

```diff
 src/markdown/xml.rs
-            ("before", _) if let Some(lines) = hundredths_lines(value) => {
-                self.before_lines = Some(lines);
-            }
+            ("before", _) => {
+                if let Some(lines) = hundredths_lines(value) {
+                    self.before_lines = Some(lines);
+                } else {
+                    (self.before, self.before_auto) = (Some(points(value)?), false);
+                }
+            }

 src/bin/jubarte.rs
-        !job.force && !(job.in_place && output == job.file) && output.symlink_metadata().is_ok()
+        !(job.force || job.in_place && output == job.file) && output.symlink_metadata().is_ok()

 tests/common/dead_comments.rs
+#![allow(dead_code)]

 tests/recorded_move_refusal.rs
-fn plan(extra: serde_json::Value) -> EditPlan {
+fn plan(extra: &serde_json::Value) -> EditPlan {
```

## Evidence

- **Before:**
  - `cargo clippy --all-targets --all-features -- -D warnings` fails due to:
    - Experimental `if let` guards (E0658) in `src/markdown/xml.rs:348` and `351`.
    - Unused functions `reference`, `deleted`, `probe`, `assert_state` in `tests/common/dead_comments.rs:23`.
    - `clippy::nonminimal-bool` in `src/bin/jubarte.rs:2597`.
    - `clippy::needless_pass_by_value` in `tests/recorded_move_refusal.rs:24`.
- **After:**
  - `cargo clippy --all-targets --all-features -- -D warnings` runs cleanly.
  - Tests build and run successfully.

## Merge Danger

**Door:** two-way

**Blast Radius:** Isolated

This fixes compilation and lint errors with trivial and semantic-preserving adjustments.
