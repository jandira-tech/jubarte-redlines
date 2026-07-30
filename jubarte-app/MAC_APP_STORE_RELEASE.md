# Publishing Jubarte to the Mac App Store

This document captures the full, repeatable process for building, signing, and
submitting Jubarte to the Mac App Store as a $99.99/year auto-renewable
subscription app. Follow these steps for every new version release.

## Quick publish (the whole thing, repeatable)

Everything is automated by two scripts. To ship a new build:

```bash
# 1. Bump the version — Apple rejects a re-used version number
npm run bump          # bumps package.json + src-tauri/tauri.conf.json

# 2. Build → sign → package → upload, one command
npm run publish:mac   # ./scripts/publish-mac-app-store.sh

# 3. Wait a few minutes for Apple to finish processing, then check
npm run asc:status    # PROCESSING → VALID means it's ready to attach
```

`publish:mac` runs the full pipeline: compile Rust + the Swift StoreKit lib →
bundle + sign the `.app` → embed the provisioning profile, make every file
all-readable, re-sign with the MAS entitlements → build the signed `.pkg` →
upload with `altool`. It **stops at the upload** — the final "Submit for Review"
is a manual click (see [After the upload](#after-the-upload--finish-in-app-store-connect)).
If a required cert/key is missing the script fails early and says which.

The detailed manual steps behind those scripts are in
[Step 1–5](#step-1--build-the-signed-app) below, kept for reference and
debugging.

## After the upload — finish in App Store Connect

Once `npm run asc:status` shows the new build as **VALID**, open
[App Store Connect](https://appstoreconnect.apple.com/apps/6790926615):

1. **Attach the build** — macOS version under "Prepare for Submission" →
   **Build** → **＋** → pick the newly-processed build.
2. **Attach the subscription** — under "In-App Purchases and Subscriptions",
   confirm **Jubarte Pro Yearly** (`com.jandira.jubarte.pro.yearly`) is attached. Upload
   its review screenshot if it still says "Missing Metadata".
3. **Fill/verify metadata** — description, keywords, screenshots (1280×800 or
   1440×900), support/marketing URLs, **Privacy Policy URL**
   (`https://jubarte.pro/privacy` once the domain's nameservers are live at
   Cloudflare), App Privacy answers, pricing/availability, age rating, copyright.
4. **Submit** — click **Add for Review** / **Submit for Review**.

One-time (already done, re-check only if you move the worker): App Information →
App Store Server Notifications → Production + Sandbox URLs both set to
`https://jubarte.pro/notifications`.

## One-time setup (already done, kept here for reference)

- **App Store Connect app record**: "Jubarte", Apple ID `6790926615`,
  bundle ID `com.jandira.jubarte`, SKU `jubarte-mac-2026`.
- **Team**: Jandira Technologies, LLC — Team ID `NW99N2W6TA`.
- **App Store Connect API key**: Key ID `D6P299RB86`, Issuer ID
  `04eda013-ed0d-4a50-8bf8-d39149ce7aa6`, Role: Admin.
  Key file lives at `~/Downloads/AuthKey_D6P299RB86.p8` — back this up
  somewhere safe, Apple only lets you download it once.
- **Signing certificate**: "Apple Distribution: Jandira Technologies, LLC
  (NW99N2W6TA)" (unified cert covering Mac App Store + iOS), already in
  the login keychain, expires 2027-04-10.
- **Installer certificate**: "3rd Party Mac Developer Installer: Jandira
  Technologies, LLC (NW99N2W6TA)", expires 2027-04-10.
- **Provisioning profile**: "Jubarte Mac App Store Profile" for
  `com.jandira.jubarte`, type "Mac App Store", stored at
  `src-tauri/embedded.provisionprofile` in the repo and also installed at
  `~/Library/MobileDevice/Provisioning Profiles/`. Expires 2027-04-10.
- **Subscription**: Group "Jubarte Pro" (id 22237556), subscription
  "Jubarte Pro Yearly" (Apple ID 6791004310; the original "Jubarte Annual"
  6790927274 was deleted — deleted IAP product ids can never be reused), product ID
  `com.jandira.jubarte.pro.yearly`, price tier $99.99/year (Apple has no exact
  $99.00 tier; $99.99 is the nearest).

## Files added to the repo for App Store distribution

- `src-tauri/entitlements.plist` — App Sandbox entitlements:
  - `com.apple.security.app-sandbox`
  - `com.apple.security.files.user-selected.read-write`
  - `com.apple.security.files.bookmarks.app-scope`
- `src-tauri/embedded.provisionprofile` — the Mac App Store provisioning
  profile (re-download from developer.apple.com if it expires).
- `src-tauri/tauri.conf.json` — `bundle.macOS.signingIdentity` set to
  `"Apple Distribution: Jandira Technologies, LLC (NW99N2W6TA)"` and
  `bundle.macOS.entitlements` set to `"entitlements.plist"`. Bundle
  target is `["app"]` only (no dmg) since the Store needs a `.pkg`, not
  a `.dmg`, and the `.pkg` is produced by a separate `productbuild` step
  below, not by Tauri directly.

## Step 1 — Build the signed .app

```bash
cd /Users/arthrod/temp/T/ooxmlsdk/jubarte-app/src-tauri
cargo-tauri build --target aarch64-apple-darwin --bundles app
```

This produces:
`src-tauri/target/release/bundle/macos/Jubarte.app`

It will already be signed with the Apple Distribution certificate and
carry the App Sandbox entitlements (Tauri reads `signingIdentity` and
`entitlements` straight from `tauri.conf.json`). Verify with:

```bash
codesign -dvv --entitlements - target/release/bundle/macos/Jubarte.app
```

You should see `Authority=Apple Distribution: Jandira Technologies, LLC
(NW99N2W6TA)` and the three sandbox entitlement keys.

## Step 2 — Embed the provisioning profile and re-sign

Tauri's bundler does not embed `embedded.provisionprofile` automatically,
so this has to be done manually after every build:

```bash
cp src-tauri/embedded.provisionprofile \
   src-tauri/target/release/bundle/macos/Jubarte.app/Contents/embedded.provisionprofile

codesign --sign "Apple Distribution: Jandira Technologies, LLC (NW99N2W6TA)" \
  --entitlements src-tauri/entitlements.plist \
  --options runtime --force --deep --timestamp \
  src-tauri/target/release/bundle/macos/Jubarte.app
```

Verify again:

```bash
codesign --verify --deep --strict --verbose=2 \
  src-tauri/target/release/bundle/macos/Jubarte.app
# -> "valid on disk" / "satisfies its Designated Requirement"
```

## Step 3 — Build the signed installer .pkg

The Mac App Store only accepts `.pkg` installers, built with
`productbuild`, signed with the *Installer* certificate (not the
Distribution/Application cert used for the .app itself):

```bash
mkdir -p /tmp/jubarte-pkg-out
productbuild --component src-tauri/target/release/bundle/macos/Jubarte.app /Applications \
  --sign "3rd Party Mac Developer Installer: Jandira Technologies, LLC (NW99N2W6TA)" \
  /tmp/jubarte-pkg-out/Jubarte.pkg
```

Sanity check the signature:

```bash
pkgutil --check-signature /tmp/jubarte-pkg-out/Jubarte.pkg
```

## Step 4 — Bump the version before every new submission

Apple rejects re-uploading the same build/version number. Bump both:

- `src-tauri/tauri.conf.json` → `"version"`
- `package.json` → `"version"`

(there's already a `bun scripts/bump-version.mjs` helper — `npm run bump`
— check it keeps both files in sync before relying on it blindly).

## Step 5 — Upload to App Store Connect

Make sure the API key is where `altool` expects it:

```bash
mkdir -p ~/.appstoreconnect/private_keys
cp ~/Downloads/AuthKey_D6P299RB86.p8 ~/.appstoreconnect/private_keys/
```

Then upload:

```bash
xcrun altool --upload-app \
  -f /tmp/jubarte-pkg-out/Jubarte.pkg \
  -t macos \
  --apiKey D6P299RB86 \
  --apiIssuer 04eda013-ed0d-4a50-8bf8-d39149ce7aa6
```

This can take a few minutes. `altool` is deprecated by Apple in favor of
Transporter.app, but still works as of this writing; if it stops working,
install Transporter from the Mac App Store and drag the `.pkg` in
instead, using the same API key (or your Apple ID) for login.

Processing on Apple's side (virus scan + validation) usually takes
5-30 minutes after upload before the build appears in App Store Connect
under the app's "TestFlight"/build list (macOS apps don't get TestFlight,
but the build still needs to finish processing before you can attach it
to a version in "Prepare for Submission").

## Step 6 — Attach the build and submit for review

In App Store Connect (appstoreconnect.apple.com/apps/6790926615):

1. Go to the "macOS App" version in "Prepare for Submission".
2. Under "Build", click "+" and select the newly processed build.
3. Make sure the subscription ("Jubarte Pro Yearly") is attached under
   "In-App Purchases and Subscriptions" for this version.
4. Fill/verify: description, keywords, screenshots (1280x800 or
   1440x900 recommended for macOS), support URL, marketing URL,
   privacy policy URL, App Privacy questionnaire, pricing/availability,
   age rating, copyright.
5. Click "Add for Review" / "Submit for Review".

## In-app purchase (StoreKit 2) integration

The annual subscription is sold through **StoreKit 2** via a small native Swift
helper linked into the binary — chosen over a pure-Rust StoreKit crate (which
can only reach the legacy Objective-C StoreKit 1 API) and over a webview bridge
(WKWebView exposes no StoreKit to JavaScript at all).

### Where the code lives

- `src-tauri/storekit/` — a **static** Swift Package (`jubarte-storekit`) using
  StoreKit 2. `Sources/jubarte-storekit/StoreKitBridge.swift` exposes four
  `@_cdecl` entry points, each returning a JSON `{"ok": Bool, ...}` string:
  `jubarte_fetch_products`, `jubarte_purchase`, `jubarte_current_entitlement`,
  `jubarte_restore`. StoreKit 2 is async; each entry point bridges to a
  synchronous C call with a `DispatchSemaphore` and is only ever called from a
  Rust background thread.
- `src-tauri/build.rs` — links the Swift package with `swift-rs`'s `SwiftLinker`
  (macOS-only), links the **StoreKit** framework, and adds an rpath to
  `/usr/lib/swift` so `@rpath/libswift_Concurrency.dylib` (pulled in by the
  async Swift) resolves at load time. Without that rpath the binary aborts on
  launch with `dyld: Library not loaded: @rpath/libswift_Concurrency.dylib`.
- `src-tauri/Cargo.toml` — `swift-rs` is a **macОS-only** dependency (normal +
  build), so Windows builds are unaffected.
- `src-tauri/src/storekit.rs` — typed serde DTOs, the `{"ok":...}` envelope
  `decode`, and four `#[tauri::command]`s (`storekit_fetch_products`,
  `storekit_purchase`, `storekit_current_entitlement`, `storekit_restore`) run
  on `spawn_blocking`. On non-macOS the commands return a clear error.
- `src/storekit.js` — front-end `invoke` wrappers (`fetchProducts`, `purchase`,
  `currentEntitlement`, `restore`, `isEntitled`). Not yet wired into a paywall
  UI — import them from an upgrade view when you build it.

`minimumSystemVersion` was raised `10.15 → 12.0` (StoreKit 2 needs macOS 12+),
matched in `Package.swift` (`.macOS(.v12)`) and `SwiftLinker::new("12")`.

### Trust boundary — DO NOT ship without server-side verification

The `active` / `expirationDate` booleans returned to the webview are **client
convenience only**. The authoritative signal is the signed JWS
(`jwsRepresentation`) returned as `jws` on purchase/entitlement results. Before
unlocking paid functionality for real, verify that JWS against Apple's public
keys, and track the annual-renewal lifecycle (renew / cancel / refund / grace
period / billing retry) with **App Store Server Notifications V2** on a backend.
Gating features on the client boolean alone is trivially bypassable.

### Adversarial review — fixed vs. remaining

Two other-corpus/other-lens reviews (crush + a StoreKit-semantics agent) were run
against this code. Addressed in-tree:

- **Grace-period false denial (was a real bug):** entitlement was gated on
  `expirationDate > now`, which wrongly reported `active:false` for a paying
  customer in a billing grace period (StoreKit keeps such a transaction in
  `currentEntitlements` with a past `expirationDate`). Now: presence in
  `currentEntitlements` ⇒ active; `expirationDate` is informational.
- **`com.apple.security.network.client` added** to `entitlements.plist` — without
  it the App Sandbox blocks the outbound POST of the JWS, making server-side
  verification impossible.
- **`Transaction.updates` listener** installed at launch (`start_transaction_listener`
  in `main.rs` setup) — verifies and `finish()`es renewals / Ask-to-Buy approvals
  / other-device purchases / refunds so StoreKit stops re-delivering them.
- **Purchase reentrancy gate** (no double payment sheet) and **timeouts** on the
  non-interactive calls (`fetch_products`, `current_entitlement`) so a wedged
  StoreKit op can't park a blocking-pool thread forever. Interactive calls
  (`purchase`, `restore`) are intentionally untimed.

Remaining before you actually charge (NOT done here — mostly backend/UI work):

- **Server-side verification** of the JWS + **App Store Server Notifications V2**
  — the authoritative entitlement lifecycle. The client booleans are hints only.
- **Move `finish()` to after the backend ACKs** the transaction and keep a
  pending-JWS queue flushed on next launch. Today `purchase` finishes
  optimistically; for an auto-renewable sub this is recoverable (the entitlement
  persists in `currentEntitlements` and can be re-read), but the server-authoritative
  ordering is safer.
- **MAS Guideline 3.1.2 paywall disclosure UI** — before calling `purchase()` the
  UI must show title, price, subscription period, auto-renewal terms, and links to
  Terms of Use (EULA) + Privacy Policy. Submitting without this is a guaranteed
  rejection. `src/storekit.js` intentionally ships no paywall UI yet.
- **Live entitlement push to the webview** — pushing a `Transaction.updates`
  change into JS needs a Swift→Rust callback channel; until then the frontend
  should re-call `currentEntitlement` after a purchase and on window focus.

### Automated vs. manual testing

- **Automated (runs in CI / `cargo test`):** the Rust↔Swift JSON contract —
  `cargo test storekit` covers envelope decode, `ok:false → Err`, malformed
  input, and every DTO field mapping (8 tests). The Swift package's own
  compile+link is covered by `cargo build` and `swift build`.
- **Manual (requires a signed app + StoreKit context):** the actual purchase
  flow cannot be exercised from an unsigned `cargo` binary — StoreKit needs a
  receipt/StoreKit-test context. To test end-to-end:
  1. In Xcode, create a `Jubarte.storekit` StoreKit Configuration file with the
     `com.jandira.jubarte.pro.yearly` product, **or** use a Sandbox Apple ID
     (App Store Connect → Users and Access → Sandbox Testers).
  2. Build and sign the `.app` (Steps 1–2 above), then launch it.
  3. Verify: products load with `$99.99`; purchase presents the sheet;
     `currentEntitlement` reports `active:true` after buying; `restore`
     recovers it on a fresh install; a non-renewing/expired sandbox sub reports
     `active:false`.
  4. Confirm the App-Sandboxed, signed build launches without the dyld
     concurrency-runtime error (`otool -l Jubarte.app/Contents/MacOS/Jubarte |
     grep -A2 LC_RPATH` should show `/usr/lib/swift`).

## Upload errors hit on the v0.3.0 (subscription) build — and their fixes

- **90255 — "installer package includes files only readable by the root user."**
  `embedded.provisionprofile` is mode `600` in the repo, and `cp` preserves it
  into the `.app`, so the bundle ships an owner-only file and altool rejects it.
  Fix: after embedding, `chmod -R a+rX "<app>"` (or at least `chmod 644` the
  profile), THEN re-sign. Check with `find "<app>" -type f ! -perm -004`.
- **90886 — signature missing `application-identifier`.** The signed entitlements
  must include `com.apple.application-identifier` = `NW99N2W6TA.com.jandira.jubarte`
  (and `com.apple.developer.team-identifier`). These are now in
  `src-tauri/entitlements.plist`; without them StoreKit can't resolve the app's
  products at runtime.

## Known gotchas hit during the first release

- `find-identity -v -p codesigning` sometimes reports the Apple
  Distribution identity as invalid ("Missing required extension") even
  though it works perfectly fine for actual signing. Don't trust that
  output — test with a real `codesign --sign` on a throwaway file
  instead of relying on `find-identity`.
- Apple's subscription price tiers do not include an exact $99.00/year
  option; the closest is $99.99/year.
- The subscription's "Missing Metadata" status is expected until a
  review screenshot (recommended size ~1280x800, i.e. a screenshot of
  the in-app upgrade/subscribe screen) is uploaded under the
  subscription's "Review Information".
- App Store Connect's "New App" creation dialog can fail silently (no
  error banner) on the first attempt for no clear reason — the button
  just stops loading and nothing happens. Simply retrying with the same
  values usually succeeds on the second attempt.

## Provisioning profile embedding

`src-tauri/tauri.conf.json` deliberately does **not** list
`bundle.macOS.files["embedded.provisionprofile"]` — the profile is a local secret
(gitignored as `src-tauri/embedded.provisionprofile`). The publish script copies
it into `Jubarte.app/Contents/` after `tauri build` (see step 3 in
`scripts/publish-mac-app-store.sh`).
