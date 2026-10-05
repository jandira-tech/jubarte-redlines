#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
#
# Builds the Finder Sync extension (src-tauri/finder-sync) and puts it inside
# a built Jubarte.app, then signs the extension and re-signs the app around it:
#
#   scripts/build-finder-sync.sh APP IDENTITY [PROFILE]
#
#   APP       a built Jubarte.app (Tauri's bundle), changed in place
#   IDENTITY  the codesign identity the app is signed with: "Apple
#             Distribution: …" for the Mac App Store, "Developer ID
#             Application: …" for a direct build (hardened runtime added)
#   PROFILE   the extension's provisioning profile (App Store builds only:
#             one for the bundle id com.jandira.jubarte.finder)
#
# The menu's tests run first and stop the build when they fail. The app keeps
# the entitlements it was signed with.
set -euo pipefail
cd "$(dirname "$0")/.."

APP=${1:?usage: build-finder-sync.sh APP IDENTITY [PROFILE]}
IDENTITY=${2:?usage: build-finder-sync.sh APP IDENTITY [PROFILE]}
PROFILE=${3:-}
SRC=src-tauri/finder-sync
OUT=src-tauri/target/finder-sync
VERSION=$(node -p 'require("./src-tauri/tauri.conf.json").version')
[ -d "$APP/Contents/MacOS" ] || { echo "ERROR: $APP is not an app bundle" >&2; exit 1; }

rm -rf "$OUT"
mkdir -p "$OUT"
swiftc -parse-as-library "$SRC/FinderSyncMenu.swift" "$SRC/FinderSyncMenuTests.swift" -o "$OUT/menu-tests"
"$OUT/menu-tests"

for arch in arm64 x86_64; do
  swiftc -O -target "$arch-apple-macos12.0" -application-extension -module-name JubarteFinder \
    -Xlinker -e -Xlinker _NSExtensionMain -framework FinderSync -framework AppKit \
    "$SRC/FinderSyncMenu.swift" "$SRC/FinderSync.swift" -o "$OUT/JubarteFinder-$arch"
done

APPEX="$OUT/JubarteFinder.appex"
mkdir -p "$APPEX/Contents/MacOS"
lipo -create "$OUT/JubarteFinder-arm64" "$OUT/JubarteFinder-x86_64" -output "$APPEX/Contents/MacOS/JubarteFinder"
sed "s/@VERSION@/$VERSION/g" "$SRC/Info.plist" > "$APPEX/Contents/Info.plist"
plutil -lint "$APPEX/Contents/Info.plist" >/dev/null
[ -z "$PROFILE" ] || cp "$PROFILE" "$APPEX/Contents/embedded.provisionprofile"

RUNTIME=()
case "$IDENTITY" in "Developer ID Application:"*) RUNTIME=(--options runtime) ;; esac

# Inside out: the extension, then the app around it with its own entitlements.
mkdir -p "$APP/Contents/PlugIns"
rm -rf "$APP/Contents/PlugIns/JubarteFinder.appex"
cp -R "$APPEX" "$APP/Contents/PlugIns/"
codesign --force --timestamp "${RUNTIME[@]}" --sign "$IDENTITY" \
  --entitlements "$SRC/FinderSync.entitlements" "$APP/Contents/PlugIns/JubarteFinder.appex"
codesign -d --entitlements - --xml "$APP" > "$OUT/app.entitlements" 2>/dev/null
codesign --force --timestamp "${RUNTIME[@]}" --sign "$IDENTITY" \
  --entitlements "$OUT/app.entitlements" "$APP"
codesign --verify --deep --strict "$APP"
echo "Finder extension $VERSION inside $APP, signed by $IDENTITY"
