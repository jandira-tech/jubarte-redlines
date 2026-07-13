# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-12

### Added

- Initial release, extracted from the `ooxmlsdk-redline` development crate.
- `document_comparer::compare_documents` (+ `_with_options`,
  `_with_settings`): compare two `.docx` documents into a tracked-changes
  (redline) `.docx`.
- `document_comparer::get_revisions`: list tracked revisions (type, author,
  date, part, move group, format-change details, text).
- `document_comparer::accept_revisions` / `reject_revisions`: flatten a
  redline package-wide.
- `comparer::WmlComparerSettings`: author/date stamping, detail threshold,
  Word-visual alignment passes (default) or the PowerTools-faithful preset.
- `jubarte` CLI (default `cli` feature): plain compare plus `revisions` and
  `accept` subcommands.

### Fixed

- External hyperlinks no longer lose their targets in the default
  (Word-visual) mode: `unwrap_hyperlinks_to_styled_runs` now preserves
  `r:id`-bearing `w:hyperlink` wrappers and unwraps only anchor-based
  internal (TOC) hyperlinks, so relationship reconciliation keeps the
  hyperlink relationship (with `TargetMode="External"`) in the output.

### Known issues

- See [KNOWN_ISSUES.md](KNOWN_ISSUES.md); the covering tests are marked
  `#[ignore]` with matching reasons.

[0.1.0]: https://github.com/arthrod/jubarte-rs/releases/tag/v0.1.0
