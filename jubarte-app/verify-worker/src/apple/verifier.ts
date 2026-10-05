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
      // Name both reasons: a Sandbox transaction always fails Production with
      // INVALID_ENVIRONMENT, so its real failure is the Sandbox one
      // (gemini #3583828482).
      throw new Error(
        `production: ${describeVerificationError(productionError)}; ` +
          `sandbox: ${describeVerificationError(sandboxError)}`,
      );
    }
  }
}

// Apple's VerificationStatus, by value. Copied rather than imported: importing
// the library at module scope breaks Workers (see AppleVerifier).
const VERIFICATION_STATUS = [
  "OK",
  "VERIFICATION_FAILURE",
  "RETRYABLE_VERIFICATION_FAILURE",
  "INVALID_APP_IDENTIFIER",
  "INVALID_ENVIRONMENT",
  "INVALID_CHAIN_LENGTH",
  "INVALID_CERTIFICATE",
  "FAILURE",
];

/** Why a verification failed, for the log. Apple's VerificationException
 * carries a numeric status and an empty message, so String(e) reads "Error". */
export function describeVerificationError(e: unknown): string {
  if (e instanceof Error && "status" in e && typeof e.status === "number") {
    const name = VERIFICATION_STATUS[e.status] ?? `status ${e.status}`;
    return e.cause instanceof Error ? `${name} (${e.cause.message})` : name;
  }
  return e instanceof Error ? e.message || e.name : String(e);
}
