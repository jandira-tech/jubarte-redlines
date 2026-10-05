# Jubarte (desktop)

Proprietary desktop app for
[jubarte-redlines](https://github.com/jandira-tech/jubarte-redlines): drop two
Word documents, get a tracked-changes redline that opens cleanly in Microsoft
Word. This repository is **not** open source; the comparison engine it embeds
(crate `jubarte-redlines`, Rust path `jubarte::`, AGPL-3.0) is.

| | |
|---|---|
| **Visibility** | Private (`arthrod/jubarte-app`) |
| **License** | Proprietary — see [LICENSE](LICENSE) |
| **Engine** | AGPL-3.0 `jubarte-redlines` (`jubarte::`) via path dep on this checkout |
| **MSRV** | 1.90 (edition 2024) |

## Repository layout

This app is a plain directory tracked in the engine monorepo —
[jubarte-redlines](https://github.com/jandira-tech/jubarte-redlines), the
canonical checkout — with its own nested `.git` (no `.gitmodules`, nothing to
`--recurse-submodules`). The engine crate sits at the repo root and is consumed
through the path dependency
`jubarte = { package = "jubarte-redlines", path = "../..", default-features = false }`
in `src-tauri/Cargo.toml`, so a checkout of the monorepo builds as-is.

## Features

- Drag & drop (or click to browse) the original and modified `.docx`
- One-click redline, written into the app's own cache container — the App
  Sandbox blocks writing next to the picked inputs — with **"Save a copy"** as
  the export path to wherever you choose; the output name is editable
  (defaults to `<a>_v_<b>.docx`, deduped with ` (n)`)
- **"Revisions by" defaults to the modified document's author** (`dc:creator`,
  falling back to `cp:lastModifiedBy`) — editable, so the tracked changes are
  attributed to whoever produced the modified version
- Swap original ↔ modified instantly
- Two-pane live preview: insertions underlined in blue, deletions struck in
  red, moves in green (double-struck where they left, double-underlined where
  they landed), with revision-count chips that wear the same marks
- Open in Word / Show in Finder / Save a copy
- **Convert to PDF**: a second tab takes one `.docx` and writes a PDF through
  the engine's layout (`jubarte::convert`), tracked changes in red, blue and
  green or as Word prints them; the PDF shows in the window, opens in Preview,
  reveals in Finder or saves anywhere
- **Redline or convert from Finder**: select two `.docx` files, right-click →
  **Compare with Jubarte** (at the top of the menu), **Quick Actions → Redline
  with Jubarte**, **Open With → Jubarte**, or drop both on the Dock icon. Both
  slots fill (the older file becomes the original) and the redline runs.
  Right-click one `.docx` (or three or more) for **Convert to PDF with
  Jubarte**, a PDF of each — see [Finder](#finder)
- **Five free uses per install (a redline or a PDF each, counted when the user
  opens, shows in Finder or saves it; a preview is free), then an annual
  subscription** (StoreKit 2):
  the quota is counted and enforced in Rust
  ([`src-tauri/src/quota.rs`](src-tauri/src/quota.rs)), the paywall lives in
  [`src/paywall.js`](src/paywall.js), and the entitlement is verified
  server-side
- **Mac App Store distribution**: `pnpm publish:mac`
  ([`scripts/publish-mac-app-store.sh`](scripts/publish-mac-app-store.sh))
  builds, signs and uploads the `.pkg` in one command — see
  [`MAC_APP_STORE_RELEASE.md`](MAC_APP_STORE_RELEASE.md)

Three sibling packages live in this checkout beside the app:

- [`redlines-site/`](redlines-site/) — the public web front end (Cloudflare
  Worker): drop two `.docx` in a browser; five free redlines per visitor
- [`verify-worker/`](verify-worker/) — the StoreKit receipt-verification
  backend, the authoritative entitlement record the app checks
- [`jubarte-site/`](jubarte-site/) — the `jubarte.pro` marketing Worker,
  hosting the Terms and Privacy pages the paywall links to

## Finder

Four Finder entry points share one hand-off
([`src-tauri/src/finder.rs`](src-tauri/src/finder.rs)): the paths are queued in
Rust, the window is brought forward and told to drain the queue, so a launch
that races the webview still runs each request exactly once.

- **Redline with Jubarte** and **Convert to PDF with Jubarte** are macOS
  Services declared under `NSServices` in
  [`src-tauri/Info.plist`](src-tauri/Info.plist) and provided by
  [`src-tauri/src/finder_service.rs`](src-tauri/src/finder_service.rs). Each
  hands over the files with what it asked for (`finder::Intent`), so the
  window switches to the matching tab. They accept `.docx` only
  (`NSSendFileTypes`) and are enabled by default (empty `NSRequiredContext`).
  Services cannot filter on how many files are selected, so both items show
  for any `.docx` selection: Convert makes a PDF of each file, one after
  another; Redline with one file fills the original slot. If a fresh install
  does not show them yet, macOS has not re-read the services cache: open
  Jubarte once, or run `/System/Library/CoreServices/pbs -update`.
- **Compare with Jubarte** and **Convert to PDF with Jubarte** at the top of
  the right-click menu come from a Finder Sync extension
  ([`src-tauri/finder-sync/`](src-tauri/finder-sync/)), the way Araxis Merge
  adds its own item. It counts the selection: exactly two `.docx` show
  Compare, any other number Convert, and anything else (a folder, a `.pdf`,
  a `~$` lock file) shows nothing. The choice lives in
  [`FinderSyncMenu.swift`](src-tauri/finder-sync/FinderSyncMenu.swift), tested
  by `FinderSyncMenuTests.swift`. A click calls the matching service above, so
  the sandboxed extension needs no file access of its own and the app gets
  the files exactly as from the Services menu.
  [`scripts/build-finder-sync.sh`](scripts/build-finder-sync.sh) runs those
  tests, builds the universal `.appex`, places it in `Contents/PlugIns` of a
  built `Jubarte.app` and signs inside out. If the items do not show, turn
  the extension on under **System Settings → General → Login Items &
  Extensions**, or run `pluginkit -e use -i com.jandira.jubarte.finder` and
  relaunch Finder. An App Store build needs its own
  App ID and provisioning profile for `com.jandira.jubarte.finder`, passed as
  the script's third argument.
- **Open With → Jubarte** and a drop on the Dock icon arrive as
  `RunEvent::Opened`. The `.docx` association has `LSHandlerRank=Alternate`, so
  Word stays the double-click default.

Without a service's intent, one file fills the next empty slot (or the
Convert slot) and two fill both and run at once. Word's `~$name.docx` lock
files are ignored. The services are AppKit glue, so they are checked by hand:
[`scripts/finder-service-smoke.swift`](scripts/finder-service-smoke.swift)
invokes one exactly as Finder does (`swift scripts/finder-service-smoke.swift
A.docx B.docx`, or `--convert A.docx` for the PDF service).

## Stack

Tauri 2 (Rust backend, static vanilla frontend — no bundler). The engine is a
path dependency on the enclosing `jubarte-redlines` checkout (`path = "../.."`
from `src-tauri`); the same crate is published to crates.io as
`jubarte-redlines`, so a version pin is available if the monorepo layout ever
gets in the way.

Rust hygiene: `rustfmt.toml`, Clippy lints in `Cargo.toml`, `Cargo.lock`
committed (binary), `publish = false`. CI lives in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml).

