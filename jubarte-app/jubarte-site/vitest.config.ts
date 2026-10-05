import { cloudflareTest } from "@cloudflare/vitest-pool-workers";
import { defineConfig } from "vitest/config";

// Two projects: the Worker runs in workerd with its real bindings; the browser
// modules (zip, docx preview, case filters) and the page templates run in Node.
export default defineConfig({
  test: {
    // Istanbul, not V8: the Workers pool cannot collect V8 coverage from workerd.
    coverage: {
      provider: "istanbul",
      include: ["src/**/*.ts", "site/**/*.{ts,js}"],
      reporter: ["text-summary", "text"],
    },
    projects: [
      {
        test: { name: "worker", include: ["test/worker/**/*.test.ts"] },
        plugins: [cloudflareTest({ wrangler: { configPath: "./wrangler.jsonc" } })],
      },
      {
        test: {
          name: "unit",
          include: ["test/unit/**/*.test.ts", "test/node/**/*.test.ts"],
          environment: "node",
        },
      },
    ],
  },
});
