import { env } from "cloudflare:test";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { describeVerificationError } from "../src/apple/verifier";
import { getSubscription, upsertSubscription } from "../src/db";
import { isEntitled, type SubscriptionState } from "../src/entitlement";
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

describe("a verify without renewal info keeps the stored grace period", () => {
  it("does not end a billing grace period that a notification recorded", async () => {
    // A DID_FAIL_TO_RENEW notification: expired, but in grace, renewal info present.
    await upsertSubscription(
      env.DB,
      baseState({
        signedDateMs: NOW - DAY,
        expiresDateMs: NOW - DAY,
        gracePeriodExpiresDateMs: NOW + 10 * DAY,
        autoRenewStatus: true,
        renewalInfoPresent: true,
      }),
      NOW,
    );
    // Then the app calls /verify: the transaction is newer but carries no renewal info.
    await upsertSubscription(
      env.DB,
      baseState({
        signedDateMs: NOW,
        expiresDateMs: NOW - DAY,
        gracePeriodExpiresDateMs: null,
        autoRenewStatus: false,
        renewalInfoPresent: false,
      }),
      NOW,
    );
    const got = await getSubscription(env.DB, "2000000123");
    expect(got?.gracePeriodExpiresDateMs).toBe(NOW + 10 * DAY);
    expect(got?.autoRenewStatus).toBe(true);
    expect(got).not.toBeNull();
    expect(isEntitled(got as SubscriptionState, NOW)).toBe(true);
  });

  it("still applies renewal fields when the update carries them", async () => {
    await upsertSubscription(
      env.DB,
      baseState({ gracePeriodExpiresDateMs: NOW + DAY, signedDateMs: NOW - DAY }),
      NOW,
    );
    await upsertSubscription(
      env.DB,
      baseState({
        gracePeriodExpiresDateMs: null,
        autoRenewStatus: false,
        signedDateMs: NOW,
        renewalInfoPresent: true,
      }),
      NOW,
    );
    const got = await getSubscription(env.DB, "2000000123");
    expect(got?.gracePeriodExpiresDateMs).toBeNull();
    expect(got?.autoRenewStatus).toBe(false);
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

describe("POST /verify rate limit and error detail", () => {
  const post = (e: unknown, ip = "203.0.113.7") =>
    app.request(
      "/verify",
      {
        method: "POST",
        body: JSON.stringify({ signedTransaction: "x" }),
        headers: { "content-type": "application/json", "cf-connecting-ip": ip },
      },
      e as typeof env,
    );

  it("answers 429 when the limiter refuses, keyed on the caller's IP", async () => {
    const keys: string[] = [];
    const limited = {
      ...env,
      VERIFY_LIMITER: {
        limit: async ({ key }: { key: string }) => {
          keys.push(key);
          return { success: false };
        },
      },
    };
    const res = await post(limited);
    expect(res.status).toBe(429);
    expect(keys).toEqual(["203.0.113.7"]);
  });

  it("passes through when the limiter allows (here: verification of a bogus JWS fails, 400)", async () => {
    const allowed = { ...env, VERIFY_LIMITER: { limit: async () => ({ success: true }) } };
    expect((await post(allowed)).status).toBe(400);
  });

  it("works without the binding (local dev, tests)", async () => {
    expect((await post(env)).status).toBe(400);
  });
});

describe("a failed verification is logged, not returned", () => {
  afterEach(() => vi.restoreAllMocks());

  const post = (path: string, body: unknown, e: unknown = env) =>
    app.request(
      path,
      {
        method: "POST",
        body: JSON.stringify(body),
        headers: { "content-type": "application/json" },
      },
      e as typeof env,
    );

  for (const [path, body] of [
    ["/verify", { signedTransaction: "x" }],
    ["/notifications", { signedPayload: "x" }],
  ] as const) {
    it(`${path} answers a bare 400 and logs Apple's reason for both environments`, async () => {
      const logged = vi.spyOn(console, "error").mockImplementation(() => {});
      const res = await post(path, body);
      expect(res.status).toBe(400);
      expect(await res.json()).toEqual({ error: "verification failed" });
      // A token that is not a JWS decodes to null, which Apple's validator
      // reports as VERIFICATION_FAILURE with the TypeError as its cause.
      expect(logged).toHaveBeenCalledWith(
        "verification failed:",
        expect.stringMatching(
          /^production: VERIFICATION_FAILURE \(.+\); sandbox: VERIFICATION_FAILURE \(.+\)$/,
        ),
      );
    });
  }

  it("answers 500, not 400, when the worker itself is misconfigured", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    const res = await post(
      "/verify",
      { signedTransaction: "x" },
      { ...env, APPLE_APP_APPLE_ID: "none" },
    );
    expect(res.status).toBe(500);
  });
});

describe("describeVerificationError", () => {
  it("names the status Apple's exception carries, since its message is empty", async () => {
    const { VerificationException, VerificationStatus } = await import(
      "@apple/app-store-server-library"
    );
    const e = new VerificationException(VerificationStatus.INVALID_ENVIRONMENT);
    expect(String(e)).toBe("Error");
    expect(describeVerificationError(e)).toBe("INVALID_ENVIRONMENT");
    for (const [name, status] of Object.entries(VerificationStatus)) {
      if (typeof status === "number") {
        expect(describeVerificationError(new VerificationException(status))).toBe(name);
      }
    }
  });

  it("adds the underlying cause when there is one", async () => {
    const { VerificationException, VerificationStatus } = await import(
      "@apple/app-store-server-library"
    );
    const e = new VerificationException(
      VerificationStatus.INVALID_CERTIFICATE,
      new Error("chain of 1"),
    );
    expect(describeVerificationError(e)).toBe("INVALID_CERTIFICATE (chain of 1)");
  });

  it("falls back to an ordinary error's message", () => {
    expect(describeVerificationError(new Error("boom"))).toBe("boom");
    expect(describeVerificationError("plain")).toBe("plain");
  });
});
