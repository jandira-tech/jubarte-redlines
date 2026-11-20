<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Audit and fields

`fields` needs the `render` feature. The audit builds without it; only
its layout rules (`FONT_SUBSTITUTED`, and the page-count check of
`STALE_FIELD_CACHE`) need the layout pass that `render` provides.

## Audit

`audit::audit_report(docx, rules)` runs accessibility, style and structure
rules and returns the findings, the codes that ran, and whether a layout
pass ran. `rules` takes rule sets (`a11y`, `style`, `structure`) or codes;
an empty list runs every rule.

| Code | Set | Severity |
|---|---|---|
| `HEADING_SKIP` | a11y | warning |
| `IMAGE_NO_DESCR` | a11y | error |
| `TABLE_NO_HEADER_ROW` | a11y | warning |
| `MISSING_LANG` | a11y | warning |
| `LITERAL_BULLET` | style | warning |
| `EMPTY_SPACER_PARAGRAPH` | style | info |
| `DIRECT_FORMATTING_OVERRIDES_STYLE` | style | info |
| `STALE_FIELD_CACHE` | structure | warning |
| `FONT_SUBSTITUTED` | structure | info |

`audit_report_with` takes the layout pass from the caller, so a build
without the renderer (the slim WebAssembly build) can still audit; without a
pass, `FONT_SUBSTITUTED` is left out.

## Fields

`fields::update_fields(docx)` recomputes field results (`TOC`, `PAGEREF`,
`REF`, `NUMPAGES`, `SEQ`) from jubarte's own layout and returns the
package, every field it rewrote with its old and new result, and the page
count.

```rust,ignore
{{#include ../../../../crates/jubarte-doc/examples/audit_fields.rs:example}}
```

Run it with `cargo run -p jubarte-doc --features render --example audit_fields`.
