import { env } from "cloudflare:test";
import { describe, expect, it } from "vitest";
import * as main from "../../src/index";
import { LIMIT_PER_MINUTE } from "../../src/site";
import { THEME_SCRIPT_SHA256 } from "../../src/theme";

const worker = main.default;

let ip = 0;
/** Each test gets its own client address, so the per-IP limit does not leak between tests. */
function client(): string {
  ip += 1;
  return `203.0.113.${ip}`;
}

function get(
  path: string,
  from: string,
  init: RequestInit = {},
  origin = "https://jubarte.pro",
): Promise<Response> {
  const headers = new Headers(init.headers);
  headers.set("cf-connecting-ip", from);
  return worker.fetch(new Request(`${origin}${path}`, { ...init, headers }), env);
}

describe("main module", () => {
  it("exports only the handler, as workerd requires", () => {
    expect(Object.keys(main)).toEqual(["default"]);
  });
});

describe("https", () => {
  it("sends a plain-http page request to https, path and query intact", async () => {
    const res = await get("/cases?x=1", client(), { redirect: "manual" }, "http://jubarte.pro");
    expect(res.status).toBe(301);
    expect(res.headers.get("location")).toBe("https://jubarte.pro/cases?x=1");
  });

  it("leaves local development hosts on http", async () => {
    for (const origin of ["http://127.0.0.1:8787", "http://localhost:8787"]) {
      const res = await get("/", client(), {}, origin);
      expect(res.status, origin).toBe(200);
      expect(res.headers.get("strict-transport-security"), origin).toBeNull();
      await res.arrayBuffer();
    }
  });

  it("pins https with HSTS on pages served over https", async () => {
    const res = await get("/", client());
    expect(res.headers.get("strict-transport-security")).toBe("max-age=31536000");
    await res.arrayBuffer();
  });
});

describe("pages", () => {
  it("serves the home page with the security headers", async () => {
    const res = await get("/", client());
    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toContain("text/html");
    const csp = res.headers.get("content-security-policy") ?? "";
    expect(csp).toContain(`script-src 'self' 'wasm-unsafe-eval' 'sha256-${THEME_SCRIPT_SHA256}'`);
    expect(csp).toContain("frame-ancestors 'self'");
    expect(res.headers.get("x-content-type-options")).toBe("nosniff");
    expect(await res.text()).toContain("Without Word.");
  });

  it("serves pages without the .html suffix, including the case viewer embed", async () => {
    const from = client();
    for (const path of [
      "/demo",
      "/benchmark",
      "/use-cases",
      "/use-cases/embed",
      "/pro",
      "/privacy",
    ]) {
      const res = await get(path, from);
      expect(res.status, path).toBe(200);
      await res.arrayBuffer();
    }
  });

  it("answers an unknown page with the 404 page", async () => {
    const res = await get("/no-such-page", client());
    expect(res.status).toBe(404);
    expect(await res.text()).toContain("This page swam off.");
  });

  it("redirects the design prototype's file names", async () => {
    const res = await get("/Jubarte%20Demo.dc.html", client(), { redirect: "manual" });
    expect(res.status).toBe(301);
    expect(res.headers.get("location")).toBe("https://jubarte.pro/demo");
  });

  it("moves the renamed pages to their new paths", async () => {
    for (const [from, to] of [
      ["/cases", "/use-cases"],
      ["/cases/embed", "/use-cases/embed"],
      ["/app", "/pro"],
      ["/Jubarte%20Cases.dc.html", "/use-cases"],
      ["/Jubarte%20App.dc.html", "/pro"],
    ]) {
      const res = await get(from, client(), { redirect: "manual" });
      expect(res.status, from).toBe(301);
      expect(res.headers.get("location"), from).toBe(`https://jubarte.pro${to}`);
    }
  });

  it("redirects only the prototype's own names, not what every object inherits", async () => {
    for (const path of ["/toString", "/constructor", "/__proto__", "/hasOwnProperty"]) {
      const res = await get(path, client(), { redirect: "manual" });
      expect(res.status, path).toBe(404);
      await res.arrayBuffer();
    }
  });

  it("answers a malformed percent-encoded path with the 404 page, not an error", async () => {
    const res = await get("/%E0%A4%A", client());
    expect(res.status).toBe(404);
    await res.arrayBuffer();
  });

  it("refuses anything but GET and HEAD", async () => {
    const res = await get("/", client(), { method: "POST", body: "x" });
    expect(res.status).toBe(405);
    expect(res.headers.get("allow")).toBe("GET, HEAD");
  });
});

describe("rate limit", () => {
  it(`allows ${LIMIT_PER_MINUTE} pages a minute per address, then answers 429`, async () => {
    const from = client();
    for (let i = 0; i < LIMIT_PER_MINUTE; i++) {
      const res = await get("/live", from);
      expect(res.status, `request ${i + 1}`).toBe(200);
      await res.arrayBuffer();
    }
    const limited = await get("/live", from);
    expect(limited.status).toBe(429);
    expect(limited.headers.get("retry-after")).toBe("60");
    expect(limited.headers.get("cache-control")).toBe("no-store");
    expect(limited.headers.get("content-security-policy")).toBeTruthy();
    expect(await limited.text()).toContain("60 pages a minute");

    const other = await get("/live", client());
    expect(other.status).toBe(200);
    await other.arrayBuffer();
  });
});
