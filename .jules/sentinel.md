## 2024-11-02 - [Quadratic Parsing in Pulldown-cmark Wikilinks]
**Vulnerability:** The codebase was vulnerable to RUSTSEC-2026-0433: Quadratic parsing of nested image wikilinks with `ENABLE_WIKILINKS` set in `pulldown-cmark` version `0.13.5`.
**Learning:** `pulldown-cmark` <=0.13.5 has an algorithmic complexity vulnerability (ReDoS/DoS) when parsing nested wikilink tags (`![[![[![[…]]]]]]`), causing CPU exhaustion.
**Prevention:** Make sure third party markdown parsers and similar text processors are updated to non-vulnerable versions. Upgraded `pulldown-cmark` to `0.13.6`.

## 2024-11-02 - [Quadratic Parsing in Pulldown-cmark Wikilinks]
**Vulnerability:** The codebase was vulnerable to RUSTSEC-2026-0433: Quadratic parsing of nested image wikilinks with `ENABLE_WIKILINKS` set in `pulldown-cmark` version `0.13.5`.
**Learning:** `pulldown-cmark` <=0.13.5 has an algorithmic complexity vulnerability (ReDoS/DoS) when parsing nested wikilink tags (`![[![[![[…]]]]]]`), causing CPU exhaustion.
**Prevention:** Make sure third party markdown parsers and similar text processors are updated to non-vulnerable versions. Upgraded `pulldown-cmark` to `0.13.6`.
