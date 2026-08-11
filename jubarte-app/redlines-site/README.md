<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# redlines-site

Public web front end for the Jubarte redline engine: drop two `.docx` files, get
a real Word tracked-changes redline. Five free redlines per visitor, then a
"contact us" panel.

Deployed as a Cloudflare Worker named **`jubarte-redlines-site`**. Self-contained
— it shares nothing with the Tauri app in `../src` or with `../verify-worker`,
apart from building its wasm from the same `jubarte-wasm` crate.

> **Not to be confused with `../jubarte-site`**, which is the jubarte.pro
> marketing Worker (Worker name `jubarte-site`, route `jubarte.pro/*`). They are
> separate Workers on separate domains and must keep separate `name` values in
> `wrangler.jsonc` — deploying one under the other's name replaces it.

```
redlines-site/
  src/index.ts          Worker: /about, /api/quota, /api/redline
  src/quota.ts          free-redline accounting (pure D1, no HTTP)
  public/               the static site
    index.html          page
    privacy.html        /privacy
    terms.html          /terms
    styles.css          arthur.law design tokens
    app.js              UI + orchestration
    redline-worker.js   Web Worker that runs the wasm
    og.png favicon.*    generated — `bun run render-og`
    robots.txt sitemap.xml
    vendor/             wasm bundle (tracked; regenerate + commit — see "Building")
  assets/               SVG + font sources for the generated images
  migrations/           D1 schema
  test/                 vitest + @cloudflare/vitest-pool-workers
```

## Why the compare is client-side

The redline runs as WebAssembly **in the visitor's browser**, not in the Worker.
Two reasons, in order of weight:

1. **Privacy.** These are contracts. Documents never leave the machine, and that
   is a claim worth being able to make literally.
2. **It could not work otherwise on the Free plan.** Workers Free caps CPU at
   **10 ms per request**; a DOCX compare is ~250 ms for a small pair and seconds
   for a real one. Server-side compare would require Workers Paid, plus staying
   under the 128 MB per-isolate limit for large documents.

The consequence, stated plainly: **the five-redline limit is a funnel gate, not
an entitlement boundary.** Someone who opens devtools can call the wasm directly
or clear the `jid` cookie for a fresh allowance. That is an accepted trade — this
gate exists to start a conversation, not to protect revenue. The real
entitlement check, against Apple-signed receipts, lives in `../verify-worker`.

If it ever needs to be enforceable, the change is contained: move
`compareDocuments` into the Worker behind `/api/redline`, on a Paid plan. The
quota layer (`src/quota.ts`) already returns the right answer either way.

## How a redline flows

1. Page load → `GET /api/quota`. Mints an opaque `jid` cookie (HttpOnly, Secure,
   SameSite=Lax, 1 year) if absent. Does **not** create a D1 row — most visitors
   never run anything, and a bounce shouldn't cost a write.
2. Two `.docx` files dropped → the wasm starts downloading in the background so
   the button doesn't wait on ~2 MB.
3. **Compare** runs in `redline-worker.js`, off the main thread.
4. **Then** `POST /api/redline` charges one. The download link is armed only on
   `200`; on `402` the contact section becomes the paywall and no file is handed
   over.

Charging after a successful compare means our bugs never cost a visitor one of
their five, and `used` counts redlines that actually happened.

## ABOUT

`/about` is a `302` to `ABOUT_URL` (`https://arthur.law`) — handled by the Worker,
not a page in this app.

## Custom domains

All four are declared in `wrangler.jsonc` as `custom_domain` routes and all serve
the same site:

| Domain                   | Zone         |
| ------------------------ | ------------ |
| `redlines.free`          | redlines.free |
| `www.redlines.free`      | redlines.free |
| `redlines.jubarte.pro`   | jubarte.pro   |
| `redlines.arthur.law`    | arthur.law    |

Each zone is already in this Cloudflare account, so `wrangler deploy` creates the
proxied DNS record and provisions the cert — nothing to add by hand.

**`redlines.free` is the canonical host.** Every page carries
`<link rel="canonical">` pointing at it, and `sitemap.xml` lists only that host,
so the four domains consolidate into one set of search signals instead of
competing as duplicates.

There is intentionally **no www→apex redirect in the Worker**: the asset server
answers page requests before the Worker runs (see `run_worker_first`), so a
Worker-side redirect would fire on `/about` and `/api/*` but not on `/`. If you
want one, add a zone-level **Redirect Rule** on `redlines.free` —
`http.host eq "www.redlines.free"` → `https://redlines.free${http.request.uri.path}`,
301. That runs before the cache, costs nothing, and applies to every path.

