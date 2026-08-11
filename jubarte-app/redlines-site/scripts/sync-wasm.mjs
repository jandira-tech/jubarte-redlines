/**
 * Rebuild the wasm bundle from the local `jubarte-wasm` crate and copy the
 * artefacts into `public/vendor/`.
 *
 * The crate lives in the parent jubarte-redlines checkout, not in this
 * repository, so this only works from the monorepo. Outside it — a bare clone of
 * jubarte-app, or CI that checked out only this repo — the committed bundle in
 * public/vendor/ is already correct, so we say so and exit 0 rather than
 * breaking `bun run deploy`.
 *
 * Requires wasm-pack and binaryen (`wasm-opt`) on PATH.
 */
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import path from "node:path";

const here = import.meta.dirname;
const crate = path.resolve(here, "../../../jubarte-wasm");
const pkg = path.join(crate, "pkg-web");
const vendor = path.resolve(here, "../public/vendor");

if (!existsSync(crate)) {
  console.log(`sync-wasm: no jubarte-wasm crate at ${crate} — outside the monorepo.`);
  console.log("sync-wasm: keeping the committed bundle in public/vendor/. Skipping.");
  process.exit(0);
}

console.log("sync-wasm: building (wasm-pack --release --target web)…");
const build = spawnSync(
  "wasm-pack",
  ["build", "--release", "--target", "web", "--out-dir", "pkg-web"],
  { cwd: crate, stdio: "inherit" },
);

if (build.error?.code === "ENOENT") {
  console.error("sync-wasm: wasm-pack not found on PATH — cargo install wasm-pack");
  process.exit(1);
}
if (build.status !== 0) process.exit(build.status ?? 1);

mkdirSync(vendor, { recursive: true });
for (const file of ["jubarte_wasm.js", "jubarte_wasm_bg.wasm", "jubarte_wasm.d.ts"]) {
  copyFileSync(path.join(pkg, file), path.join(vendor, file));
  console.log(`sync-wasm: ${file}`);
}
console.log("sync-wasm: done");
