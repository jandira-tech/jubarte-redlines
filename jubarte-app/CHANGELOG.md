# Changelog

All notable changes to the Jubarte desktop app are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0: new
features bump the **minor**, fixes bump the **patch**).

See [README → Versioning & release](README.md#versioning--release) for how to cut
a new version.

## [0.7.1] — 2026-08-16

### Changed
- **Engine upgraded to jubarte-redlines 0.7.1.** Adds independent DOCX→PDF
  (`jubarte convert`). Redline behaviour is unchanged from 0.7.0.

### Fixed
- ASC skips builds with no `CFBundleVersion`.
- ASC `MAC_OS` train filter; drop deprecated `load_module`.
- Share the ASC module loader; tests use the script `PLATFORM`.

## [0.7.0] — 2026-08-13

### Changed
- **Engine upgraded to jubarte-redlines 0.7.0 — now the fastest redline engine
  as well as the most faithful.** On top of 0.6.0's fidelity lead, 0.7.0 wins
  every generation-speed measure against docxodus 9.0.0 (median 5.3 ms vs
  7.2 ms, mean 22.2 ms vs 24.1 ms, p95 94.8 ms vs 96.2 ms, p99 139.7 ms vs
  179.9 ms, throughput 45.0/s vs 41.4/s, 0 vs 120 generation failures),
  measured load-fair — every speed change is output-identical to 0.6.0. Also
  folds in the 0.6.0→0.7.0 correctness batch (M468–M496: mesh ordering,
  paragraph spacing, style/Normal merge, numbering, images, sections, fields).

### Added
- Changelog catch-up for **`asc-new-version.py`** (script already on main
  since `2de7e3a`, missed the 0.6.2 notes): creates and attaches a fresh
  App Store Connect version through the ASC API, so cutting a store
  submission no longer requires hand-clicking the version row.

### Fixed
- **`asc-new-version.py` attaches the matching marketing train.** It no
  longer picks the newest `VALID` build regardless of
  `CFBundleShortVersionString` (build `attributes.version` is the build
  number). Selection now requires an included `preReleaseVersion` whose
  version equals the requested train.

## [0.6.2] — 2026-08-11

### Fixed
- **Redlines failed to save on 0.6.1 ("cannot save into disk").** The app
  wrote the result next to the original document, but the Mac App Store
  sandbox's user-selected read-write entitlement covers only the files the
  user picked in the open dialog — never their parent folder — so every
  redline write was denied. Results now land in the app's own sandbox cache
  container (always writable); **Save a copy…** uses the system save dialog,
  which grants write access to wherever the user chooses. UI copy updated to
  point at "Save a copy" instead of promising a file beside the original.

### Changed
- **Engine upgraded to jubarte-redlines 0.6.0 — the best redline engine on
  the market.** On the 763-document Word-oracle benchmark it now leads
  docxodus 9.0.0 on every headline metric: fidelity mean 83.27 vs 80.55,
  median 91.67 vs 91.19, generation failures 0 vs 4, documents ≥90 403 vs
  392 — at 4× (median) to 10× (mean) docxodus's generation speed
  (20.7 ms vs 82.2 ms median per document).

## [0.6.1] — 2026-07-30

### Fixed
- **The app could not be built at all.** The engine crate was renamed
  `jubarte` → `jubarte-redlines` in the engine repo's v0.5.1 release, but
  `src-tauri/Cargo.toml` still required a package literally named `jubarte`, so
  every build died with `no matching package named 'jubarte' found`. The path
  dependency now renames explicitly
  (`jubarte = { package = "jubarte-redlines", path = "../.." }`), which keeps
  the `use jubarte::…` imports in `main.rs` unchanged.

### Submission
- Resubmission of 0.6.0's feature set after App Review rejection
  (submission `35ab86d9`, 2026-07-28) under guidelines 2.1(b) and 3.1.2(c).
  Both causes were App Store Connect metadata, not app code:
  the "Jubarte Pro Yearly" In-App Purchase was never attached to the review
  submission, and the App Description carried no Terms of Use (EULA) link.
  No user-facing behaviour changed in this version.

## [0.6.0] — 2026-07-21

### Added
- **5 free redlines per install.** The app is fully usable out of the box: the
  first five redlines are free (counted and enforced in Rust, persisted in the
  app's data container), and only after that does the subscription gate apply.
  A badge under the actions shows how many free redlines remain; clicking it
  opens the subscribe sheet, which is dismissable ("Not now") while free
  redlines remain.

### Fixed
- **Subscribe button could hang forever.** The product lookup inside a purchase
  had no timeout, so on a build without a working App Store context the button
  stayed on "Contacting the App Store…" indefinitely with no payment sheet.
  The lookup is now bounded (30 s) and failures surface a clear message.
- **Paying customers can no longer be locked out by a backend hiccup.** A
  successful Apple purchase now unlocks immediately off the on-device signed
  receipt; the server-side verification of the JWS is recorded best-effort in
  the background instead of gating the unlock.
- **Engine path dependency repaired** — `jubarte` now resolves to the enclosing
  engine checkout (the old `../../jubarte-rs` path no longer existed), so the
  app builds against the current, perf-optimized engine.

## [0.5.0] — 2026-07-15

### Changed
- **Product version 0.5.0** aligned with **jubarte-rs 0.5.0** (package-wide Word
  validity, notes/settings coherence, parity restore, measured Q0 engine stack).
- Version fields re-synced via `bun run bump 0.5.0`.

## [0.3.1] — 2026-07-15

### Changed
- **Engine path dep:** pulls **jubarte-rs 0.2.0** (package validity + notes/settings
  coherence + parity restore + measured Q0 perf stack). Version fields re-synced
  (`package.json` / `Cargo.toml` / `tauri.conf.json` / app-bar) via `bun run bump`.

## [0.3.0] — 2026-07-14

### Changed
- Version line advanced for desktop packaging; see engine changelog for core
  behavior. (App UI features remain those of 0.2.0 unless noted below.)

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

[0.6.1]: https://github.com/arthrod/jubarte-app/releases/tag/v0.6.1
[0.6.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.6.0
[0.5.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.5.0
[0.3.1]: https://github.com/arthrod/jubarte-app/releases/tag/v0.3.1
[0.3.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.3.0
[0.2.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.2.0
[0.1.0]: https://github.com/arthrod/jubarte-app/releases/tag/v0.1.0
