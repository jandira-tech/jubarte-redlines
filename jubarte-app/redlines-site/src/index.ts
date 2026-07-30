import { type Context, Hono } from "hono";
import { getCookie, setCookie } from "hono/cookie";
import { consume, peek } from "./quota";

export type Bindings = {
  DB: D1Database;
  ASSETS: Fetcher;
  FREE_LIMIT: string;
  ABOUT_URL: string;
  CONTACT_EMAIL: string;
};

type Ctx = Context<{ Bindings: Bindings }>;

const COOKIE = "jid";
const YEAR_SECONDS = 60 * 60 * 24 * 365;
const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

function freeLimit(env: Bindings): number {
  const n = Number.parseInt(env.FREE_LIMIT, 10);
  // A misconfigured var must not silently hand out unlimited redlines, nor
  // wall off every visitor. Fall back to the documented default.
  return Number.isFinite(n) && n > 0 ? n : 5;
}

/**
 * Resolve the visitor id, minting one if this is a first visit.
 *
 * The cookie is validated against the UUID shape we issue rather than trusted:
 * it is used as a D1 primary key, and an attacker-chosen value could otherwise
 * be an oversized blob or collide with another visitor's row on purpose.
 */
function visitorId(c: Ctx): { id: string; fresh: boolean } {
  const existing = getCookie(c, COOKIE);
  if (existing && UUID_RE.test(existing)) return { id: existing, fresh: false };
  return { id: crypto.randomUUID(), fresh: true };
}

function issueCookie(c: Ctx, id: string) {
  setCookie(c, COOKIE, id, {
    path: "/",
    httpOnly: true,
    secure: true,
    sameSite: "Lax",
    maxAge: YEAR_SECONDS,
  });
}

const app = new Hono<{ Bindings: Bindings }>();

// No CORS middleware, deliberately. The page and the API share an origin, so
// nothing legitimate needs a cross-origin grant — and without one, a third-party
// page cannot spend a visitor's free redlines from the background.

/** ABOUT — sends people to the practice site rather than an in-app page. */
app.get("/about", (c) => c.redirect(c.env.ABOUT_URL, 302));

/** Current standing. Called on page load to render the counter. */
app.get("/api/quota", async (c) => {
  const { id, fresh } = visitorId(c);
  if (fresh) issueCookie(c, id);
  return c.json(await peek(c.env.DB, id, freeLimit(c.env)));
});

/**
 * Charge one redline.
 *
 * The client calls this *before* running the compare and only proceeds on 200.
 * Since the wasm runs in the browser, this is a funnel gate rather than an
 * enforcement boundary — see README.md.
 */
app.post("/api/redline", async (c) => {
  const { id, fresh } = visitorId(c);
  if (fresh) issueCookie(c, id);

  const result = await consume(c.env.DB, id, freeLimit(c.env), Date.now());
  const { allowed, ...body } = result;

  if (!allowed) {
    return c.json(
      { ...body, contactEmail: c.env.CONTACT_EMAIL, aboutUrl: c.env.ABOUT_URL },
      402,
    );
  }
  return c.json(body);
});

// Anything else under /api is a client bug, not a page — answer it as JSON so a
// typo never falls through to the SPA shell and gets parsed as a quota reply.
app.all("/api/*", (c) => c.json({ error: "not found" }, 404));

// Static site. In production Cloudflare's asset server handles these before the
// Worker runs (see `run_worker_first` in wrangler.jsonc); this catch-all keeps
// the Worker correct on its own — under `vitest` and any routing change alike.
app.all("*", (c) => c.env.ASSETS.fetch(c.req.raw));

export default app;
