// The Worker's helpers. They live outside src/index.ts because workerd
// refuses a main module whose named exports are not handlers.

import { THEME_SCRIPT, THEME_SCRIPT_SHA256 } from "./theme";

/** The page limit wrangler.jsonc enforces. The Worker cannot read
 * data/facts.jsonl, so test/node/facts.test.ts holds this, wrangler.jsonc and
 * the site.limit_per_minute fact to one value. */
export const LIMIT_PER_MINUTE: number = 60;

/** Old addresses that still reach their page: the design prototype's file
 * names, and the paths Cases and App had before they became Use cases and PRO. */
export const LEGACY: Record<string, string> = {
  cases: "/use-cases",
  "cases/embed": "/use-cases/embed",
  "cases.html": "/use-cases",
  app: "/pro",
  "app.html": "/pro",
  "Jubarte Home.dc.html": "/",
  "Jubarte Live.dc.html": "/live",
  "Jubarte Demo.dc.html": "/demo",
  "Jubarte Benchmark.dc.html": "/benchmark",
  "Jubarte Cases.dc.html": "/use-cases",
  "Jubarte App.dc.html": "/pro",
  "Jubarte Contact.dc.html": "/contact",
  "Jubarte Download.dc.html": "/download",
};

/** The file name a prototype link asked for; null when the path is not valid percent-encoding. */
export function legacyName(pathname: string): string | null {
  try {
    return decodeURIComponent(pathname.slice(1));
  } catch {
    return null;
  }
}

// Plain http on a local development host stays http: `wrangler dev` serves
// http://127.0.0.1:8787 and has nothing to upgrade to.
const LOCAL_HOST = /^(?:localhost|127\.0\.0\.1|\[::1\])$|\.localhost$/;

/** Where to send a plain-http request; null when it is served as it is. */
export function httpsUpgrade(url: URL): string | null {
  if (url.protocol !== "http:" || LOCAL_HOST.test(url.hostname)) return null;
  const to = new URL(url);
  to.protocol = "https:";
  return to.toString();
}

/** A year of https-only, for this host alone: no subdomains, no preload list. */
export const HSTS: string = "max-age=31536000";

// 'wasm-unsafe-eval' lets the demo compile jubarte-wasm; nothing else is eval'd.
// The one inline script, the theme restore (theme.ts), is allowed by its hash.
// Everything a page loads is first-party: fonts are self-hosted, nothing is
// fetched from another origin (Hugging Face links are plain navigations).
const CSP = [
  "default-src 'self'",
  `script-src 'self' 'wasm-unsafe-eval' 'sha256-${THEME_SCRIPT_SHA256}'`,
  "worker-src 'self' blob:",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob:",
  "font-src 'self'",
  "connect-src 'self'",
  "frame-src 'self' blob:",
  "frame-ancestors 'self'",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'self'",
].join("; ");

export function securityHeaders(headers: Headers): Headers {
  headers.set("content-security-policy", CSP);
  headers.set("x-content-type-options", "nosniff");
  headers.set("referrer-policy", "strict-origin-when-cross-origin");
  headers.set("permissions-policy", "camera=(), microphone=(), geolocation=(), interest-cohort=()");
  headers.set("cross-origin-opener-policy", "same-origin");
  return headers;
}

export function tooManyRequests(): Response {
  const body = `<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Slow down a little — Jubarte</title>
<script>${THEME_SCRIPT}</script>
<link rel="stylesheet" href="/static/site.css"></head>
<body><main class="wrap limit-page">
<p class="eyebrow">/ 429 · too many requests</p>
<h1 class="h1">You are moving faster than the whale.</h1>
<p class="lead">This site serves at most ${LIMIT_PER_MINUTE} pages a minute to each visitor. Wait a minute and reload — nothing you had open is lost.</p>
<p><a class="btn btn-outline" href="/">Back to the home page</a></p>
</main></body></html>`;
  return new Response(body, {
    status: 429,
    headers: securityHeaders(
      new Headers({
        "content-type": "text/html; charset=utf-8",
        "retry-after": "60",
        "cache-control": "no-store",
      }),
    ),
  });
}
