## 2026-11-06 - pulldown-cmark Quadratic Parsing Vulnerability
**Vulnerability:** Quadratic parsing of nested image wikilinks with `ENABLE_WIKILINKS` (RUSTSEC-2026-0433) in `pulldown-cmark` version `0.13.5`.
**Learning:** `cargo-deny` checks failed because `pulldown-cmark` was identified as having a security vulnerability. This codebase had multiple Cargo workspaces, so updating dependencies must be done across all manifests to pass CI.
**Prevention:** Make sure to upgrade dependencies like `pulldown-cmark` across the entire workspace (all `Cargo.toml` files, e.g., root, `jubarte-python`, `jubarte-wasm`) using `cargo update -p <package> --manifest-path <path/to/Cargo.toml>` whenever security alerts trigger.
