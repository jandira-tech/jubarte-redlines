-- Anonymous free-redline counter, keyed by the opaque `jid` cookie the Worker
-- mints on first visit. No document bytes, no filenames, no IP — the compare
-- runs in the visitor's browser and this table only ever sees a random id.
--
-- Clearing cookies yields a fresh allowance. That is intentional: this gates a
-- "contact us" funnel, not a paid entitlement (see verify-worker for the real
-- thing), and hardening it further would cost more privacy than it buys.
CREATE TABLE IF NOT EXISTS quota (
  visitor_id     TEXT    PRIMARY KEY,
  used           INTEGER NOT NULL DEFAULT 0,
  first_seen_ms  INTEGER NOT NULL,
  updated_at_ms  INTEGER NOT NULL
);

-- Supports "how many visitors hit the wall this week" without a full scan.
CREATE INDEX IF NOT EXISTS idx_quota_updated ON quota (updated_at_ms);
