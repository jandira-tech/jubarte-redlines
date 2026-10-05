import { spawnSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { fold } from "../../site/data/facts.ts";

// scripts/release.sh, run for real in a throwaway copy of the files it reads
// (the facts log, facts.py, the data modules and the three node scripts) with
// a throwaway engine checkout beside it. pnpm, wrangler, gh, npm and curl are
// stubs first on PATH that write each call to a log: nothing is installed,
// deployed or fetched, and the copy holds no wrangler.jsonc to deploy from.
const SITE = fileURLToPath(new URL("../..", import.meta.url));
const APP = join(SITE, "..");
const VER = "0.9.9";

// facts.py is the log's only writer and needs uv with Python 3.14 (CI's Node
// job has neither).
const canRun =
  spawnSync("uv", ["python", "find", "3.14"]).status === 0 &&
  spawnSync("git", ["--version"]).status === 0;

const STUBS: Record<string, string> = {
  pnpm: `echo "pnpm $*" >> "$CALLS"
[ "$1" = add ] && grep "jubarte-wasm@" pnpm-workspace.yaml | sed 's/^ *- /excluded /' >> "$CALLS"
[ "\${PNPM_FAIL:-}" = "$1" ] && exit 1
exit 0
`,
  wrangler: `echo "wrangler $*" >> "$CALLS"\n`,
  npm: `echo "npm $*" >> "$CALLS"\n[ "$1" = view ] && echo ${VER}\nexit 0\n`,
  curl: `echo "curl $*" >> "$CALLS"\ncat "$STUB_DIR/page.html"\n`,
  gh: `echo "gh $*" >> "$CALLS"
case "$*" in
  *'.assets[].name'*) cat "$STUB_DIR/names.txt" ;;
  *'.assets'*) cat "$STUB_DIR/assets.json" ;;
esac
`,
};

const ASSETS = [
  `jubarte-${VER}-linux-x86_64.tar.gz`,
  `jubarte-${VER}-linux-aarch64.tar.gz`,
  `jubarte-${VER}-macos-aarch64.tar.gz`,
  `jubarte-${VER}-macos-x86_64.tar.gz`,
  `jubarte-${VER}-windows-x86_64.zip`,
  `jubarte_redlines-${VER}-cp310-abi3-macosx_11_0_arm64.whl`,
  `jubarte_redlines-${VER}.tar.gz`,
  "SHA256SUMS.txt",
];

let root: string;
let app: string;
let engine: string;
let calls: string;

const git = (...args: string[]) =>
  spawnSync("git", ["-C", app, "-c", "user.name=t", "-c", "user.email=t@example.com", ...args], {
    encoding: "utf8",
  });

function release(args: string[], env: Record<string, string> = {}) {
  return spawnSync("bash", [join(app, "jubarte-site/scripts/release.sh"), ...args], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${join(root, "bin")}:${process.env.PATH}`,
      CALLS: calls,
      STUB_DIR: join(root, "bin"),
      ENGINE: engine,
      CHANGELOG: join(engine, "CHANGELOG.md"),
      LIVE_TRIES: "2",
      LIVE_WAIT_SECONDS: "0",
      ...env,
    },
  });
}

const log = () => readFileSync(calls, "utf8");
const factsText = () => readFileSync(join(app, "data/facts.jsonl"), "utf8");
const page = (html: string) => writeFileSync(join(root, "bin/page.html"), html);

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "site-release-"));
  app = join(root, "app");
  engine = join(root, "engine");
  calls = join(root, "calls.log");
  writeFileSync(calls, "");
  for (const rel of [
    "data/facts.jsonl",
    "scripts/facts.py",
    "jubarte-site/package.json",
    "jubarte-site/pnpm-workspace.yaml",
    "jubarte-site/scripts/release.sh",
    "jubarte-site/scripts/sync-bench.ts",
    "jubarte-site/scripts/sync-release.ts",
    "jubarte-site/scripts/check-bench.ts",
    "jubarte-site/site/data/facts.ts",
    "jubarte-site/site/data/bench.ts",
    "jubarte-site/site/data/release.ts",
    "jubarte-site/test/fixtures/RESULTS.md",
  ]) {
    cpSync(join(APP, rel), join(app, rel));
  }
  cpSync(join(SITE, "test/fixtures/release_info"), join(engine, "release_info"), {
    recursive: true,
  });
  writeFileSync(
    join(engine, "CHANGELOG.md"),
    `# Changelog\n\n## [${VER}] - 2026-10-03\n\n> **Summary.** A release for the tests.\n`,
  );
  mkdirSync(join(root, "bin"));
  for (const [name, body] of Object.entries(STUBS)) {
    writeFileSync(join(root, "bin", name), `#!/bin/sh\n${body}`);
    chmodSync(join(root, "bin", name), 0o755);
  }
  writeFileSync(join(root, "bin/names.txt"), `${ASSETS.join("\n")}\n`);
  writeFileSync(
    join(root, "bin/assets.json"),
    JSON.stringify(ASSETS.map((name) => ({ name, size: 1 }))),
  );
  page(`<h2>Engine: jubarte-redlines ${VER}</h2>`);
  git("init", "-q", "-b", "main");
  git("add", "-A");
  git("commit", "-q", "-m", "base");
});

afterEach(() => {
  rmSync(root, { recursive: true, force: true });
});