## Legal pages

`/privacy` and `/terms` are static pages carried over from
[jubarte.pro](https://jubarte.pro/privacy), **adapted for the web version**. The
differences are deliberate and worth a lawyer's eye before launch:

- The app policy says documents stay "on your Mac"; the web policy says they stay
  in your browser, and explains the WebAssembly reason.
- The web version collects something the app does not: the `jid` cookie and the
  free-redline count. The privacy policy describes that record explicitly (opaque
  UUID, a count, two timestamps) — it has to, or the policy is inaccurate.
- Google Fonts is named as a service provider, since requesting Manrope discloses
  the visitor's IP to Google. Self-hosting Manrope the way Departure Mono is
  self-hosted would remove that disclosure entirely.
- Subscription/App Store/EULA clauses are dropped from the web terms (nothing is
  sold here) and replaced with a free-use section.

## SEO

- Canonical host on every page; `robots.txt` disallows `/api/`; `sitemap.xml`
  lists the three indexable pages.
- Open Graph + Twitter `summary_large_image` on all pages, pointing at
  `/og.png` (1200×630).
- JSON-LD `@graph` on the front page: `WebApplication` (with a free `Offer`),
  `Organization`, and `FAQPage`.
- **The `FAQPage` schema mirrors the visible `#faq` section verbatim.** If you
  edit one, edit the other — schema describing content the page does not show is
  a quality problem, not a ranking trick.

Regenerate the social card and favicons after editing `assets/og.svg` or
`assets/icon.svg`:

```sh
bun run render-og
```

Rendering loads `assets/*.otf` / `*.ttf` explicitly, so the card is identical on
every machine instead of substituting whatever fonts happen to be installed.
Both bundled families are OFL-1.1 (Departure Mono © Helena Zhang; Manrope © The
Manrope Project Authors), with REUSE `.license` sidecars.

## Building

The wasm bundle in `public/vendor/` **is committed**, which is unusual for build
output and deliberate: the crate it is built from (`jubarte-wasm`) lives in the
parent `jubarte-redlines` checkout, *not* in this repository. Ignoring it would
leave a fresh clone of `jubarte-app` unable to build or deploy the site at all.

To refresh it after an engine change — from inside a full `jubarte-redlines`
checkout, since that is where the crate is:

```sh
bun install
bun run sync-wasm        # wasm-pack build --release --target web  →  public/vendor/
```

Needs `wasm-pack` and `binaryen` (`wasm-opt`) on PATH. `bun run deploy` runs
`sync-wasm` first; outside the monorepo it exits with a clear message and you
deploy the committed bundle instead. **Commit the regenerated bundle** alongside
whatever engine change caused it, or the site silently keeps shipping the old
engine.

`DepartureMono.woff2` in `public/vendor/` is a design asset, not build output.

## Local development

```sh
bun run db:migrate:local     # once, creates the local D1 schema
bun run dev                  # http://localhost:8788
```

Reset your own allowance while testing by clearing the `jid` cookie, or:

```sh
wrangler d1 execute jubarte-redlines-quota --local --command "DELETE FROM quota"
```

## First deploy

`wrangler.jsonc` ships a placeholder `database_id`. Create the database and paste
the real id in before deploying:

```sh
wrangler d1 create jubarte-redlines-quota
# → copy database_id into wrangler.jsonc
bun run db:migrate:remote
bun run deploy
```

The quota database is deliberately **separate** from verify-worker's
`jubarte-entitlements`. An anonymous marketing counter should not share a blast
radius with paid entitlement state.

## Checks

```sh
bun run test         # vitest (D1-backed, real Worker runtime)
bun run typecheck    # needs `bun run cf-typegen` once — worker-configuration.d.ts is gitignored
bun run lint
```

## Configuration

| Var             | Default                | Meaning                              |
| --------------- | ---------------------- | ------------------------------------ |
| `FREE_LIMIT`    | `5`                    | Free redlines before the paywall     |
| `ABOUT_URL`     | `https://arthur.law`   | Where `/about` redirects             |
| `CONTACT_EMAIL` | `contact@arthur.law`   | Shown in the 402 payload             |

A non-numeric or non-positive `FREE_LIMIT` falls back to `5` rather than handing
out unlimited redlines.

## Design

Visual language is carried over from `arthur.law` (the `1-arthur-astro`
checkout): warm off-white `#FAFAF8`, one near-black ink `#0A0A0A`, and a single
highlighter yellow `#FFD400` used **only as a background mark**, never as text
colour. Labels are uppercase Departure Mono; body is Manrope. The contact
section is the practice site's contact block, and doubles as the paywall.
