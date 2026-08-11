// StoreKit 2 front-end helper — thin wrappers over the Tauri commands defined
// in src-tauri/src/storekit.rs (which call the Swift helper in
// src-tauri/storekit/), plus server-side verification against the deployed
// Cloudflare Worker (see verify-worker/). Import from a paywall/upgrade view:
//
//   import { PRODUCT_ID, fetchProducts, purchaseAndVerify,
//            isEntitledVerified, restore } from "./storekit.js";
//
// Tauri v2 maps JS camelCase args to Rust snake_case params, so `productId`
// here reaches the `product_id` parameter on the Rust side.
//
// SECURITY: the raw `currentEntitlement()`/`isEntitled()` results are the
// *client's* self-reported view — UI hints only. `verifyWithBackend()` /
// `isEntitledVerified()` send the signed `jws` to the Worker, which validates
// Apple's signature and is the authoritative source. Gate anything that matters
// on the verified path. (For a locally-run product, no client check is
// tamper-proof; this deters casual bypass and keeps the entitlement lifecycle
// server-tracked. For stronger resistance, move the fetch into Rust.)

// Optional chaining so this module can load in a plain browser (UI prototyping
// / unit tests) without throwing at import time (gemini #3583605127).
const invoke =
  window.__TAURI__?.core?.invoke ||
  ((cmd) => {
    throw new Error(`Tauri core invoke is not available (command: ${cmd})`);
  });

/** The single annual auto-renewable subscription (App Store Connect id).
 * NOTE: the original `com.jandira.jubarte.annual` product was deleted in App
 * Store Connect (deleted IAP ids can never be reused) — the live product is
 * `com.jandira.jubarte.pro.yearly`. A stale id here makes StoreKit return an
 * empty product list and every purchase fail with "product not found". */
export const PRODUCT_ID = "com.jandira.jubarte.pro.yearly";

/** Deployed verification backend (verify-worker/). */
export const VERIFY_BASE = "https://jubarte-verify-worker.cicero-im.workers.dev";

/**
 * Fetch product metadata from the App Store.
 * @param {string[]} [productIds]
 * @returns {Promise<Array<{
 *   id: string, displayName: string, description: string,
 *   displayPrice: string, price: string, isSubscription: boolean,
 *   subscriptionPeriodUnit: string|null, subscriptionPeriodValue: number|null
 * }>>}
 */
export function fetchProducts(productIds = [PRODUCT_ID]) {
  return invoke("storekit_fetch_products", { productIds });
}

/**
 * Start a purchase and present the App Store payment sheet.
 * @param {string} [productId]
 * @returns {Promise<{
 *   status: "purchased"|"pending"|"userCancelled"|"verificationFailed",
 *   productId: string|null, transactionId: string|null,
 *   originalTransactionId: string|null, expirationDate: number|null,
 *   jws: string|null
 * }>}
 */
export function purchase(productId = PRODUCT_ID) {
  return invoke("storekit_purchase", { productId });
}

/**
 * Current entitlement snapshot for a product.
 * @param {string} [productId]
 * @returns {Promise<{
 *   active: boolean, productId: string|null, transactionId: string|null,
 *   expirationDate: number|null, jws: string|null
 * }>}
 */
export function currentEntitlement(productId = PRODUCT_ID) {
  return invoke("storekit_current_entitlement", { productId });
}

/**
 * Restore purchases (AppStore.sync) and report the first active entitlement.
 * @returns {Promise<{ active: boolean, productId: string|null,
 *   transactionId: string|null, expirationDate: number|null, jws: string|null }>}
 */
export function restore() {
  return invoke("storekit_restore");
}

/**
 * Convenience: is there an active entitlement right now? Swallows errors into
 * `false` so callers can gate UI without a try/catch. UI-hint only — see the
 * security note at the top of this file.
 * @param {string} [productId]
 * @returns {Promise<boolean>}
 */
export async function isEntitled(productId = PRODUCT_ID) {
  try {
    const status = await currentEntitlement(productId);
    return Boolean(status && status.active);
  } catch {
    return false;
  }
}

/**
 * Send a StoreKit-signed transaction (`jws`) to the verification backend, which
 * validates Apple's signature, records the subscription, and returns the
 * authoritative entitlement. Throws on a network/validation failure.
 * @param {string} signedTransaction  the `jws` from purchase()/currentEntitlement()
 * @returns {Promise<{
 *   entitled: boolean, originalTransactionId: string, productId: string,
 *   environment: "Sandbox"|"Production", expiresDateMs: number|null,
 *   gracePeriodExpiresDateMs: number|null, autoRenewStatus: boolean
 * }>}
 */
/** Default timeout for backend verify — a stalled Worker must not hang the UI. */
export const VERIFY_TIMEOUT_MS = 15_000;

export async function verifyWithBackend(signedTransaction, { timeoutMs = VERIFY_TIMEOUT_MS } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  let res;
  try {
    res = await fetch(`${VERIFY_BASE}/verify`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ signedTransaction }),
      signal: controller.signal,
    });
  } catch (err) {
    if (err?.name === "AbortError") {
      throw new Error(`backend verify timed out after ${timeoutMs}ms`);
    }
    throw err;
  } finally {
    clearTimeout(timer);
  }
  if (!res.ok) {
    const detail = await res.text().catch(() => "");
    throw new Error(`backend verify failed (${res.status}): ${detail}`);
  }
  return res.json();
}

/**
 * Purchase, then verify the result server-side. Returns the backend's
 * authoritative entitlement plus the raw StoreKit purchase result. For a
 * non-purchased outcome (pending / userCancelled / failed) there's no jws to
 * verify, so `entitled` is false and `verified` is false.
 *
 * If Apple reports `purchased` with a JWS but the backend is unreachable, we
 * still treat the user as entitled (local receipt is the offline unlock —
 * CodeRabbit #3584706876). `verified` is false in that case.
 * @param {string} [productId]
 * @returns {Promise<{ entitled: boolean, verified: boolean, purchase: object,
 *   backend?: object }>}
 */
export async function purchaseAndVerify(productId = PRODUCT_ID) {
  const result = await purchase(productId);
  if (result.status !== "purchased" || !result.jws) {
    return { entitled: false, verified: false, purchase: result };
  }
  try {
    const backend = await verifyWithBackend(result.jws);
    return {
      entitled: Boolean(backend.entitled),
      verified: true,
      purchase: result,
      backend,
    };
  } catch {
    // Paid on-device; backend hiccup must not lock the customer out.
    return { entitled: true, verified: false, purchase: result };
  }
}

/**
 * Authoritative entitlement check: fetch the current StoreKit entitlement's
 * signed `jws` and verify it with the backend. Swallows errors into `false` so
 * callers can gate UI without a try/catch. This is the check to trust.
 * @param {string} [productId]
 * @returns {Promise<boolean>}
 */
export async function isEntitledVerified(productId = PRODUCT_ID) {
  try {
    const status = await currentEntitlement(productId);
    if (!status || !status.jws) return false;
    const backend = await verifyWithBackend(status.jws);
    return Boolean(backend.entitled);
  } catch {
    return false;
  }
}
