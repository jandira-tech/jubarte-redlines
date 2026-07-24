# Jubarte (desktop)

Proprietary desktop app for [jubarte](https://github.com/arthrod/jubarte-rs):
drop two Word documents, get a tracked-changes redline that opens cleanly in
Microsoft Word. This repository is **not** open source; the comparison engine
it embeds (`jubarte`, AGPL-3.0) is.

## Features

- Drag & drop (or click to browse) the original and modified `.docx`
- One-click redline, written next to the original; the output name is editable
  (defaults to `<a>_v_<b>.docx`, deduped with ` (n)`)
- **"Revisions by" defaults to the modified document's author** (`dc:creator`,
  falling back to `cp:lastModifiedBy`) — editable, so the tracked changes are
  attributed to whoever produced the modified version
- Swap original ↔ modified instantly
- Two-pane live preview: insertions / deletions / moves with a legend and
  revision-count chips
- Open in Word / Show in Finder / Save a copy
- Finder "Open with… → Jubarte": select two `.docx` files and both slots fill
  (older file becomes the original), then the redline runs automatically

## Stack

Tauri 2 (Rust backend, static vanilla frontend — no bundler). The engine is a
path dependency on `../jubarte-rs` during development; switch
`src-tauri/Cargo.toml` to the crates.io `jubarte` release once published.

## Develop

```sh
bun install
bun run dev        # tauri dev
```

## Build (signed)

```sh
bun run build      # tauri build → .app + .dmg, both signed
```

Signing uses the keychain identity configured in `src-tauri/tauri.conf.json`
(`Developer ID Application: Jandira Technologies, LLC (NW99N2W6TA)`), with the
hardened runtime enabled — a prerequisite for notarization. The bundles land in:

- `src-tauri/target/release/bundle/macos/Jubarte.app`
- `src-tauri/target/release/bundle/dmg/Jubarte_<version>_aarch64.dmg`

`tauri build` prints `Warn skipping app notarization, no APPLE_ID …` — that is
expected. We do **not** hand credentials to Tauri; notarization is a manual step
below using the `notarytool-cicero` keychain profile.

## Notarize (app + DMG)

Prerequisites (already set up on the build machine):

- The Developer ID identity above is in the login keychain
  (`security find-identity -v -p codesigning`).
- A notarytool keychain profile named `notarytool-cicero` exists
  (`xcrun notarytool store-credentials notarytool-cicero --apple-id … --team-id NW99N2W6TA --password <app-specific-password>`).

Run from the bundle directory:

```sh
cd src-tauri/target/release/bundle
IDENTITY="Developer ID Application: Jandira Technologies, LLC (NW99N2W6TA)"
APP="macos/Jubarte.app"
DMG="dmg/Jubarte_0.1.0_aarch64.dmg"          # match the built version

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
bun run bump 0.3.0
```

That rewrites all four:

| File | Field |
|---|---|
| `package.json` | `"version"` |
| `src-tauri/tauri.conf.json` | `"version"` (drives the bundle + DMG filename) |
| `src-tauri/Cargo.toml` | `[package] version` |
| `src/index.html` | the app-bar `vX.Y.Z` label |

The bump script deliberately does **not** touch the CHANGELOG — you write that.

Release flow:

1. `bun run bump <x.y.z>` — bump all four version strings.
2. Add a dated section to [`CHANGELOG.md`](CHANGELOG.md) (Added / Changed / Fixed).
3. `bun run build` — produces the signed `.app` (see *Build* above).
4. Notarize and staple the app, then the DMG (see *Notarize* above). Update the
   `DMG="dmg/Jubarte_<version>_aarch64.dmg"` line to the new version.
5. Commit (`chore(release): vX.Y.Z`), tag `vX.Y.Z`, push.

## Icons & art

The whale lives in `assets/whale.svg` (hero, inlined into `src/index.html`)
and `assets/icon.svg` (app icon). After editing:

```sh
bun run icons      # re-render PNG + regenerate src-tauri/icons/*
```
