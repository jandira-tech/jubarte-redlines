## Summary
This PR fixes a critical Regular Expression Denial of Service (ReDoS) vulnerability in the Regex search functionality.
Previously, the `--regex` input was directly compiled without length bounding.

## Evidence
Before:
An attacker could pass a 100MB regex string to `--regex`, leading to extreme memory exhaustion ("Compiled regex exceeds size limit") and CPU load during compilation.

After:
The regex pattern is now bounded to 1024 bytes.
```rust
if pattern.len() > 1024 {
    return Err(super::err("USAGE", None, "regex pattern exceeds maximum length of 1024 bytes"));
}
```

## Merge Danger
Door: two-way
Blast Radius: Low. This only restricts abnormally large regex patterns which are not typical for text editing operations.

---
🚨 Severity: CRITICAL
💡 Vulnerability: Regex Denial of Service (ReDoS) due to unbounded regex pattern input.
🎯 Impact: An attacker could crash the application or cause severe performance degradation by supplying an extremely large or complex regex pattern.
🔧 Fix: Bounded the input length of the regex pattern to 1024 bytes before compiling.
✅ Verification: Ran `cargo test -p jubarte-redlines --test m7_cli` and `cargo test -p jubarte-redlines --test read_for_edit` to ensure standard tests pass.
