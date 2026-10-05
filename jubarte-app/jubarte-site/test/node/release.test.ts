import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { ARCHIVES, ENGINE_VERSION, RELEASES, size, WHEELS } from "../../site/data/release.ts";

describe("release facts", () => {
  it("runs the demo on the engine release the site offers", () => {
    const pkg = JSON.parse(readFileSync(new URL("../../package.json", import.meta.url), "utf8"));
    expect(pkg.devDependencies["jubarte-wasm"]).toBe(ENGINE_VERSION);
    const exclude = readFileSync(new URL("../../pnpm-workspace.yaml", import.meta.url), "utf8");
    expect(exclude).toContain(`jubarte-wasm@${ENGINE_VERSION}`);
  });

  it("lists only that release's files and puts it first in the history", () => {
    for (const a of [...ARCHIVES, ...WHEELS]) expect(a.file).toContain(`-${ENGINE_VERSION}`);
    expect(RELEASES[0].v).toBe(ENGINE_VERSION);
  });
});

describe("size", () => {
  it("prints bytes, KB and MB on binary boundaries", () => {
    expect(size(0)).toBe("0 B");
    expect(size(1023)).toBe("1023 B");
    expect(size(1024)).toBe("1 KB");
    expect(size(1048063)).toBe("1023 KB");
    expect(size(1048575)).toBe("1.0 MB");
    expect(size(1048576)).toBe("1.0 MB");
    expect(size(12433381)).toBe("11.9 MB");
  });
});
