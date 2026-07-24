import type {
  JWSRenewalInfoDecodedPayload,
  JWSTransactionDecodedPayload,
  ResponseBodyV2DecodedPayload,
  SignedDataVerifier,
} from "@apple/app-store-server-library";
import { APPLE_ROOT_CAS } from "./roots";

export interface VerifierConfig {
  bundleId: string;
  /** Numeric App Store app id — REQUIRED to verify Production data. */
  appAppleId: number;
  /** OCSP revocation checks. Needs outbound fetch; off by default on Workers. */
  enableOnlineChecks: boolean;
}

type Pair = { production: SignedDataVerifier; sandbox: SignedDataVerifier };

/**
 * Wraps Apple's SignedDataVerifier for both environments. Sandbox testers and
 * production customers hit the same endpoint, and a JWS doesn't announce its
 * environment before you verify it, so we follow Apple's guidance: try
 * Production, fall back to Sandbox.
 *
 * The `@apple/app-store-server-library` import is DEFERRED (dynamic import inside
 * a handler) on purpose: its transitive dep `jsrsasign` seeds an RNG at
 * module-eval time, and Workers forbids generating random values in global
 * scope. Importing it lazily moves that init to request time, where it's legal.
 */
export class AppleVerifier {
  private pair: Promise<Pair> | null = null;

  constructor(private readonly cfg: VerifierConfig) {}

  private verifiers(): Promise<Pair> {
    if (!this.pair) this.pair = this.build();
    return this.pair;
  }

  private async build(): Promise<Pair> {
    const { Environment, SignedDataVerifier } = await import(
      "@apple/app-store-server-library"
    );
    return {
      production: new SignedDataVerifier(
        APPLE_ROOT_CAS,
        this.cfg.enableOnlineChecks,
        Environment.PRODUCTION,
        this.cfg.bundleId,
        this.cfg.appAppleId,
      ),
      sandbox: new SignedDataVerifier(
        APPLE_ROOT_CAS,
        this.cfg.enableOnlineChecks,
        Environment.SANDBOX,
        this.cfg.bundleId,
        undefined,
      ),
    };
  }

  async verifyTransaction(signed: string): Promise<JWSTransactionDecodedPayload> {
    const { production, sandbox } = await this.verifiers();
    return tryBoth(production, sandbox, (v) => v.verifyAndDecodeTransaction(signed));
  }

  async verifyRenewalInfo(signed: string): Promise<JWSRenewalInfoDecodedPayload> {
    const { production, sandbox } = await this.verifiers();
    return tryBoth(production, sandbox, (v) => v.verifyAndDecodeRenewalInfo(signed));
  }

  async verifyNotification(signed: string): Promise<ResponseBodyV2DecodedPayload> {
    const { production, sandbox } = await this.verifiers();
    return tryBoth(production, sandbox, (v) => v.verifyAndDecodeNotification(signed));
  }
}

async function tryBoth<T>(
  production: SignedDataVerifier,
  sandbox: SignedDataVerifier,
  fn: (v: SignedDataVerifier) => Promise<T>,
): Promise<T> {
  try {
    return await fn(production);
  } catch (productionError) {
    try {
      return await fn(sandbox);
    } catch (sandboxError) {
      // Sandbox is the usual path during development; log it so a real Sandbox
      // failure is not masked by the Production error (gemini #3583828482).
      console.warn("jubarte: sandbox verification also failed", sandboxError);
      // Both failed — surface the Production error (usually the informative one).
      throw productionError;
    }
  }
}
