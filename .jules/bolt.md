## 2024-10-27 - [Avoid per-paragraph string allocation on case-insensitive search]
**Learning:** [Using text.to_lowercase().contains() in loops causes massive allocations because to_lowercase allocates a new String on every call. Precompiling regex escapes avoids it.]
**Action:** [When searching iteratively (e.g. per-paragraph or per-line) case-insensitively, lift the pattern setup out of the closure and utilize regex pattern compilation rather than mutating subject strings.]