## Develop

From `jubarte-app/` in the monorepo checkout:

```sh
pnpm install
pnpm dev        # tauri dev
```

Before opening a PR:

```sh
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D clippy::correctness
cargo check --all-targets
```

CI (`.github/workflows/ci.yml`) runs exactly that clippy invocation — the lint
levels live in `[lints]` in `Cargo.toml`, where `correctness` is deny and the
groups being cleaned up stay warnings — plus `cargo test --all-targets` and a
line-coverage gate (≥ 80%) on the free-quota business logic.

## Build (signed)

```sh
pnpm build      # tauri build → the signed .app
```

Signing uses the keychain identity configured in `src-tauri/tauri.conf.json`
(`Apple Distribution: Jandira Technologies, LLC (NW99N2W6TA)`), with the App
Sandbox entitlements from `src-tauri/entitlements.plist`. The bundle target is
`app` only — the Mac App Store needs a `.pkg`, not a `.dmg`, and that `.pkg` is
built by [`scripts/publish-mac-app-store.sh`](scripts/publish-mac-app-store.sh)
(see [`MAC_APP_STORE_RELEASE.md`](MAC_APP_STORE_RELEASE.md)). The bundle lands
in `src-tauri/target/release/bundle/macos/Jubarte.app`.

## Notarize (app + DMG) — direct distribution, kept for reference

