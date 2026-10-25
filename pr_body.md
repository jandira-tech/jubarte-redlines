## Summary

```diff
-//! name as its context. The archive's name picks the layout, as self_update's
+//! name as its context. The archive's name picks the layout, as `self_update`'s
 //! own archive detection does: `.tar.gz` and `.tgz` are tar, `.zip` is zip.
 //! The check reads a `.zip` on every platform, though only a build with
-//! self_update's `archive-zip` (Windows) can then extract it.
+//! `self_update`'s `archive-zip` (Windows) can then extract it.
...
+    // zipsign signs the exact archive name as context, so the extension case
+    // is cryptographically bound.
+    #[allow(clippy::case_sensitive_file_extension_comparisons)]
     let (message, signatures) = if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
```

## Evidence

- **Before:** `cargo clippy -p jubarte-redlines -- -D warnings -W clippy::pedantic` emitted missing backticks in documentation and warnings for case-sensitive file extension checks.
- **After:** `cargo clippy -p jubarte-redlines -- -D warnings -W clippy::pedantic` runs cleanly; tests pass.

## Merge Danger

**Door:** two-way

**Blast Radius:** Isolated

Minor linter fixes inside documentation and a targeted `#[allow]` for case-sensitive extension comparison in `zipsign.rs`.
