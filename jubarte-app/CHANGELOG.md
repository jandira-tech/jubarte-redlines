# Changelog

All notable changes to the Jubarte desktop app are recorded here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-1.0: new
features bump the **minor**, fixes bump the **patch**).

See [README → Versioning & release](README.md#versioning--release) for how to cut
a new version.

## [Unreleased]

## [0.11.3] — 2026-10-06

### Added
- **Compare and Convert from Finder's right-click menu.** "Compare with
  Jubarte" appears for two selected `.docx` files and "Convert to PDF with
  Jubarte" for one or more, at the top of the menu. Each runs the app's
  existing Finder service with the selection, so the sandboxed app reads
  the files without a temporary entitlement exception.

### Fixed
- **The app bar moves the window from anywhere but Settings.** It took only
  presses on the bar's own background; the title and version covered most
  of it.

### Changed
- **Engine upgraded to jubarte-redlines 0.11.3**:
  - **Math in exported PDFs follows Word.** Equations are set in Cambria
    Math with italic letters, display equations are centred, and brackets
    and separators are drawn where Word draws them.
  - **The font report no longer passes a guessed face as a match.** A font
    placed on an installed face only by its name's class counts as
    substituted.
  - **A package that repeats a relationship Id** with an identical
    relationship opens (the repeat is dropped and reported); two different
    relationships under one Id are refused, as Word refuses them.
  On the release's two 600-item samples, scored against Word's own output: redlines mean 72.06 (Docxodus 65.15), counting as 0 the 23 redlines Word 16.115 crashes opening; PDFs mean 79.14 (LibreOffice 54.61) with no failed document.

## [0.11.2] — 2026-10-03

### Changed
- **Engine upgraded to jubarte-redlines 0.11.2**:
  - **Redlines follow Word's own verdict.** A changed paragraph is marked
    word by word when what the two versions share (the kept words with
    their blanks, plus the paragraph mark) reaches 15 % of the longer
    version, and replaced whole under that, the rule measured in 1,178 Word
    comparisons. A replaced hyperlink shows its insertion first, as Word
    writes it.
  - **Export PDF and conversion follow Word's PDFs more closely.** Comments
    print as Word's balloons, in the author's tint with the range bracketed.
    Headers (a page break inside one, a flat connector), tables (rows inside
    content controls, skipped grid columns, double borders), footnotes,
    justified lines (including WordPerfect-style justification) and embedded
    fonts follow rules measured in Word.
    On the release's two 600-item samples, scored against Word's own output: redlines mean 76.71 (Docxodus 69.24) with no failed pair, PDFs mean 78.77 (LibreOffice 54.06) with no failed document.
- **The interface is unchanged from 0.11.1.** Nothing in the app's frontend
  (`src/`) or its Tauri shell (`src-tauri/`) moved except the four version
  strings (`package.json`, `src-tauri/tauri.conf.json`,
  `src-tauri/Cargo.toml`, the app bar in `src/index.html`) and the engine
  pin in `src-tauri/Cargo.lock`: `git diff 65efbd6 -- src src-tauri` (the
  0.11.1 release commit) shows only those lines. Every difference a user
  sees comes from the engine.

## [0.11.1] — 2026-10-03

### Fixed
- **A refused document says why in a sentence**: a Word 97-2003 `.doc` or a
  password-protected file reads "The original document is a Word 97-2003
  (.doc) or encrypted document; open it in Word and save it as .docx without
  a password." An RTF file or a package over the engine's limits names the
  document and the reason. Both used to show the engine's raw
  "Comparison failed: I/O error: LEGACY_DOC: …", and converting one showed
  "Conversion failed: opening DOCX: …".

### Changed
- Built on the released jubarte-redlines 0.11.0 engine (tag v0.11.0); the
  0.11.0 upload was built before the engine release.

## [0.11.0] — 2026-10-02

### Added
- **Side by side**: a redline shows alone or beside the original, its
  insertions taken out and its deletions kept, unmarked. The choice is
  remembered.
- **Export PDF** (File › Export Redline as PDF, ⌘E) lays the redline out as
  a PDF, its tracked changes drawn with the marks Settings chose. It is a
  free preview, opened, shown in Finder and saved as a PDF; taking it spends
  a free use. "← Back to the redline" returns to the redline.
- **Instant preview** (Settings › Preview, on by default): documents under
  1 MB each are redlined, or converted, the moment they are chosen, dropped
  or swapped, without pressing the button. The size is
  `app.instant_preview_max_bytes` in `data/facts.jsonl`; jubarte.pro's Demo
  and App pages do the same, with a checkbox to turn it off.
- **Settings (⌘, or the gear in the title bar)**: how tracked changes are
  marked (Conventional, As Word prints, or Custom: a colour and a line for
  insertions, deletions and each end of a move, used by the preview and the
  PDFs); a fixed "Revisions by" name for every redline; and Light, Dark or
  System appearance. The Convert tab's menu is the same setting.
- **Agent fingerprint** (Settings › Revisions by): for an agent that makes
  redlines, text written into each one as the custom document property
  `AgentFingerprint` (Word: File › Properties › Custom). It never appears in
  the text. One line, at most 255 characters; a property of the same name in
  any case is replaced, since Word refuses a package that holds two.
- **Menu bar**: every option the window has, where a Mac user looks for
  it. Jubarte: About, Settings (⌘,), Jubarte PRO, Restore Purchase. File:
  Choose Original (⌘O), Modified (⇧⌘O) or a Document to Convert (⌥⌘O); Make
  Redline (⌘R) or PDF (⇧⌘R); Swap; Open Result (⌘↓), Show in Finder, Save a
  Copy (⌘S). View: Redline (⌘1), Convert to PDF (⌘2), Appearance. Help: the
  website, use cases, benchmark, support, Terms of Use and Privacy Policy.
- **About Jubarte**: the version, the engine it runs, where your documents
  go (nowhere), the website and support, and the open-source
  acknowledgements.
- **Terms of Use and Privacy Policy in the app**: the same sections
  jubarte.pro prints, from `data/facts.jsonl` compiled in, so they read
  offline; their links open in the browser.

### Changed
- **The window, redrawn** after the Claude Design "Jubarte App Window":
  the title bar reads JUBARTE; Redline and Convert to PDF are tabs at the
  head of the panel; the panel folds into a strip of icons (the app icon,
  Show panel, Add documents, the two modes, Open, Show in Finder, Save a
  copy, the free uses) with the collapse button or View › Show or Hide Panel
  (⌃⌘S), and stays as it was left. Open, Show in Finder and Save a copy sit
  in a row, with icons, for a redline and for a PDF; the free uses and
  Instant preview sit at the foot of the panel. Convert offers PDF, and PNG
  pages as coming soon. The tracked-change marks are chosen in Settings
  alone.
- **A control that cannot act says why**: pressing Create redline with one
  document keeps the redline and points at the missing slot ("Add the
  modified document to make a redline."); Open, Show in Finder and Save a
  copy say what they wait for. The empty preview chooses documents when
  clicked and says what it waits for.
- **Settings, tidied**: each kind of change picks its line from a row of
  "Aa" drawn with that line in its colour, in place of a menu of names; the
  groups are headed in small mono capitals; Appearance says what it does to
  the window and that pages stay white; the fingerprint note gives its
  limit (one line, 255 characters). The preview's page is set larger (16 px,
  wider margins), as a printed page reads.
- A Finder hand-off during Export PDF waits for it instead of freezing the
  window; a PDF of a stale redline is drawn stale; going back and exporting
  again shows the same PDF, so taking it spends nothing twice; turning
  Instant preview on never remakes a current result; a redline whose
  documents changed mid-run is drawn stale; Return on a link or in the
  paywall no longer starts a run; with the panel folded, a nudge points at
  the strip; nudges are read out by VoiceOver.
- The floating and drifting whales stand still for anyone who asked macOS
  for reduced motion.
- **App icon: Night.** jubarte.pro's whale mark, flat in its pale blue, on
  the site's Night colours, so the Dock and the site show one whale. The same
  `assets/icon.svg` draws jubarte.pro's favicon.
- **A free use is spent when you take a result, not when it is made.** A
  redline or a PDF is a free preview in the window; the first Open, Show in
  Finder or Save a copy of it spends the use (`take_result` runs the gate) and
  copies it out of the previews folder, which empties at each launch. Finder's
  Convert to PDF still spends one per PDF. A preview never opens the paywall
  on its own. The Terms say so.
- **Preview marks**: insertions and deletions are marked once (underlined in
  blue, struck in red) and a move twice, in green: double-struck where it left
  and double-underlined where it landed. The "— moved from §N" note is gone: the struck source shows where the
  text came from.
- **No legend**: the revision chips now wear their marks (the deleted count is
  struck) and replace the legend that repeated them. The formatting chip reads
  "n Formatted".

- **One record of every changeable fact**: `data/facts.jsonl` holds the
  versions, prices, free uses, sizes, release list, benchmark figures, page
  limit and each section of the Terms and the Privacy Policy, one append-only
  record a line with a uuidv7 id and its timestamp. `scripts/facts.py` (Python
  3.14) is its only writer; the site reads it, and sync-release.ts appends a
  new engine release to it.
- **Jubarte PRO**: the subscription sheet is titled Jubarte PRO, as the
  website calls the Mac app, and its legal links open the text in the app.
- **Apple Account**: the sheet points to System Settings → Apple Account →
  Subscriptions, as macOS now names it (it said Apple ID).

### Fixed
- **No network at launch**: the app's fonts (Manrope, JetBrains Mono, Source
  Serif 4, SIL OFL 1.1) are bundled instead of fetched from Google Fonts, so
  opening Jubarte contacts no one; only a purchase check goes online.
