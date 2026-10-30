## Summary

A security fix to bump `pulldown-cmark` version to `0.13.5` because of a critical vulnerability: RUSTSEC-2026-0414 (Exponential event growth with `ENABLE_WIKILINKS`).

## Evidence

- **Before:** `cargo deny check advisories` failed with RUSTSEC-2026-0414 error message due to `pulldown-cmark` version `0.13.4` being vulnerable.
- **After:** `cargo update -p pulldown-cmark` updated it to `0.13.5`.

## Merge Danger

**Door:** two-way

**Blast Radius:** Isolated (only internal docx parsing dependency is bumped)

Findings fixed:
Cargo.lock:150: bump pulldown-cmark to v0.13.5 to address RUSTSEC-2026-0414
