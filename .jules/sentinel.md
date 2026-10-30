## 2024-05-24 - DoS risk from unbounded Regex compilation
**Vulnerability:** Unbounded user input passed to `regex::Regex::new()` can trigger a 'Compiled regex exceeds size limit' error, creating a DoS vulnerability.
**Learning:** `regex` compilation time and memory size scale with the input length. Even with `regex::escape`, very long inputs can exceed default size limits or cause excessive processing.
**Prevention:** In `jubarte-redlines`, inputs to `regex::Regex::new()` must be length-bounded to prevent DoS attacks.
