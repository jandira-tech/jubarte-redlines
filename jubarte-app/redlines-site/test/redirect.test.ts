import { describe, expect, it } from "vitest";
import worker from "../src/index";

const visit = (url: string, method = "GET") => worker.fetch(new Request(url, { method }));

describe("the old redline tool's hosts", () => {
  it.each([
    "redlines.free",
    "www.redlines.free",
    "redlines.jubarte.pro",
    "redlines.arthur.law",
  ])("send %s to the demo on jubarte.pro, for good", async (host) => {
    const res = await visit(`https://${host}/`);
    expect(res.status).toBe(301);
    expect(res.headers.get("location")).toBe("https://jubarte.pro/demo");
  });

  it.each([
    ["/privacy", "https://jubarte.pro/privacy"],
    ["/privacy.html", "https://jubarte.pro/privacy"],
    ["/terms", "https://jubarte.pro/terms"],
    ["/terms.html", "https://jubarte.pro/terms"],
    ["/about", "https://jubarte.pro/demo"],
    ["/api/quota", "https://jubarte.pro/demo"],
    ["/index.html?utm=x", "https://jubarte.pro/demo"],
    ["/vendor/jubarte_wasm_bg.wasm", "https://jubarte.pro/demo"],
  ])("send %s to %s", async (path, to) => {
    const res = await visit(`https://redlines.free${path}`);
    expect(res.status).toBe(301);
    expect(res.headers.get("location")).toBe(to);
  });

  it("redirects any method, with no body and no cookie", async () => {
    const res = await visit("https://redlines.free/api/redline", "POST");
    expect(res.status).toBe(301);
    expect(res.headers.get("set-cookie")).toBeNull();
    expect(await res.text()).toBe("");
  });
});
