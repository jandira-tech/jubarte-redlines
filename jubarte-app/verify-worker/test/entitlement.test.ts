import { describe, expect, it } from "vitest";
import {
  isEntitled,
  type SubscriptionState,
  toSubscriptionState,
} from "../src/entitlement";

const NOW = 1_800_000_000_000; // fixed "now" for deterministic tests
const DAY = 86_400_000;

function state(overrides: Partial<SubscriptionState> = {}): SubscriptionState {
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
    ...overrides,
  };
}

describe("isEntitled", () => {
  it("active while the subscription has not expired", () => {
    expect(isEntitled(state({ expiresDateMs: NOW + DAY }), NOW)).toBe(true);
  });

  it("inactive once expired with no grace period", () => {
    expect(isEntitled(state({ expiresDateMs: NOW - DAY }), NOW)).toBe(false);
  });

  it("ACTIVE during a billing grace period (expired but grace in the future)", () => {
    // The exact case a naive expiry check gets wrong.
    expect(
      isEntitled(
        state({ expiresDateMs: NOW - DAY, gracePeriodExpiresDateMs: NOW + 5 * DAY }),
        NOW,
      ),
    ).toBe(true);
  });

  it("inactive when the grace period itself has passed", () => {
    expect(
      isEntitled(
        state({ expiresDateMs: NOW - 10 * DAY, gracePeriodExpiresDateMs: NOW - DAY }),
        NOW,
      ),
    ).toBe(false);
  });

  it("inactive when revoked, even if not yet expired", () => {
    expect(
      isEntitled(
        state({ expiresDateMs: NOW + 30 * DAY, revocationDateMs: NOW - DAY }),
        NOW,
      ),
    ).toBe(false);
  });

  it("revocation dated in the future does not yet revoke", () => {
    expect(
      isEntitled(
        state({ expiresDateMs: NOW + 30 * DAY, revocationDateMs: NOW + DAY }),
        NOW,
      ),
    ).toBe(true);
  });

  it("ACTIVE for non-expiring products (null expiresDateMs)", () => {
    // Lifetime / non-subscription product types — schema documents null expiry.
    expect(isEntitled(state({ expiresDateMs: null }), NOW)).toBe(true);
  });

  it("revoked non-expiring product is inactive", () => {
    expect(
      isEntitled(
        state({ expiresDateMs: null, revocationDateMs: NOW - DAY }),
        NOW,
      ),
    ).toBe(false);
  });
});

describe("toSubscriptionState", () => {
  it("maps a verified transaction + renewal into state", () => {
    const s = toSubscriptionState(
      {
        originalTransactionId: "2000000123",
        transactionId: "2000000999",
        productId: "com.jandira.jubarte.annual",
        expiresDate: NOW + 30 * DAY,
        signedDate: NOW,
        environment: "Production",
      },
      { autoRenewStatus: 1, gracePeriodExpiresDate: NOW + 5 * DAY, signedDate: NOW },
    );
    expect(s.originalTransactionId).toBe("2000000123");
    expect(s.productId).toBe("com.jandira.jubarte.annual");
    expect(s.environment).toBe("Production");
    expect(s.expiresDateMs).toBe(NOW + 30 * DAY);
    expect(s.gracePeriodExpiresDateMs).toBe(NOW + 5 * DAY);
    expect(s.autoRenewStatus).toBe(true);
    expect(s.latestTransactionId).toBe("2000000999");
  });

  it("defaults environment to Sandbox and autoRenew to false", () => {
    const s = toSubscriptionState({
      originalTransactionId: "1",
      productId: "p",
      expiresDate: NOW,
    });
    expect(s.environment).toBe("Sandbox");
    expect(s.autoRenewStatus).toBe(false);
    expect(s.gracePeriodExpiresDateMs).toBeNull();
    expect(s.revocationDateMs).toBeNull();
  });

  it("carries a refund's revocationDate through", () => {
    const s = toSubscriptionState({
      originalTransactionId: "1",
      productId: "p",
      expiresDate: NOW + DAY,
      revocationDate: NOW - DAY,
    });
    expect(s.revocationDateMs).toBe(NOW - DAY);
    expect(isEntitled(s, NOW)).toBe(false);
  });

  it("throws when originalTransactionId is missing", () => {
    expect(() => toSubscriptionState({ productId: "p", expiresDate: NOW })).toThrow();
  });
});
