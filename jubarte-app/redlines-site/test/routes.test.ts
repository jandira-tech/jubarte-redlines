import { env } from "cloudflare:test";
import { beforeEach, describe, expect, it } from "vitest";
import app from "../src/index";

type QuotaBody = { used: number; limit: number; remaining: number; paywalled: boolean };

/** Pull the `jid` value out of a Set-Cookie header, or null if none was set. */
function jidFrom(res: Response): string | null {
  const raw = res.headers.get("set-cookie");
  return raw?.match(/(?:^|,\s*)jid=([^;]+)/)?.[1] ?? null;
}

/** Every Set-Cookie line on the response (Hono sets one per call). */
function cookieLines(res: Response): string[] {
  return res.headers.getSetCookie?.() ?? [res.headers.get("set-cookie") ?? ""];
}

/**
 * Stable, well-formed visitor id for a short test label. The Worker only
 * accepts the UUID shape it issues, so fixtures must look like the real thing —
 * a bare "v1" is (correctly) discarded and replaced with a fresh id.
 */
function uuidFor(label: string): string {
  const hex = [...label]
    .reduce((h, ch) => h + ch.charCodeAt(0).toString(16), "")
    .padEnd(32, "0");
  const h = hex.slice(0, 32);
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

async function req(path: string, init: RequestInit & { jid?: string } = {}) {
  const { jid, ...rest } = init;
  const headers = new Headers(rest.headers);
  if (jid) headers.set("Cookie", `jid=${uuidFor(jid)}`);
  return app.fetch(new Request(`https://jubarte.test${path}`, { ...rest, headers }), env);
}

beforeEach(async () => {
  await env.DB.exec("DELETE FROM quota");
});

describe("GET /about", () => {
  it("redirects to arthur.law", async () => {
    const res = await req("/about");
    expect(res.status).toBe(302);
    expect(res.headers.get("location")).toBe("https://arthur.law");
  });

  it("does not consume quota", async () => {
    await req("/about", { jid: "v1" });
    expect(await req("/api/quota", { jid: "v1" }).then((r) => r.json())).toMatchObject({
      used: 0,
    });
  });
});

describe("GET /api/quota", () => {
  it("mints a jid cookie for a first-time visitor", async () => {
    const res = await req("/api/quota");
    expect(res.status).toBe(200);

    const jid = jidFrom(res);
    expect(jid).toBeTruthy();
    expect(await res.json()).toEqual({ used: 0, limit: 5, remaining: 5, paywalled: false });
  });

  it("marks the cookie HttpOnly, Secure, SameSite=Lax and long-lived", async () => {
    const line = cookieLines(await req("/api/quota")).find((c) => c.startsWith("jid="));
    expect(line).toMatch(/HttpOnly/i);
    expect(line).toMatch(/Secure/i);
    expect(line).toMatch(/SameSite=Lax/i);
    expect(line).toMatch(/Path=\//i);
    expect(line).toMatch(/Max-Age=\d{7,}/i);
  });

  it("reuses an existing jid instead of reissuing", async () => {
    await req("/api/redline", { method: "POST", jid: "v1" });
    const res = await req("/api/quota", { jid: "v1" });

    expect(jidFrom(res)).toBeNull();
    expect(await res.json()).toMatchObject({ used: 1, remaining: 4 });
  });

  it("ignores a malformed jid and issues a clean one", async () => {
    // A hand-edited cookie must not become a D1 primary key.
    const res = await req("/api/quota", {
      headers: { Cookie: "jid=not-a-uuid-DROP-TABLE-quota" },
    });
    expect(jidFrom(res)).toMatch(/^[0-9a-f-]{36}$/);
  });
});

describe("POST /api/redline", () => {
  it("allows the first five and reports the running count", async () => {
    const seen: QuotaBody[] = [];
    for (let i = 0; i < 5; i++) {
      const res = await req("/api/redline", { method: "POST", jid: "v1" });
      expect(res.status).toBe(200);
      seen.push(await res.json<QuotaBody>());
    }
    expect(seen.map((s) => s.remaining)).toEqual([4, 3, 2, 1, 0]);
  });

  it("answers the sixth with 402 and the paywall payload", async () => {
    for (let i = 0; i < 5; i++) await req("/api/redline", { method: "POST", jid: "v1" });

    const res = await req("/api/redline", { method: "POST", jid: "v1" });
    expect(res.status).toBe(402);
    expect(await res.json()).toMatchObject({
      paywalled: true,
      used: 5,
      remaining: 0,
      contactEmail: "contact@arthur.law",
    });
  });

  it("mints a cookie for a visitor who arrives straight at the endpoint", async () => {
    const res = await req("/api/redline", { method: "POST" });
    expect(jidFrom(res)).toBeTruthy();
    expect(await res.json()).toMatchObject({ used: 1 });
  });

  it("rejects GET", async () => {
    const res = await req("/api/redline");
    expect(res.status).toBe(404);
  });
});

describe("static pages", () => {
  // These are plain assets, but they are load-bearing: the site makes a privacy
  // claim on its front page, and a 404 on /privacy turns that into a bad look.
  it.each([
    ["/", "Drop two Word documents"],
    ["/privacy", "Privacy Policy"],
    ["/terms", "Terms of Use"],
  ])("serves %s", async (path, marker) => {
    const res = await req(path);
    expect(res.status).toBe(200);
    expect(await res.text()).toContain(marker);
  });

  it("keeps /about a redirect rather than a page", async () => {
    // /about is the one nav item the Worker owns; if an about.html ever lands in
    // public/ it would shadow the redirect and quietly strand arthur.law.
    const res = await req("/about");
    expect(res.status).toBe(302);
  });
});

describe("hardening", () => {
  it("never reflects an arbitrary Origin back", async () => {
    // This site is same-origin; a permissive ACAO would let any page burn a
    // visitor's allowance from the background.
    const res = await req("/api/redline", {
      method: "POST",
      headers: { Origin: "https://evil.example" },
    });
    expect(res.headers.get("access-control-allow-origin")).toBeNull();
  });

  it("does not leak the visitor id in the JSON body", async () => {
    const body = await req("/api/quota", { jid: "v1" }).then((r) => r.text());
    expect(body).not.toContain(uuidFor("v1"));
  });
});
