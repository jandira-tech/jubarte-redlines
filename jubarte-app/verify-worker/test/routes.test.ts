import { env } from "cloudflare:test";
import { beforeEach, describe, expect, it } from "vitest";
import { getSubscription, upsertSubscription } from "../src/db";
import type { SubscriptionState } from "../src/entitlement";
import app from "../src/index";

const NOW = Date.now();
const DAY = 86_400_000;

function baseState(o: Partial<SubscriptionState> = {}): SubscriptionState {
  return {
    originalTransactionId: "2000000123",
    productId: "com.jandira.jubarte.annual",
    environment: "Production",
    expiresDateMs: NOW + 30 * DAY,
    gracePeriodExpiresDateMs: null,
    revocationDateMs: null,
    autoRenewStatus: true,
    latestTransactionId: "2000000999",
    signedDateMs: NOW,
    ...o,
  };
}

beforeEach(async () => {
  await env.DB.exec("DELETE FROM subscriptions");
});

describe("db layer (real D1)", () => {
  it("round-trips a subscription", async () => {
    const s = baseState();
    await upsertSubscription(env.DB, s, NOW);
    expect(await getSubscription(env.DB, s.originalTransactionId)).toEqual(s);
  });

  it("applies a newer (later signed_date) update", async () => {
    await upsertSubscription(
      env.DB,
      baseState({ signedDateMs: NOW - DAY, expiresDateMs: NOW + DAY }),
      NOW,
    );
    await upsertSubscription(
      env.DB,
      baseState({ signedDateMs: NOW, expiresDateMs: NOW + 400 * DAY }),
      NOW,
    );
    const got = await getSubscription(env.DB, "2000000123");
    expect(got?.expiresDateMs).toBe(NOW + 400 * DAY);
  });

  it("ignores a stale (older signed_date) update — cannot un-revoke a refund", async () => {
    await upsertSubscription(
      env.DB,
      baseState({ signedDateMs: NOW, revocationDateMs: NOW - DAY }),
      NOW,
    );
    await upsertSubscription(
      env.DB,
      baseState({ signedDateMs: NOW - 10 * DAY, revocationDateMs: null }),
      NOW,
    );
    const got = await getSubscription(env.DB, "2000000123");
    expect(got?.revocationDateMs).toBe(NOW - DAY);
  });
});

describe("routes (real D1)", () => {
  it("GET /entitlement is unauthorized without ownership proof", async () => {
    const res = await app.request("/entitlement/2000000123", {}, env);
    expect(res.status).toBe(401);
    expect(await res.json()).toMatchObject({ error: "unauthorized" });
  });

  it("POST /verify -> 400 when signedTransaction is missing", async () => {
    const res = await app.request(
      "/verify",
      { method: "POST", body: "{}", headers: { "content-type": "application/json" } },
      env,
    );
    expect(res.status).toBe(400);
  });

  it("POST /verify -> 400 when signedTransaction is not a string", async () => {
    const res = await app.request(
      "/verify",
      {
        method: "POST",
        body: JSON.stringify({ signedTransaction: { not: "a string" } }),
        headers: { "content-type": "application/json" },
      },
      env,
    );
    expect(res.status).toBe(400);
  });

  it("POST /notifications -> 400 when signedPayload is missing", async () => {
    const res = await app.request(
      "/notifications",
      { method: "POST", body: "{}", headers: { "content-type": "application/json" } },
      env,
    );
    expect(res.status).toBe(400);
  });

  it("POST /notifications -> 400 when signedPayload is not a string", async () => {
    const res = await app.request(
      "/notifications",
      {
        method: "POST",
        body: JSON.stringify({ signedPayload: 42 }),
        headers: { "content-type": "application/json" },
      },
      env,
    );
    expect(res.status).toBe(400);
  });

  it("sets CORS allow-origin to the Tauri webview origin", async () => {
    const res = await app.request(
      "/verify",
      {
        method: "OPTIONS",
        headers: {
          Origin: "tauri://localhost",
          "Access-Control-Request-Method": "POST",
          "Access-Control-Request-Headers": "content-type",
        },
      },
      env,
    );
    // Exact origin (not `*`) — CR #3583948708.
    expect(res.headers.get("access-control-allow-origin")).toBe("tauri://localhost");
  });

  it("does not reflect a disallowed Origin as *", async () => {
    const res = await app.request(
      "/verify",
      {
        method: "OPTIONS",
        headers: {
          Origin: "https://evil.example",
          "Access-Control-Request-Method": "POST",
        },
      },
      env,
    );
    const acao = res.headers.get("access-control-allow-origin");
    // Hono's default cors() reflects the request origin; pin the contract we
    // care about — never open with a wildcard.
    expect(acao).not.toBe("*");
  });
});
