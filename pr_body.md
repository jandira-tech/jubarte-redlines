## Summary

```diff
-    let stream = loop {
-        match listener.accept() {
-            Ok((stream, _)) => break stream,
-            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
-                std::thread::sleep(std::time::Duration::from_millis(1));
-            }
-            Err(e) => panic!("{e}"),
-        }
-    };
+    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
+    let stream = loop {
+        match listener.accept() {
+            Ok((stream, _)) => break stream,
+            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
+                if std::time::Instant::now() > deadline {
+                    panic!("timeout waiting for connection");
+                }
+                std::thread::sleep(std::time::Duration::from_millis(1));
+            }
+            Err(e) => panic!("{e}"),
+        }
+    };
```

## Evidence

- **Before:** The regression test added in `#1859` contained an unbounded `loop` matching on `.accept()`. If the thread failed to connect, the loop would wait and sleep for `1ms` indefinitely, causing a test suite hang.
- **After:** The loop is now bounded by a 5-second `std::time::Instant` deadline. If it expires, the test panics and fails correctly rather than hanging. The test suite (`cargo test -p jubarte-redlines --test edit_single_output`) passes.

## Merge Danger

**Door:** two-way
**Blast Radius:** Isolated (Test suite only)

### Findings Addressed:
- `tests/edit_single_output.rs:850` - Bounded Async Operations: Added explicit timeout bounds to the polling loop in `the_hook_waits_for_a_line_written_after_connecting` to prevent unbounded hangs.
