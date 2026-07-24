// Pure entitlement logic — no Apple SDK, no D1, no clock. Inputs mirror the
// field names/types of Apple's decoded payloads (JWSTransactionDecodedPayload /
// JWSRenewalInfoDecodedPayload: epoch-millisecond dates, all optional) so route
// code can pass the verified payloads straight through.
//
// This is the money-critical decision layer and is unit-tested exhaustively.

export type Environment = "Sandbox" | "Production";

export interface SubscriptionState {
  originalTransactionId: string;
  productId: string;
  environment: Environment;
  /** Epoch ms; null only for non-expiring product types. */
  expiresDateMs: number | null;
  /** Epoch ms; set while in a billing grace period (access continues). */
  gracePeriodExpiresDateMs: number | null;
  /** Epoch ms; set on refund/revoke (terminal). */
  revocationDateMs: number | null;
  autoRenewStatus: boolean;
  latestTransactionId: string | null;
  /** When Apple signed the source payload; used to drop stale updates. */
  signedDateMs: number;
}

/** Subset of Apple's JWSTransactionDecodedPayload we consume. */
export interface TransactionPayload {
  originalTransactionId?: string;
  transactionId?: string;
  productId?: string;
  expiresDate?: number;
  revocationDate?: number;
  signedDate?: number;
  environment?: string;
}

/** Subset of Apple's JWSRenewalInfoDecodedPayload we consume. */
export interface RenewalPayload {
  autoRenewStatus?: number;
  gracePeriodExpiresDate?: number;
  signedDate?: number;
}

/**
 * Is the customer entitled to paid access at `nowMs`?
 *
 * Grace-period aware: a subscription in billing retry keeps a past
 * `expiresDate` but a future `gracePeriodExpiresDate`, and access continues.
 * Revocation (refund / family-sharing removal) is terminal.
 */
export function isEntitled(state: SubscriptionState, nowMs: number): boolean {
  if (state.revocationDateMs != null && state.revocationDateMs <= nowMs) return false;
  // Non-expiring product types (lifetime) use null expiresDateMs — still entitled
  // unless revoked (gemini #3583828440).
  if (state.expiresDateMs == null) return true;
  if (state.expiresDateMs > nowMs) return true;
  if (state.gracePeriodExpiresDateMs != null && state.gracePeriodExpiresDateMs > nowMs) {
    return true;
  }
  return false;
}

/** Build a SubscriptionState from a verified transaction (+ optional renewal). */
export function toSubscriptionState(
  txn: TransactionPayload,
  renewal?: RenewalPayload,
): SubscriptionState {
  if (!txn.originalTransactionId) {
    throw new Error("transaction is missing originalTransactionId");
  }
  if (!txn.productId) {
    throw new Error("transaction is missing productId");
  }
  return {
    originalTransactionId: txn.originalTransactionId,
    productId: txn.productId,
    environment: txn.environment === "Production" ? "Production" : "Sandbox",
    expiresDateMs: txn.expiresDate ?? null,
    gracePeriodExpiresDateMs: renewal?.gracePeriodExpiresDate ?? null,
    revocationDateMs: txn.revocationDate ?? null,
    autoRenewStatus: renewal?.autoRenewStatus === 1,
    latestTransactionId: txn.transactionId ?? null,
    signedDateMs: txn.signedDate ?? renewal?.signedDate ?? 0,
  };
}
// Staleness / out-of-order protection is enforced atomically in the DB layer
// (upsertSubscription's `WHERE excluded.signed_date_ms >= ...` guard), and
// exercised by the route integration tests against real D1.
