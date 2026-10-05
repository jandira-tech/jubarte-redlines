import { describe, expect, it } from "vitest";
import {
  archivesFrom,
  changelogEntry,
  releaseFacts,
  wheelsFrom,
} from "../../scripts/sync-release.ts";

// The release list as of the 0.10.1 sync, frozen: the tests must not move when
// data/facts.jsonl moves to the next release (release.sh engine runs them).
const HISTORY_0_10_1 = [
  { v: "0.10.1", d: "2026-09-30", t: "Accept or reject one tracked change at a time." },
  { v: "0.10.0", d: "2026-09-28", t: "Agent editing surface." },
];

// The v0.10.1 GitHub release, as `gh release view --json assets` lists it.
const ASSETS = [
  { name: "jubarte-0.10.1-linux-aarch64.tar.gz", size: 12310041 },
  { name: "jubarte-0.10.1-linux-x86_64.tar.gz", size: 12433381 },
  { name: "jubarte-0.10.1-macos-aarch64.tar.gz", size: 11591475 },
  { name: "jubarte-0.10.1-macos-x86_64.tar.gz", size: 12044995 },
  { name: "jubarte_redlines-0.10.1-cp310-abi3-macosx_10_12_x86_64.whl", size: 8520545 },
  { name: "jubarte_redlines-0.10.1-cp310-abi3-macosx_11_0_arm64.whl", size: 8216032 },
  { name: "jubarte_redlines-0.10.1-cp310-abi3-manylinux_2_34_aarch64.whl", size: 8437047 },
  { name: "jubarte_redlines-0.10.1-cp310-abi3-manylinux_2_34_x86_64.whl", size: 8704450 },
  { name: "jubarte_redlines-0.10.1.tar.gz", size: 5497087 },
  { name: "SHA256SUMS.txt", size: 1006 },
];

const CHANGELOG = `# Changelog

## [0.10.2] - 2026-10-09

> **Summary.** Faster compares on long tables; a moved row keeps its comments. Docs too.
>
> **Docs.** README.

### Fixed
- things

## [0.10.1] - 2026-09-30

> **Summary.** Older.
`;

const as = (v: string) => ASSETS.map((a) => ({ ...a, name: a.name.replaceAll("0.10.1", v) }));

describe("sync-release", () => {
  it("lists the archives in page order with their sizes", () => {
    expect(archivesFrom("0.10.1", ASSETS).map((a) => a.target)).toEqual([
      "Linux x86_64",
      "Linux aarch64",
      "macOS Apple silicon",
      "macOS Intel",
    ]);
    expect(archivesFrom("0.10.1", ASSETS)[0]).toEqual({
      target: "Linux x86_64",
      file: "jubarte-0.10.1-linux-x86_64.tar.gz",
      size: 12433381,
    });
  });

  it("refuses an archive target the page has no name for", () => {
    expect(() =>
      archivesFrom("1.0.0", [{ name: "jubarte-1.0.0-freebsd-x86_64.tar.gz", size: 1 }]),
    ).toThrow("unknown archive target");
  });

  it("lists the wheels in page order, then the sdist", () => {
    expect(wheelsFrom("0.10.1", ASSETS).map((w) => w.target)).toEqual([
      "Python · macOS Apple silicon",
      "Python · macOS Intel",
      "Python · Linux x86_64",
      "Python · Linux aarch64",
      "Python · source",
    ]);
  });

  it("names the musl wheels the release workflow builds since 0.10.1", () => {
    // .github/workflows/release.yml builds musllinux_1_2 wheels, and
    // check_release_artifacts.py requires them; an unknown tag would stop
    // the downstream step of every release after 0.10.1.
    const musl = [
      { name: "jubarte_redlines-0.10.2-cp310-abi3-musllinux_1_2_aarch64.whl", size: 2 },
      { name: "jubarte_redlines-0.10.2-cp310-abi3-musllinux_1_2_x86_64.whl", size: 1 },
      { name: "jubarte_redlines-0.10.2-cp310-abi3-win_amd64.whl", size: 3 },
    ];
    expect(wheelsFrom("0.10.2", [...as("0.10.2"), ...musl]).map((w) => w.target)).toEqual([
      "Python · macOS Apple silicon",
      "Python · macOS Intel",
      "Python · Linux x86_64",
      "Python · Linux aarch64",
      "Python · Linux musl x86_64",
      "Python · Linux musl aarch64",
      "Python · Windows x86_64",
      "Python · source",
    ]);
  });

  it("takes the date and the first clause of the release summary", () => {
    expect(changelogEntry(CHANGELOG, "0.10.2")).toEqual({
      date: "2026-10-09",
      summary: "Faster compares on long tables.",
    });
    expect(changelogEntry(CHANGELOG, "0.10.1").summary).toBe("Older.");
    expect(() => changelogEntry(CHANGELOG, "0.9.0")).toThrow("no dated");
  });

  it("reads the version as text, not as a pattern", () => {
    const md = "## [1.0.0+b1] - 2026-10-02\n\n> **Summary.** Built.\n";
    expect(changelogEntry(md, "1.0.0+b1").date).toBe("2026-10-02");
    expect(() => changelogEntry("## [1x0x0] - 2026-10-02\n", "1.0.0")).toThrow("no dated");
  });

  it("gives the new release's facts, heading the release list once", () => {
    const entry = changelogEntry(CHANGELOG, "0.10.2");
    const facts = releaseFacts(
      "0.10.2",
      entry,
      archivesFrom("0.10.2", as("0.10.2")),
      wheelsFrom("0.10.2", as("0.10.2")),
      HISTORY_0_10_1,
    );
    expect(Object.keys(facts)).toEqual([
      "engine.version",
      "engine.released",
      "release.archives",
      "release.wheels",
      "release.history",
    ]);
    expect(facts["engine.version"]).toBe("0.10.2");
    expect(facts["engine.released"]).toBe("2026-10-09");
    const files = (facts["release.archives"] as { file: string }[]).map((a) => a.file);
    expect(files).toContain("jubarte-0.10.2-macos-aarch64.tar.gz");
    expect(files.some((f) => f.includes("0.10.1"))).toBe(false);
    const history = facts["release.history"] as { v: string }[];
    expect(history.map((r) => r.v)).toEqual(["0.10.2", "0.10.1", "0.10.0"]);
    // Synced again, the list does not grow: facts.py then appends nothing.
    const again = releaseFacts(
      "0.10.2",
      entry,
      archivesFrom("0.10.2", as("0.10.2")),
      wheelsFrom("0.10.2", as("0.10.2")),
      history as typeof HISTORY_0_10_1,
    );
    expect(again).toEqual(facts);
  });

  it("refuses a release without CLI archives", () => {
    expect(() =>
      releaseFacts("0.10.2", { date: "2026-10-09", summary: "x" }, [], [], HISTORY_0_10_1),
    ).toThrow("no CLI archives");
  });
});
