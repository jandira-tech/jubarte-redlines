// Appends an engine release's facts to data/facts.jsonl:
//
//   node scripts/sync-release.ts 0.10.2 [--changelog ../../CHANGELOG.md]
//
// The version, its date and its one-line summary come from the engine
// CHANGELOG (`## [x.y.z] - YYYY-MM-DD`, then the `> **Summary.**` line that
// scripts/release.sh writes); the archives and wheels, with their sizes, from
// the GitHub release's assets (`gh release view`). Only those facts change:
// the App Store listing, the emails and older releases are left as they are.
// scripts/facts.py writes the records, and only the values that changed.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { RELEASES, type Release } from "../site/data/release.ts";

export type Asset = { name: string; size: number };
export type Archive = { target: string; file: string; size: number };
export type ChangelogEntry = { date: string; summary: string };

const ARCHIVE_TARGETS: Record<string, string> = {
  "linux-x86_64": "Linux x86_64",
  "linux-aarch64": "Linux aarch64",
  "macos-aarch64": "macOS Apple silicon",
  "macos-x86_64": "macOS Intel",
  "windows-x86_64": "Windows x86_64",
};

// Wheel platform tags, most specific first.
const WHEEL_TARGETS: [RegExp, string][] = [
  [/macosx_\d+_\d+_arm64/, "Python · macOS Apple silicon"],
  [/macosx_\d+_\d+_x86_64/, "Python · macOS Intel"],
  [/manylinux_\d+_\d+_x86_64/, "Python · Linux x86_64"],
  [/manylinux_\d+_\d+_aarch64/, "Python · Linux aarch64"],
  [/musllinux_\d+_\d+_x86_64/, "Python · Linux musl x86_64"],
  [/musllinux_\d+_\d+_aarch64/, "Python · Linux musl aarch64"],
  [/win_amd64/, "Python · Windows x86_64"],
];

/** The CLI archives of `version`, in ARCHIVE_TARGETS order; unknown targets fail. */
export function archivesFrom(version: string, assets: Asset[]): Archive[] {
  const prefix = `jubarte-${version}-`;
  const found = assets
    .filter((a) => a.name.startsWith(prefix) && /\.(tar\.gz|zip)$/.test(a.name))
    .map((a) => {
      const key = a.name.slice(prefix.length).replace(/\.(tar\.gz|zip)$/, "");
      const target = ARCHIVE_TARGETS[key];
      if (!target) throw new Error(`unknown archive target in ${a.name}`);
      return { target, file: a.name, size: a.size };
    });
  const order = Object.values(ARCHIVE_TARGETS);
  return found.sort((a, b) => order.indexOf(a.target) - order.indexOf(b.target));
}

/** The Python wheels of `version`, then its sdist; unknown platform tags fail. */
export function wheelsFrom(version: string, assets: Asset[]): Archive[] {
  const prefix = `jubarte_redlines-${version}-`;
  const wheels = assets
    .filter((a) => a.name.startsWith(prefix) && a.name.endsWith(".whl"))
    .map((a) => {
      const hit = WHEEL_TARGETS.find(([re]) => re.test(a.name));
      if (!hit) throw new Error(`unknown wheel platform in ${a.name}`);
      return { target: hit[1], file: a.name, size: a.size };
    });
  const order = WHEEL_TARGETS.map(([, t]) => t);
  wheels.sort((a, b) => order.indexOf(a.target) - order.indexOf(b.target));
  const sdist = assets.find((a) => a.name === `jubarte_redlines-${version}.tar.gz`);
  return sdist
    ? [...wheels, { target: "Python · source", file: sdist.name, size: sdist.size }]
    : wheels;
}

/** The dated section of `version` and the first sentence of its summary. */
export function changelogEntry(changelog: string, version: string): ChangelogEntry {
  const esc = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const head = new RegExp(`^## \\[${esc}\\] - (\\d{4}-\\d{2}-\\d{2})$`, "m").exec(changelog);
  if (!head) throw new Error(`CHANGELOG has no dated ## [${version}] section`);
  const rest = changelog.slice(head.index + head[0].length);
  const next = rest.search(/^## \[/m);
  const section = next < 0 ? rest : rest.slice(0, next);
  const line = /^> \*\*Summary\.\*\* (.+)$/m.exec(section);
  if (!line) throw new Error(`CHANGELOG ## [${version}] has no "> **Summary.**" line`);
  // The release list shows one sentence; the summary runs on in clauses.
  const first = line[1]
    .split(/(?<=\.)\s/)[0]
    .split("; ")[0]
    .trim()
    .replace(/[.;]?$/, ".");
  return { date: head[1], summary: first };
}

/** The facts `version` changes, keyed as data/facts.jsonl keys them. */
export function releaseFacts(
  version: string,
  entry: ChangelogEntry,
  archives: Archive[],
  wheels: Archive[],
  history: Release[],
): Record<string, unknown> {
  if (!archives.length) throw new Error(`release v${version} has no CLI archives`);
  const listed = history.some((r) => r.v === version);
  return {
    "engine.version": version,
    "engine.released": entry.date,
    "release.archives": archives,
    "release.wheels": wheels,
    // A new release heads the list; one already listed keeps its line.
    "release.history": listed
      ? history
      : [{ v: version, d: entry.date, t: entry.summary }, ...history],
  };
}

function main(argv: string[]): void {
  const version = argv[0];
  if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) {
    throw new Error("usage: node scripts/sync-release.ts <x.y.z> [--changelog PATH]");
  }
  const here = dirname(fileURLToPath(import.meta.url));
  const flag = argv.indexOf("--changelog");
  const changelog = flag > 0 ? argv[flag + 1] : join(here, "../../../CHANGELOG.md");
  const entry = changelogEntry(readFileSync(changelog, "utf8"), version);
  const assets = JSON.parse(
    execFileSync(
      "gh",
      [
        "release",
        "view",
        `v${version}`,
        "-R",
        "jandira-tech/jubarte-redlines",
        "--json",
        "assets",
        "-q",
        ".assets",
      ],
      { encoding: "utf8" },
    ),
  ) as Asset[];
  const facts = releaseFacts(
    version,
    entry,
    archivesFrom(version, assets),
    wheelsFrom(version, assets),
    RELEASES,
  );
  const out = execFileSync(
    "uv",
    [
      "run",
      "--python",
      "3.14",
      join(here, "../../scripts/facts.py"),
      "merge",
      "-",
      "--source",
      `GitHub release v${version} assets and the engine CHANGELOG`,
    ],
    { encoding: "utf8", input: JSON.stringify(facts) },
  );
  process.stdout.write(out);
  console.log(`data/facts.jsonl → ${version} (${entry.date})`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2));
}
