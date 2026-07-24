/// <reference types="@cloudflare/vitest-pool-workers/types" />
import type { D1Migration } from "@cloudflare/vitest-pool-workers";

// The pool types `env` (from "cloudflare:test") as `Cloudflare.Env`. DB + APPLE_*
// already live there (wrangler-generated); add only the test-injected migrations.
declare global {
  namespace Cloudflare {
    interface Env {
      TEST_MIGRATIONS: D1Migration[];
    }
  }
}
