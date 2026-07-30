/// <reference types="@cloudflare/vitest-pool-workers/types" />
import type { D1Migration } from "@cloudflare/vitest-pool-workers";

// The pool types `env` (from "cloudflare:test") as `Cloudflare.Env`. DB and the
// wrangler `vars` already live there; add only the test-injected migrations.
declare global {
  namespace Cloudflare {
    interface Env {
      TEST_MIGRATIONS: D1Migration[];
    }
  }
}
