-- Entitlement state, keyed by Apple's originalTransactionId (stable across the
-- whole subscription lifetime, including renewals). One row per subscription.
CREATE TABLE IF NOT EXISTS subscriptions (
  original_transaction_id       TEXT    PRIMARY KEY,
  product_id                    TEXT    NOT NULL,
  environment                   TEXT    NOT NULL,             -- 'Sandbox' | 'Production'
  expires_date_ms               INTEGER,                      -- null only for non-expiring types
  grace_period_expires_date_ms  INTEGER,                      -- set during billing grace period
  revocation_date_ms            INTEGER,                      -- set on refund / revoke (terminal)
  auto_renew_status             INTEGER NOT NULL DEFAULT 0,   -- 0 | 1
  latest_transaction_id         TEXT,
  signed_date_ms                INTEGER NOT NULL,             -- Apple's signing time; drops stale updates
  updated_at_ms                 INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_product ON subscriptions (product_id);
