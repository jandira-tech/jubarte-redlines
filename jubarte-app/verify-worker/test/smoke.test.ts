import {
  Environment,
  SignedDataVerifier,
  VerificationException,
  VerificationStatus,
} from "@apple/app-store-server-library";
import { describe, expect, it } from "vitest";

// Apple's unit-test CA chain (from app-store-server-library jws_verification tests).
// Used only to prove our Worker reaches certificate-chain + signature verification,
// not JWT-decode-only rejection of garbage strings.
const ROOT_CA_B64 =
  "MIIBgjCCASmgAwIBAgIJALUc5ALiH5pbMAoGCCqGSM49BAMDMDYxCzAJBgNVBAYTAlVTMRMwEQYDVQQIDApDYWxpZm9ybmlhMRIwEAYDVQQHDAlDdXBlcnRpbm8wHhcNMjMwMTA1MjEzMDIyWhcNMzMwMTAyMjEzMDIyWjA2MQswCQYDVQQGEwJVUzETMBEGA1UECAwKQ2FsaWZvcm5pYTESMBAGA1UEBwwJQ3VwZXJ0aW5vMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEc+/Bl+gospo6tf9Z7io5tdKdrlN1YdVnqEhEDXDShzdAJPQijamXIMHf8xWWTa1zgoYTxOKpbuJtDplz1XriTaMgMB4wDAYDVR0TBAUwAwEB/zAOBgNVHQ8BAf8EBAMCAQYwCgYIKoZIzj0EAwMDRwAwRAIgemWQXnMAdTad2JDJWng9U4uBBL5mA7WI05H7oH7c6iQCIHiRqMjNfzUAyiu9h6rOU/K+iTR0I/3Y/NSWsXHX+acc";
const INTERMEDIATE_CA_B64 =
  "MIIBnzCCAUWgAwIBAgIBCzAKBggqhkjOPQQDAzA2MQswCQYDVQQGEwJVUzETMBEGA1UECAwKQ2FsaWZvcm5pYTESMBAGA1UEBwwJQ3VwZXJ0aW5vMB4XDTIzMDEwNTIxMzEwNVoXDTMzMDEwMTIxMzEwNVowRTELMAkGA1UEBhMCVVMxCzAJBgNVBAgMAkNBMRIwEAYDVQQHDAlDdXBlcnRpbm8xFTATBgNVBAoMDEludGVybWVkaWF0ZTBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABBUN5V9rKjfRiMAIojEA0Av5Mp0oF+O0cL4gzrTF178inUHugj7Et46NrkQ7hKgMVnjogq45Q1rMs+cMHVNILWqjNTAzMA8GA1UdEwQIMAYBAf8CAQAwDgYDVR0PAQH/BAQDAgEGMBAGCiqGSIb3Y2QGAgEEAgUAMAoGCCqGSM49BAMDA0gAMEUCIQCmsIKYs41ullssHX4rVveUT0Z7Is5/hLK1lFPTtun3hAIgc2+2RG5+gNcFVcs+XJeEl4GZ+ojl3ROOmll+ye7dynQ=";
const LEAF_CERT_B64 =
  "MIIBoDCCAUagAwIBAgIBDDAKBggqhkjOPQQDAzBFMQswCQYDVQQGEwJVUzELMAkGA1UECAwCQ0ExEjAQBgNVBAcMCUN1cGVydGlubzEVMBMGA1UECgwMSW50ZXJtZWRpYXRlMB4XDTIzMDEwNTIxMzEzNFoXDTMzMDEwMTIxMzEzNFowPTELMAkGA1UEBhMCVVMxCzAJBgNVBAgMAkNBMRIwEAYDVQQHDAlDdXBlcnRpbm8xDTALBgNVBAoMBExlYWYwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAATitYHEaYVuc8g9AjTOwErMvGyPykPa+puvTI8hJTHZZDLGas2qX1+ErxgQTJgVXv76nmLhhRJH+j25AiAI8iGsoy8wLTAJBgNVHRMEAjAAMA4GA1UdDwEB/wQEAwIHgDAQBgoqhkiG92NkBgsBBAIFADAKBggqhkjOPQQDAwNIADBFAiBX4c+T0Fp5nJ5QRClRfu5PSByRvNPtuaTsk0vPB3WAIAIhANgaauAj/YP9s0AkEhyJhxQO/6Q2zouZ+H1CIOehnMzQ";

