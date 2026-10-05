// jubarte.pro — the public site. Every page is a static file in public/; this
// Worker stands in front of the page requests only (see `run_worker_first` in
// wrangler.jsonc) to send plain http to https, rate-limit them, add the
// security headers and redirect the design prototype's file names. Styles,
// scripts, wasm, fonts and case images are served by the asset server without
// waking it.

import { HSTS, httpsUpgrade, LEGACY, legacyName, securityHeaders, tooManyRequests } from "./site";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method !== "GET" && request.method !== "HEAD") {
      return new Response("Method not allowed", { status: 405, headers: { allow: "GET, HEAD" } });
    }
    const url = new URL(request.url);
    // The zone serves http too; a page asked for over http moves to https
    // before it costs a rate-limit slot.
    const upgrade = httpsUpgrade(url);
    if (upgrade !== null) return Response.redirect(upgrade, 301);
    const name = legacyName(url.pathname);
    // Own names only: `in` would also match `toString`, `constructor`, `__proto__`.
    if (name !== null && Object.hasOwn(LEGACY, name)) {
      return Response.redirect(new URL(LEGACY[name], url).toString(), 301);
    }

    const ip = request.headers.get("cf-connecting-ip") ?? "unknown";
    const { success } = await env.PAGE_LIMIT.limit({ key: ip });
    const response = success ? await env.ASSETS.fetch(request) : tooManyRequests();
    const out = new Response(response.body, response);
    securityHeaders(out.headers);
    if (url.protocol === "https:") out.headers.set("strict-transport-security", HSTS);
    return out;
  },
} satisfies ExportedHandler<Env>;