- **About Jubarte opens on Done**: it used to open with the Engine link
  focused and ringed, so Return opened the link instead of closing the window.

## [0.10.1] — 2026-09-30

### Changed
- **Engine upgraded to jubarte-redlines 0.10.1**: tracked changes can be
  accepted or rejected one at a time, as Word's Accept / Reject This Change;
  Reject All matches Word on text and mark state; and PDF export follows Word
  much more closely on page breaks, keep-with-next, table rows, floating
  tables, headers, footers and comment balloons.

## [0.10.0] — 2026-09-28

### Changed
- **Engine upgraded to jubarte-redlines 0.10.0**: a changed text box stays one
  box with its changed words marked inside it, each section's headers and
  footers are compared with their own section's, changes in a header or
  footer holding a logo or hyperlink now reach the redline, and comments stay
  on their own occurrence of repeated text. Redline output changes from 0.9.3.
- The document preview and the author lookup read with quick-xml 0.42.

## [0.9.3] — 2026-09-27

### Changed
- **Engine upgraded to jubarte-redlines 0.9.3**, a redline release: replaced
  regions follow Word's replace-gap grammar, comments, bookmarks, field codes
  and footnotes survive the comparison, rejecting every change restores the
  original, and redlines Word refused to open now open. Redline output
  changes from 0.9.2.

## [0.9.2] — 2026-09-26

### Changed
- **Engine upgraded to jubarte-redlines 0.9.2**, and the app version now
  follows the engine's. PDF conversion gains the 0.9.x Word-fidelity pass and
  smaller `--compress` output; redline behaviour is unchanged.

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
