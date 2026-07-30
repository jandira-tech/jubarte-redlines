/**
 * Free-redline accounting.
 *
 * The redline itself runs as wasm in the visitor's browser, so this module
 * never sees a document — only an opaque visitor id and a count. It exists to
 * decide when to show the "contact us" panel, not to protect a paid feature
 * (that is verify-worker's job, against Apple-signed receipts).
 */

export type QuotaView = {
  used: number;
  limit: number;
  remaining: number;
  /** True once the allowance is spent — the client swaps in the contact panel. */
  paywalled: boolean;
};

export type ConsumeResult = QuotaView & { allowed: boolean };

function view(used: number, limit: number): QuotaView {
  const remaining = Math.max(0, limit - used);
  return { used, limit, remaining, paywalled: remaining === 0 };
}

/**
 * Read a visitor's standing without creating a row.
 *
 * Page loads call this, and most visitors never run a redline — writing on
 * read would turn every bounce into a D1 write and a row we have to store.
 */
export async function peek(db: D1Database, visitorId: string, limit: number) {
  const row = await db
    .prepare("SELECT used FROM quota WHERE visitor_id = ?")
    .bind(visitorId)
    .first<{ used: number }>();

  return view(row?.used ?? 0, limit);
}

/**
 * Charge one redline against the allowance.
 *
 * The increment is a single atomic statement with the limit enforced in the
 * WHERE clause, so concurrent requests from the same visitor can never
 * overshoot: SQLite serializes the writes and the loser sees `used` already at
 * the cap. A read-then-write would let two in-flight requests both observe
 * used=4 and both be granted.
 *
 * `RETURNING used` distinguishes the two outcomes without a second query — no
 * row comes back exactly when the guard rejected the update.
 */
export async function consume(
  db: D1Database,
  visitorId: string,
  limit: number,
  nowMs: number,
): Promise<ConsumeResult> {
  const granted = await db
    .prepare(
      `INSERT INTO quota (visitor_id, used, first_seen_ms, updated_at_ms)
       VALUES (?1, 1, ?2, ?2)
       ON CONFLICT (visitor_id) DO UPDATE
         SET used = quota.used + 1, updated_at_ms = ?2
         WHERE quota.used < ?3
       RETURNING used`,
    )
    .bind(visitorId, nowMs, limit)
    .first<{ used: number }>();

  if (granted) return { allowed: true, ...view(granted.used, limit) };

  // Guard rejected the update: the visitor is already at the cap. Report the
  // stored count rather than incrementing — a denied click is not usage, and
  // letting it climb would corrupt the funnel numbers.
  return { allowed: false, ...(await peek(db, visitorId, limit)) };
}