The shipping path is the Mac App Store flow above. For a direct
(Developer ID) distribution you would additionally notarize an `.app` + `.dmg`;
this is the manual recipe from before the Store, kept for reference.

Prerequisites (already set up on the build machine):

- The Developer ID identity used in `IDENTITY` below is in the login keychain
  (`security find-identity -v -p codesigning`).
- A notarytool keychain profile named `notarytool-cicero` exists
  (`xcrun notarytool store-credentials notarytool-cicero --apple-id … --team-id NW99N2W6TA --password <app-specific-password>`).

Run from the bundle directory:

```sh
cd src-tauri/target/release/bundle
IDENTITY="Developer ID Application: Jandira Technologies, LLC (NW99N2W6TA)"
APP="macos/Jubarte.app"
DMG="dmg/Jubarte_0.10.1_aarch64.dmg"          # match the built version

# 1. Notarize the .app (zip → submit → staple).
ditto -c -k --keepParent "$APP" Jubarte.zip
xcrun notarytool submit Jubarte.zip --keychain-profile notarytool-cicero --wait
xcrun stapler staple "$APP"

# 2. Rebuild the DMG from the *stapled* app so the copy inside is stapled too,
#    then sign it.
rm -rf dmg-staging && mkdir dmg-staging
cp -R "$APP" dmg-staging/
ln -s /Applications dmg-staging/Applications
rm -f "$DMG"
hdiutil create -volname "Jubarte" -srcfolder dmg-staging -ov -format UDZO "$DMG"
codesign --force --sign "$IDENTITY" --timestamp "$DMG"

# 3. Notarize the DMG and staple it.
xcrun notarytool submit "$DMG" --keychain-profile notarytool-cicero --wait
xcrun stapler staple "$DMG"

rm -rf dmg-staging Jubarte.zip
```

Verify the result — both must report `source=Notarized Developer ID`:

```sh
spctl -a -vvv --type exec "$APP"
spctl -a -vvv --type open --context context:primary-signature "$DMG"
xcrun stapler validate "$APP" "$DMG"
```

Notes:

- Rebuilding the DMG with `hdiutil create -format UDZO` (step 2) is deliberate:
  it embeds the stapled app, and it sidesteps Tauri's pretty-DMG script, whose
  Finder AppleScript times out on a headless machine.
- Notarization matches on the signed content's hash, so the app must be signed
  (hardened runtime) **before** it is stapled, and the DMG must be built from
  the already-stapled app.

## Versioning & release

The app follows [semantic versioning](https://semver.org/) (pre-1.0: new features
bump the **minor**, fixes bump the **patch**). The version is hard-coded in four
places, kept in sync by one helper:

```sh
pnpm bump 0.3.0
```

That rewrites all four:

| File | Field |
|---|---|
| `package.json` | `"version"` |
| `src-tauri/tauri.conf.json` | `"version"` (drives the bundle name) |
| `src-tauri/Cargo.toml` | `[package] version` |
| `src/index.html` | the app-bar `vX.Y.Z` label |

The bump script deliberately does **not** touch the CHANGELOG — you write that.

Release flow:

1. `pnpm bump <x.y.z>` — bump all four version strings.
2. Add a dated section to [`CHANGELOG.md`](CHANGELOG.md) (Added / Changed / Fixed).
3. `pnpm publish:mac` — build, sign, package and upload the `.pkg` in one
   command (see [`MAC_APP_STORE_RELEASE.md`](MAC_APP_STORE_RELEASE.md); Apple
   rejects a re-used version number, hence step 1).
4. Attach the build and submit for review in App Store Connect.
5. Commit (`chore(release): vX.Y.Z`), tag `vX.Y.Z`, push.

## Icons & art

The whale lives in `assets/whale.svg` (hero, inlined into `src/index.html`)
and `assets/icon.svg` (app icon). After editing:

```sh
pnpm icons      # re-render PNG + regenerate src-tauri/icons/*
```
