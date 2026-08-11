#!/usr/bin/env bash
#
# Build, sign, package, and upload Jubarte to the Mac App Store.
# One command does everything up to (but NOT including) "Submit for Review",
# which stays a manual click in App Store Connect.
#
# Prereqs (one-time, already set up on this machine):
#   - Xcode + `cargo-tauri` (`cargo install tauri-cli` or via the repo).
#   - Signing certs in the login keychain: "Apple Distribution: …" and
#     "3rd Party Mac Developer Installer: …".
#   - App Store Connect API key at ~/.appstoreconnect/private_keys/AuthKey_<ASC_KEY_ID>.p8
#   - src-tauri/embedded.provisionprofile present.
#
# Usage:
#   1. Bump the version first (Apple rejects a re-used version):
#        npm run bump          # or edit "version" in package.json + src-tauri/tauri.conf.json
#   2. Run this:
#        ./scripts/publish-mac-app-store.sh
#
set -euo pipefail

# ── config ───────────────────────────────────────────────────────────────────
BUNDLE_ID="com.jandira.jubarte"
APP_APPLE_ID="6790926615"
SIGN_APP="Apple Distribution: Jandira Technologies, LLC (NW99N2W6TA)"
SIGN_PKG="3rd Party Mac Developer Installer: Jandira Technologies, LLC (NW99N2W6TA)"
# Credentials are never committed (SECURITY.md): env first, then the local
# untracked ~/.appstoreconnect/jubarte.json ({"key_path","issuer_id","key_id"}).
CRED_FILE="$HOME/.appstoreconnect/jubarte.json"
ASC_KEY_ID="${ASC_KEY_ID:-}"
ASC_ISSUER="${ASC_ISSUER_ID:-}"
if { [ -z "$ASC_KEY_ID" ] || [ -z "$ASC_ISSUER" ]; } && [ -f "$CRED_FILE" ]; then
  ASC_KEY_ID="${ASC_KEY_ID:-$(python3 -c 'import json,os;print(json.load(open(os.path.expanduser("~/.appstoreconnect/jubarte.json")))["key_id"])')}"
  ASC_ISSUER="${ASC_ISSUER:-$(python3 -c 'import json,os;print(json.load(open(os.path.expanduser("~/.appstoreconnect/jubarte.json")))["issuer_id"])')}"
fi
if [ -z "$ASC_KEY_ID" ] || [ -z "$ASC_ISSUER" ]; then
  echo "ERROR: set ASC_KEY_ID/ASC_ISSUER_ID or create $CRED_FILE" >&2
  exit 1
fi
TARGET="aarch64-apple-darwin"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ENTITLEMENTS="$ROOT/src-tauri/entitlements.plist"
PROFILE="$ROOT/src-tauri/embedded.provisionprofile"
PKG_OUT="$ROOT/dist/Jubarte.pkg"

say() { printf '\n\033[1;34m▸ %s\033[0m\n' "$*"; }

VERSION="$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed -E 's/.*"version": *"([^"]+)".*/\1/')"
say "Publishing Jubarte v$VERSION ($TARGET)"
echo "  Reminder: Apple rejects a re-used version. Bump it if v$VERSION was uploaded before."

# ── 1. build (compiles Rust + the Swift StoreKit lib, bundles + signs the .app) ─
say "1/6  cargo tauri build"
( cd src-tauri && cargo tauri build --target "$TARGET" --bundles app )

# ── 2. locate the bundled .app (respect CARGO_TARGET_DIR if set) ───────────────
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/src-tauri/target}"
APP="$TARGET_DIR/$TARGET/release/bundle/macos/Jubarte.app"
[ -d "$APP" ] || { echo "ERROR: built app not found at $APP"; exit 1; }
say "2/6  Built: $APP"

# ── 3. embed the provisioning profile, fix perms, re-sign ──────────────────────
# Tauri does not embed the profile, and it ships mode 600 (owner-only) which
# Apple rejects with error 90255 — so copy it in, make the bundle all-readable,
# then re-sign with the full MAS entitlements (which include
# com.apple.application-identifier, required for StoreKit — error 90886 without it).
say "3/6  Embed profile + fix perms + re-sign"
cp "$PROFILE" "$APP/Contents/embedded.provisionprofile"
chmod -R a+rX "$APP"
# Prefer explicit re-sign of the app bundle without --deep: --deep is deprecated
# for distribution signing and can mis-apply entitlements to nested code
# (CodeRabbit #3584903488 / Apple codesign guidance). Nested helpers, if any,
# must be signed individually before the outer bundle.
codesign --sign "$SIGN_APP" \
  --entitlements "$ENTITLEMENTS" \
  --options runtime --force --timestamp \
  "$APP"

# ── 4. verify ──────────────────────────────────────────────────────────────────
say "4/6  Verify signature + entitlements"
codesign --verify --deep --strict --verbose=2 "$APP"
if ! codesign -d --entitlements :- "$APP" 2>/dev/null | grep -q "application-identifier"; then
  echo "ERROR: signed app is missing com.apple.application-identifier"; exit 1
fi
if find "$APP" -type f ! -perm -004 | grep -q .; then
  echo "ERROR: some files are not world-readable (would fail upload 90255)"; exit 1
fi

# ── 5. build the signed installer .pkg ─────────────────────────────────────────
say "5/6  productbuild → $PKG_OUT"
mkdir -p "$(dirname "$PKG_OUT")"
rm -f "$PKG_OUT"
productbuild --component "$APP" /Applications --sign "$SIGN_PKG" "$PKG_OUT"
pkgutil --check-signature "$PKG_OUT" | head -3

# ── 6. upload to App Store Connect ─────────────────────────────────────────────
say "6/6  Upload to App Store Connect"
xcrun altool --upload-app -f "$PKG_OUT" -t macos \
  --apiKey "$ASC_KEY_ID" --apiIssuer "$ASC_ISSUER"

say "Done. Build v$VERSION uploaded."
cat <<EOF

Next (in App Store Connect — https://appstoreconnect.apple.com/apps/$APP_APPLE_ID):
  1. Wait for processing to finish (usually a few minutes). Check with:
       ./scripts/asc-build-status.sh
  2. Open the version in "Prepare for Submission" → Build → "+" → pick this build.
  3. Confirm the "Jubarte Annual" subscription is attached to the version.
  4. Fill/verify metadata, then click "Add for Review" / "Submit for Review".
EOF