describe.skipIf(!canRun)("scripts/release.sh", () => {
  it("release --no-deploy moves the engine and the figures, checks them, and deploys nothing", () => {
    const r = release(["release", VER, "--no-deploy"]);
    expect(r.status, r.stderr).toBe(0);
    const facts = fold(factsText());
    expect(facts.get("engine.version")).toBe(VER);
    expect((facts.get("bench.tables") as { id: string }[]).map((t) => t.id)).toEqual([
      "conversion-sample",
      "redlines-sample",
      "docxide-metrics",
      "accept-reject",
      "below-3-pages",
      "no-redline",
    ]);
    expect(log()).toContain("pnpm test\npnpm lint\npnpm typecheck\n");
    expect(log()).not.toContain("deploy");
    expect(log()).not.toContain("curl");
    expect(log()).not.toContain("wrangler");
    expect(r.stdout).toContain("not deployed (--no-deploy)");
    expect(r.stdout).toContain(`scripts/release.sh deploy ${VER}`);
  }, 120_000);

  it("keeps the lockfile's jubarte-wasm excluded from pnpm's age policy until the add replaces it", () => {
    // 0.11.2: two engine releases in one day; the exclusion moved off 0.11.0
    // before `pnpm add`, and pnpm refused the lockfile's still-young 0.11.0.
    const pkg = JSON.parse(readFileSync(join(app, "jubarte-site/package.json"), "utf8"));
    const old = String(pkg.devDependencies["jubarte-wasm"]).replace(/^[~^]/, "");
    const r = release(["release", VER, "--no-deploy"]);
    expect(r.status, r.stderr).toBe(0);
    const atAdd = log()
      .split("\n")
      .filter((l) => l.startsWith("excluded "));
    expect(atAdd).toContain(`excluded jubarte-wasm@${old}`);
    expect(atAdd).toContain(`excluded jubarte-wasm@${VER}`);
    const after = readFileSync(join(app, "jubarte-site/pnpm-workspace.yaml"), "utf8");
    expect(after).toContain(`  - jubarte-wasm@${VER}\n`);
    expect(after).not.toContain(`jubarte-wasm@${old}\n`);
  }, 120_000);

  it("a second run adds no record", () => {
    expect(release(["release", VER, "--no-deploy"]).status).toBe(0);
    const once = factsText();
    const r = release(["release", VER, "--no-deploy"]);
    expect(r.status, r.stderr).toBe(0);
    expect(factsText()).toBe(once);
  }, 120_000);

  it("figures refuses a log that names another engine, and writes nothing", () => {
    const before = factsText();
    const r = release(["figures", VER]);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain(`not ${VER}`);
    expect(r.stderr).toContain(`scripts/release.sh engine ${VER}`);
    expect(factsText()).toBe(before);
    expect(log()).toBe("");
  }, 120_000);

  it("figures that differ from the evidence are not written, tested or deployed", () => {
    expect(release(["engine", VER, "--no-deploy"]).status).toBe(0);
    const before = factsText();
    const data = join(engine, "release_info/website_data_0.9.9_10-03-26_16-51.jsonl");
    writeFileSync(data, readFileSync(data, "utf8").replace('"median": 90.5', '"median": 91.5'));
    writeFileSync(calls, "");
    const r = release(["figures", VER]);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("median 91.5, release_info 90.5");
    expect(factsText()).toBe(before);
    expect(log()).toBe("");
  }, 120_000);

  it("a failing check stops before the deploy", () => {
    const r = release(["release", VER], { PNPM_FAIL: "lint" });
    expect(r.status).not.toBe(0);
    expect(log()).toContain("pnpm lint\n");
    expect(log()).not.toContain("pnpm typecheck");
    expect(log()).not.toContain("deploy");
    expect(log()).not.toContain("curl");
  }, 120_000);

  it("deploy refuses uncommitted changes", () => {
    expect(release(["release", VER, "--no-deploy"]).status).toBe(0);
    writeFileSync(calls, "");
    const r = release(["deploy", VER]);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("uncommitted");
    expect(r.stderr).toContain("data/facts.jsonl");
    expect(log()).toBe("");
  }, 120_000);

  it("deploy publishes the committed tree and checks the live page, without the tests", () => {
    expect(release(["release", VER, "--no-deploy"]).status).toBe(0);
    git("add", "-A");
    git("commit", "-q", "-m", "site");
    writeFileSync(calls, "");
    const r = release(["deploy", VER]);
    expect(r.status, r.stderr).toBe(0);
    expect(log()).toBe(`pnpm run deploy\ncurl -fsS https://jubarte.pro/download?release=${VER}\n`);
    expect(r.stdout).toContain("live: https://jubarte.pro");
  }, 120_000);

  it("deploy fails when the live page still shows another engine", () => {
    expect(release(["release", VER, "--no-deploy"]).status).toBe(0);
    git("add", "-A");
    git("commit", "-q", "-m", "site");
    page("<h2>Engine: jubarte-redlines 0.9.8</h2>");
    writeFileSync(calls, "");
    const r = release(["deploy", VER]);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain(`does not show engine ${VER}`);
    // LIVE_TRIES fetches, then it gives up
    expect(log().match(/^curl /gm)).toHaveLength(2);
  }, 120_000);

  it("deploy refuses a tree whose figures are not the release's", () => {
    // Committed, but only the engine half ran: the figures are still the old ones.
    expect(release(["engine", VER, "--no-deploy"]).status).toBe(0);
    git("add", "-A");
    git("commit", "-q", "-m", "site");
    writeFileSync(calls, "");
    const r = release(["deploy", VER]);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain(`scripts/release.sh figures ${VER}`);
    expect(log()).toBe("");
  }, 120_000);

  it("takes no --no-deploy with deploy, and no unknown kind", () => {
    expect(release(["deploy", VER, "--no-deploy"]).status).toBe(2);
    expect(release(["publish", VER]).status).toBe(2);
    expect(log()).toBe("");
  });
});
