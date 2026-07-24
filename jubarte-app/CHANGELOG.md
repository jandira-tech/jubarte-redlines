# Changelog

All notable changes to the Jubarte desktop app are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0: new
features bump the **minor**, fixes bump the **patch**).

See [README → Versioning & release](README.md#versioning--release) for how to cut
a new version.

## [0.2.0] — 2026-07-14

### Added
- **"Revisions by" now defaults to the modified document's author.** The field
  is pre-filled from the modified `.docx`'s core properties (`dc:creator`,
  falling back to `cp:lastModifiedBy`) instead of the machine user, so tracked
  changes are attributed to whoever produced the modified version. Still fully
  editable — type over it and the auto-fill stops. A `from modified doc` hint
  appears when the value came from the file.
- **Editable "File name" field.** Name the output redline directly; the app
  proposes `<original>_v_<modified>.docx` and still dedupes with ` (n)` if a file
  by that name already exists next to the original.
- **Live redline preview panel.** A two-column layout — working controls on the
  left, the rendered redline on the right — with a legend and revision-count
  chips (inserted / deleted / moved / format).

### Changed
- **Faster redline engine.** Rebuilt on the profile-driven engine optimizations
  (memoized move-detection and format-change passes): roughly **2.5×** wall-clock
  on large dissimilar documents and **1.35×** on redline-vs-redlined-self pairs,
  with byte-for-byte identical output.
- **Redesigned interface** to the editorial-technical "redline desk": foam-blue
  token system derived from `#25628F`, Manrope + JetBrains Mono chrome, a
  Source Serif 4 document preview, and a tinted insert/delete/move treatment
  (background tint + colour + underline/strikethrough) that reads like a marked-up
  document rather than a diff. Sharp corners throughout.
- Window widened to **1120×820** (min 820×640) to fit the two-pane layout; it
  stacks to a single column under 900px.

## [0.1.0] — 2026-07-12

### Added
- Initial release. Drag & drop (or browse) the original and modified `.docx`;
  one-click tracked-changes redline written next to the original as
  `<a>_v_<b>.docx`.
- Swap original ↔ modified; inline preview of insertions/deletions/moves with
  revision counts; Open in Word / Show in Finder / Save a copy.
- Finder **Open with… → Jubarte**: select two `.docx` files and both slots fill
  (older file becomes the original), then the redline runs automatically.
- Signed (Developer ID, hardened runtime) and notarized (app + DMG).

[0.2.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.2.0
[0.1.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.1.0
