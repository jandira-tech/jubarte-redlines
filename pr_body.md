## Summary

This PR addresses two primary items identified during static triage:

1.  **Type Safety (E0658) in `src/markdown/xml.rs`:** In Rust MSRV 1.94.0, `if let` guards in `match` arms are experimental. This PR replaces them with nested `if let` blocks inside standard `match` arms.
2.  **Type Safety (Dead Code) in `tests/common/dead_comments.rs`:** `assert_state` is a helper function used across shared testing modules. This PR adds `#![allow(dead_code)]` to the top of the file to prevent the compiler from emitting warnings for modules that compile but don't strictly call it.

## Evidence

- **Before:**
  *   `cargo clippy --all-targets --all-features` reported `E0658` for `if let` guards and `dead_code` warnings.
- **After:**
  *   The code cleanly compiles without `E0658` errors or `dead_code` warnings (`cargo check --workspace --tests`).

## Merge Danger

**Door:** two-way

**Blast Radius:** Isolated

The modifications apply solely to internal documentation processing and test modules. They do not change any public API signatures.
