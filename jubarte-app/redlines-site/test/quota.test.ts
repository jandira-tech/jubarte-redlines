import { env } from "cloudflare:test";
import { beforeEach, describe, expect, it } from "vitest";
import { consume, peek } from "../src/quota";

const NOW = 1_700_000_000_000;

beforeEach(async () => {
  await env.DB.exec("DELETE FROM quota");
});

describe("peek", () => {
  it("reports a full allowance for a visitor it has never seen", async () => {
    // Deliberately does NOT create a row — a visitor who only loads the page
    // must not cost us a D1 write.
    expect(await peek(env.DB, "ghost", 5)).toEqual({
      used: 0,
      limit: 5,
      remaining: 5,
      paywalled: false,
    });
    const row = await env.DB.prepare("SELECT COUNT(*) AS n FROM quota").first<{
      n: number;
    }>();
    expect(row?.n).toBe(0);
  });

  it("reflects consumption without mutating it", async () => {
    await consume(env.DB, "v1", 5, NOW);
    await consume(env.DB, "v1", 5, NOW);

    expect(await peek(env.DB, "v1", 5)).toMatchObject({ used: 2, remaining: 3 });
    expect(await peek(env.DB, "v1", 5)).toMatchObject({ used: 2, remaining: 3 });
  });
});

describe("consume", () => {
  it("grants the first redline and counts it", async () => {
    expect(await consume(env.DB, "v1", 5, NOW)).toEqual({
      allowed: true,
      used: 1,
      limit: 5,
      remaining: 4,
      paywalled: false,
    });
  });

  it("grants exactly `limit` redlines", async () => {
    const results = [];
    for (let i = 0; i < 5; i++) results.push(await consume(env.DB, "v1", 5, NOW + i));

    expect(results.map((r) => r.allowed)).toEqual([true, true, true, true, true]);
    expect(results.map((r) => r.used)).toEqual([1, 2, 3, 4, 5]);
    expect(results.map((r) => r.remaining)).toEqual([4, 3, 2, 1, 0]);
    // The 5th is still allowed but leaves nothing behind it.
    expect(results[4]).toMatchObject({ allowed: true, paywalled: true });
  });

  it("refuses the sixth and stops counting", async () => {
    for (let i = 0; i < 5; i++) await consume(env.DB, "v1", 5, NOW + i);

    const sixth = await consume(env.DB, "v1", 5, NOW + 5);
    expect(sixth).toEqual({
      allowed: false,
      used: 5,
      limit: 5,
      remaining: 0,
      paywalled: true,
    });

    // A denied request must not inflate `used` — otherwise a visitor who keeps
    // clicking would look like a heavier user than they are in the funnel data.
    const seventh = await consume(env.DB, "v1", 5, NOW + 6);
    expect(seventh.used).toBe(5);
  });

  it("keeps visitors independent", async () => {
    for (let i = 0; i < 5; i++) await consume(env.DB, "spent", 5, NOW + i);

    expect(await consume(env.DB, "spent", 5, NOW)).toMatchObject({ allowed: false });
    expect(await consume(env.DB, "fresh", 5, NOW)).toMatchObject({
      allowed: true,
      used: 1,
    });
  });

  it("preserves first_seen_ms across later redlines", async () => {
    await consume(env.DB, "v1", 5, NOW);
    await consume(env.DB, "v1", 5, NOW + 60_000);

    const row = await env.DB.prepare(
      "SELECT first_seen_ms, updated_at_ms FROM quota WHERE visitor_id = ?",
    )
      .bind("v1")
      .first<{ first_seen_ms: number; updated_at_ms: number }>();

    expect(row?.first_seen_ms).toBe(NOW);
    expect(row?.updated_at_ms).toBe(NOW + 60_000);
  });

  it("counts atomically under concurrent requests", async () => {
    // Read-then-write would let these interleave and overshoot the allowance.
    const results = await Promise.all(
      Array.from({ length: 12 }, (_, i) => consume(env.DB, "racer", 5, NOW + i)),
    );

    expect(results.filter((r) => r.allowed)).toHaveLength(5);
    expect(new Set(results.filter((r) => r.allowed).map((r) => r.used))).toEqual(
      new Set([1, 2, 3, 4, 5]),
    );
  });
});
