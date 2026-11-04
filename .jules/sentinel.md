## 2024-05-24 - [Fix Regex DoS]
**Vulnerability:** Unbounded regex compilation allows DoS.
**Learning:** regex crate size limit needs to be constrained.
**Prevention:** Bound input length and set lower size limit.