function b64urlJson(obj: unknown): string {
  const json = JSON.stringify(obj);
  // btoa is available in workerd; Buffer also works under nodejs_compat.
  const bytes =
    typeof Buffer !== "undefined"
      ? Buffer.from(json, "utf8")
      : new TextEncoder().encode(json);
  const b64 =
    typeof Buffer !== "undefined"
      ? Buffer.from(bytes).toString("base64")
      : btoa(String.fromCharCode(...bytes));
  return b64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/**
 * Well-formed 3-segment JWS with x5c cert chain (leaf, intermediate, root)
 * and a deliberately invalid signature. SignedDataVerifier must reach
 * certificate-chain / signature verification — not fail only at JWT decode.
 */
function jwsWithX5cButBadSignature(): string {
  const header = {
    alg: "ES256",
    x5c: [LEAF_CERT_B64, INTERMEDIATE_CA_B64, ROOT_CA_B64],
  };
  // signedDate inside test-CA validity window (2023–2033).
  const payload = {
    transactionId: "1000000123456789",
    originalTransactionId: "1000000123456789",
    bundleId: "com.jandira.jubarte",
    productId: "com.jandira.jubarte.annual",
    purchaseDate: 1_700_000_000_000,
    originalPurchaseDate: 1_700_000_000_000,
    signedDate: 1_700_000_000_000,
    environment: "Sandbox",
    type: "Auto-Renewable Subscription",
    inAppOwnershipType: "PURCHASED",
    quantity: 1,
  };
  // 64 zero bytes as ES256 "signature" — will not verify against the leaf key.
  const sig =
    typeof Buffer !== "undefined"
      ? Buffer.alloc(64, 0).toString("base64url")
      : "A".repeat(86);
  return `${b64urlJson(header)}.${b64urlJson(payload)}.${sig}`;
}

// Does Apple's official library actually load AND execute inside workerd
// (with nodejs_compat)? A static import means the bundler must resolve every
// transitive node dependency (jsonwebtoken -> node:crypto, node-fetch ->
// node:http, jsrsasign). If workerd can't provide them, this file fails to
// load and every test errors — which is exactly the signal we want.
describe("@apple/app-store-server-library on workerd", () => {
  it("exposes the verification symbols", () => {
    expect(typeof SignedDataVerifier).toBe("function");
    expect(Environment.SANDBOX).toBeDefined();
    expect(Environment.PRODUCTION).toBeDefined();
  });

  it("rejects garbage before cert verification (decode path)", async () => {
    const verifier = new SignedDataVerifier(
      [],
      false,
      Environment.SANDBOX,
      "com.jandira.jubarte",
      undefined,
    );
    await expect(
      verifier.verifyAndDecodeTransaction("not-a-real-jws"),
    ).rejects.toBeDefined();
  });

  it("reaches cert-chain / signature verification for a well-formed x5c JWS", async () => {
    const root = Buffer.from(ROOT_CA_B64, "base64");
    const verifier = new SignedDataVerifier(
      [root],
      false, // offline — no OCSP/node-fetch
      Environment.SANDBOX,
      "com.jandira.jubarte",
      undefined,
    );
    const jws = jwsWithX5cButBadSignature();
    // Must fail with Apple's VerificationException after attempting chain/sig,
    // not with a raw jsonwebtoken parse error on an unstructured string.
    try {
      await verifier.verifyAndDecodeTransaction(jws);
      expect.fail("expected verification to reject bad signature / chain");
    } catch (e) {
      expect(e).toBeInstanceOf(VerificationException);
      const status = (e as VerificationException).status;
      // Chain may pass (test CA) then signature fails → VERIFICATION_FAILURE;
      // or OID/chain rules fail → INVALID_CERTIFICATE / VERIFICATION_FAILURE.
      expect([
        VerificationStatus.VERIFICATION_FAILURE,
        VerificationStatus.INVALID_CERTIFICATE,
        VerificationStatus.INVALID_CHAIN_LENGTH,
      ]).toContain(status);
    }
  });
});
