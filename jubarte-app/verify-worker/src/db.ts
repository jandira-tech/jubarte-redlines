import type { SubscriptionState } from "./entitlement";

interface Row {
  original_transaction_id: string;
  product_id: string;
  environment: string;
  expires_date_ms: number | null;
  grace_period_expires_date_ms: number | null;
  revocation_date_ms: number | null;
  auto_renew_status: number;
  latest_transaction_id: string | null;
  signed_date_ms: number;
}

function rowToState(r: Row): SubscriptionState {
  return {
    originalTransactionId: r.original_transaction_id,
    productId: r.product_id,
    environment: r.environment === "Production" ? "Production" : "Sandbox",
    expiresDateMs: r.expires_date_ms,
    gracePeriodExpiresDateMs: r.grace_period_expires_date_ms,
    revocationDateMs: r.revocation_date_ms,
    autoRenewStatus: r.auto_renew_status === 1,
    latestTransactionId: r.latest_transaction_id,
    signedDateMs: r.signed_date_ms,
  };
}

export async function getSubscription(
  db: D1Database,
  originalTransactionId: string,
): Promise<SubscriptionState | null> {
  const row = await db
    .prepare("SELECT * FROM subscriptions WHERE original_transaction_id = ?")
    .bind(originalTransactionId)
    .first<Row>();
  return row ? rowToState(row) : null;
}

/**
 * Upsert a subscription's state. The `ON CONFLICT ... WHERE` guard makes stale
 * / out-of-order deliveries a no-op atomically: an update only lands if its
 * `signed_date_ms` is at least as recent as what's stored.
 */
export async function upsertSubscription(
  db: D1Database,
  state: SubscriptionState,
  nowMs: number,
): Promise<void> {
  await db
    .prepare(
      `INSERT INTO subscriptions (
         original_transaction_id, product_id, environment, expires_date_ms,
         grace_period_expires_date_ms, revocation_date_ms, auto_renew_status,
         latest_transaction_id, signed_date_ms, updated_at_ms
       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(original_transaction_id) DO UPDATE SET
         product_id = excluded.product_id,
         environment = excluded.environment,
         expires_date_ms = excluded.expires_date_ms,
         grace_period_expires_date_ms = excluded.grace_period_expires_date_ms,
         revocation_date_ms = excluded.revocation_date_ms,
         auto_renew_status = excluded.auto_renew_status,
         latest_transaction_id = excluded.latest_transaction_id,
         signed_date_ms = excluded.signed_date_ms,
         updated_at_ms = excluded.updated_at_ms
       WHERE excluded.signed_date_ms >= subscriptions.signed_date_ms`,
    )
    .bind(
      state.originalTransactionId,
      state.productId,
      state.environment,
      state.expiresDateMs,
      state.gracePeriodExpiresDateMs,
      state.revocationDateMs,
      state.autoRenewStatus ? 1 : 0,
      state.latestTransactionId,
      state.signedDateMs,
      nowMs,
    )
    .run();
}
