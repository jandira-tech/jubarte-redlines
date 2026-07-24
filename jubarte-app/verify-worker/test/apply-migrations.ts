import { applyD1Migrations, env } from "cloudflare:test";

// Apply the schema to each test worker's isolated D1 before any test runs.
await applyD1Migrations(env.DB, env.TEST_MIGRATIONS);
