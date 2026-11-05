## 2024-11-05 - Unbounded Regex Compilation DoS

**Vulnerability:** Found an unbounded user input directly passed to `regex::Regex::new()` when processing regex queries.
**Learning:** The `regex` crate can exhaust memory or hit size limits when compiling extremely long or complex inputs, resulting in Denial of Service or "Compiled regex exceeds size limit" runtime errors. This occurs because unbounded size string inputs can be supplied as expressions.
**Prevention:** Always length-bound the user input string before attempting regex compilation and explicitly configure `.size_limit()` on a `RegexBuilder` to provide defense-in-depth against resource exhaustion.
