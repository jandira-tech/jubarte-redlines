import { Hono } from "hono";
import { cors } from "hono/cors";
import { AppleVerifier } from "./apple/verifier";
import { getSubscription, upsertSubscription } from "./db";
import { isEntitled, type SubscriptionState, toSubscriptionState } from "./entitlement";

export type Bindings = {
  DB: D1Database;
  APPLE_BUNDLE_ID: string;
  APPLE_APP_APPLE_ID: string;
  APPLE_ENABLE_ONLINE_CHECKS?: string;
};

function getVerifier(env: Bindings): AppleVerifier {
  // Cheap to construct (config only); avoid isolate-global mutable state
  // (gemini #3583828456).
  const appAppleId = Number.parseInt(env.APPLE_APP_APPLE_ID, 10);
  if (!Number.isFinite(appAppleId)) {
    throw new Error("APPLE_APP_APPLE_ID must be a number");
  }
  if (!env.APPLE_BUNDLE_ID) {
    throw new Error("APPLE_BUNDLE_ID is required");
  }
  return new AppleVerifier({
    bundleId: env.APPLE_BUNDLE_ID,
    appAppleId,
    enableOnlineChecks: env.APPLE_ENABLE_ONLINE_CHECKS === "true",
  });
}

/** Shape returned to the app — never leak internal columns. */
function publicView(state: SubscriptionState, nowMs: number) {
  return {
    entitled: isEntitled(state, nowMs),
    originalTransactionId: state.originalTransactionId,
    productId: state.productId,
    environment: state.environment,
    expiresDateMs: state.expiresDateMs,
    gracePeriodExpiresDateMs: state.gracePeriodExpiresDateMs,
    autoRenewStatus: state.autoRenewStatus,
  };
}

const app = new Hono<{ Bindings: Bindings }>();

// The Tauri webview calls /verify cross-origin (origin is tauri://localhost or
// similar), so it needs CORS. Every endpoint here is safe to expose: /verify
// only accepts an unforgeable Apple-signed JWS, and /notifications is a
// server-to-server webhook where CORS is irrelevant.
// Reflect only Tauri / local dev origins — never `*` (CR #3583948708).
app.use(
  "*",
  cors({
    origin: (origin) => {
      if (!origin) return "";
      if (origin === "tauri://localhost") return origin;
      if (origin === "http://tauri.localhost") return origin;
      if (origin.startsWith("http://localhost:") || origin.startsWith("http://127.0.0.1:")) {
        return origin;
      }
      return "";
    },
  }),
);

app.get("/", (c) => c.text("jubarte verify-worker"));

// The app POSTs a StoreKit-signed transaction (jwsRepresentation from a purchase
// or currentEntitlements). We verify it against Apple, record/refresh the
// subscription, and return the authoritative entitlement.
app.post("/verify", async (c) => {
  const body = (await c.req.json().catch(() => ({}))) as { signedTransaction?: unknown };
  const signed = body.signedTransaction;
  if (typeof signed !== "string" || !signed) {
    return c.json({ error: "missing signedTransaction" }, 400);
  }

  let txn: Awaited<ReturnType<AppleVerifier["verifyTransaction"]>>;
  try {
    txn = await getVerifier(c.env).verifyTransaction(signed);
  } catch (e) {
    return c.json({ error: "verification failed", detail: String(e) }, 400);
  }

  const now = Date.now();
  try {
    const state = toSubscriptionState(txn);
    await upsertSubscription(c.env.DB, state, now);
    // Re-read so the response reflects any newer data already stored from a
    // notification (the DB guard keeps the freshest signed state).
    const current = (await getSubscription(c.env.DB, state.originalTransactionId)) ?? state;
    return c.json(publicView(current, now));
  } catch (e) {
    // DB / internal failures are not client errors (gemini #3583828479).
    return c.json({ error: "processing failed", detail: String(e) }, 500);
  }
});

// Apple App Store Server Notifications V2 webhook. Configure this URL in App
// Store Connect. A non-2xx response makes Apple retry, so only 4xx on a payload
// we genuinely can't process.
app.post("/notifications", async (c) => {
  const body = (await c.req.json().catch(() => ({}))) as { signedPayload?: unknown };
  const signed = body.signedPayload;
  if (typeof signed !== "string" || !signed) {
    return c.json({ error: "missing signedPayload" }, 400);
  }

  const verifier = getVerifier(c.env);
  let payload: Awaited<ReturnType<AppleVerifier["verifyNotification"]>>;
  try {
    payload = await verifier.verifyNotification(signed);
  } catch (e) {
    return c.json({ error: "verification failed", detail: String(e) }, 400);
  }

  const signedTxn = payload.data?.signedTransactionInfo;
  if (!signedTxn) {
    // TEST notifications and app-level events carry no transaction — ack them.
    return c.json({ ok: true, notificationType: payload.notificationType });
  }

  try {
    const txn = await verifier.verifyTransaction(signedTxn);
    const signedRenewal = payload.data?.signedRenewalInfo;
    const renewal = signedRenewal
      ? await verifier.verifyRenewalInfo(signedRenewal)
      : undefined;
    const state = toSubscriptionState(txn, renewal);
    await upsertSubscription(c.env.DB, state, Date.now());
  } catch (e) {
    // Internal failures must be 5xx so Apple retries (gemini #3583828479).
    return c.json({ error: "processing failed", detail: String(e) }, 500);
  }
  return c.json({ ok: true, notificationType: payload.notificationType });
});

// Unauthenticated status-by-id is disabled: originalTransactionId is guessable
// and would leak product/entitlement metadata (CR #3583861670, gemini #3583828487).
// Clients must POST /verify with a signed Apple transaction.
app.get("/entitlement/:originalTransactionId", (c) => {
  return c.json(
    {
      error: "unauthorized",
      detail: "use POST /verify with a signed transaction",
    },
    401,
  );
});

export default app;
