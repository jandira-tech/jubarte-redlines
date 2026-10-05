// Builds public/, the folder Cloudflare serves as static assets: every page,
// the stylesheet and scripts, self-hosted fonts, the vendored jubarte-wasm
// and pdf.js, icons, the App page's sample pair and the small data files.
// The case fixtures (public/fixtures, public/data/cases-*.json) come from
// `pnpm fixtures:render` and are kept as they are.
//
//   node scripts/build.ts

import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Resvg } from "@resvg/resvg-js";
import { TABLES } from "../site/data/bench.ts";
import { DEMO_MODIFIED, DEMO_ORIGINAL, demoDocs } from "../site/demo-docs.ts";
import { ORIGIN, type Page, render, setAssetVersions, setVendorBases } from "../site/layout.ts";
import { pro } from "../site/pages/app.ts";
import { benchmarkPage } from "../site/pages/benchmark.ts";
import { useCases, useCasesEmbed } from "../site/pages/cases.ts";
import { contact } from "../site/pages/contact.ts";
import { demo } from "../site/pages/demo.ts";
import { download } from "../site/pages/download.ts";
import { homePage } from "../site/pages/home.ts";
import { notFound, privacy, terms } from "../site/pages/legal.ts";
import { live } from "../site/pages/live.ts";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const PUB = join(ROOT, "public");
const MODULES = join(ROOT, "node_modules");
const ASSETS = join(ROOT, "..", "assets");

type EngineScore = { score: number; failed: boolean; page_scores?: number[] };
type Render = { pages: number; h: number; offsets: [number, number][] };
type Case = {
  id: string;
  stem: string;
  state: string;
  engines: Record<string, EngineScore>;
  renders: Record<string, Render>;
};

const out = (rel: string, data: string | Uint8Array) => {
  const path = join(PUB, rel);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, data);
};
const pkgVersion = (name: string) =>
  (JSON.parse(readFileSync(join(MODULES, name, "package.json"), "utf8")) as { version: string })
    .version;
const hash = (data: string | Uint8Array) =>
  createHash("sha256").update(data).digest("hex").slice(0, 10);

/** Empty public/, keeping the rendered fixtures and the case lists. */
function clean(): void {
  mkdirSync(PUB, { recursive: true });
  for (const name of readdirSync(PUB)) {
    if (name === "fixtures") continue;
    if (name === "data") {
      for (const f of readdirSync(join(PUB, "data"))) {
        if (!/^cases-(convert|redline)\.json$/.test(f))
          rmSync(join(PUB, "data", f), { recursive: true });
      }
      continue;
    }
    rmSync(join(PUB, name), { recursive: true });
  }
}

function copyStatic(): void {
  cpSync(join(ROOT, "site", "static"), join(PUB, "static"), { recursive: true });
  const fonts: [string, string[]][] = [
    ["manrope", ["manrope-latin-wght-normal.woff2", "manrope-latin-ext-wght-normal.woff2"]],
    [
      "jetbrains-mono",
      ["jetbrains-mono-latin-wght-normal.woff2", "jetbrains-mono-latin-ext-wght-normal.woff2"],
    ],
    [
      "source-serif-4",
      ["source-serif-4-latin-opsz-normal.woff2", "source-serif-4-latin-ext-opsz-normal.woff2"],
    ],
  ];
  for (const [pkg, files] of fonts) {
    const dir = join(MODULES, "@fontsource-variable", pkg);
    for (const f of files) cpSync(join(dir, "files", f), join(PUB, "static", "fonts", f));
    cpSync(join(dir, "LICENSE"), join(PUB, "static", "fonts", `LICENSE-${pkg}.txt`));
  }
  const docs = demoDocs();
  out("static/demo/msa-v3.docx", docs[DEMO_ORIGINAL]);
  out("static/demo/msa-v4.docx", docs[DEMO_MODIFIED]);
}

/**
 * Pages link the stylesheet by a name that carries its content hash, so
 * /static/css/ caches for a year. The Worker's 429 page cannot know the hash
 * and links the plain copy at /static/site.css, which revalidates.
 */
function hashCss(): void {
  const css = readFileSync(join(PUB, "static/css/site.css"));
  const v = hash(css);
  renameSync(join(PUB, "static/css/site.css"), join(PUB, `static/css/site.${v}.css`));
  out("static/site.css", css);
  setAssetVersions({ "/static/css/site.css": v });
}

/** jubarte-wasm and pdf.js under versioned folders, so they cache for a year. */
function vendor(): { jubarte: string; pdfjs: string } {
  const jv = pkgVersion("jubarte-wasm");
  const jubarte = `/vendor/jubarte/${jv}`;
  const wasm = join(MODULES, "jubarte-wasm");
  for (const [build, from] of [
    ["slim", "web-slim"],
    ["full", "web"],
  ]) {
    for (const f of ["jubarte_wasm.js", "jubarte_wasm_bg.wasm"]) {
      cpSync(join(wasm, from, f), join(PUB, jubarte.slice(1), build, f));
    }
  }
  for (const f of ["LICENSE", "ENGINE_COMMIT.txt"])
    cpSync(join(wasm, f), join(PUB, jubarte.slice(1), f));

  const pv = pkgVersion("pdfjs-dist");
  const pdfjs = `/vendor/pdfjs/${pv}`;
  const pdf = join(MODULES, "pdfjs-dist");
  for (const f of ["pdf.min.mjs", "pdf.worker.min.mjs"]) {
    cpSync(join(pdf, "build", f), join(PUB, pdfjs.slice(1), "build", f));
  }
  for (const d of ["cmaps", "standard_fonts", "wasm", "iccs"]) {
    cpSync(join(pdf, d), join(PUB, pdfjs.slice(1), d), { recursive: true });
  }
  cpSync(join(pdf, "LICENSE"), join(PUB, pdfjs.slice(1), "LICENSE"));
  return { jubarte, pdfjs };
}

/** A case list and the engine labels of the run that rendered it. */
type CaseList = {
  revision: string;
  resolve: string;
  engines: Record<string, Record<string, string>>;
  cases: Case[];
};

function readCases(bench: "convert" | "redline"): CaseList {
  const path = join(PUB, "data", `cases-${bench}.json`);
  if (!existsSync(path)) {
    throw new Error(
      `${path} is missing: run \`pnpm fixtures:fetch && pnpm fixtures:render\` first`,
    );
  }
  const data = JSON.parse(readFileSync(path, "utf8")) as CaseList;
  // cases.js builds each file's URL from `resolve`; data rendered before it
  // existed carries full URLs per case instead, and its links would break.
  if (!data.resolve) {
    throw new Error(
      `${path} predates per-case file names: run \`uv run scripts/site_fixtures.py render --keep\``,
    );
  }
  return data;
}

/** The home strip's cases: page one of Word, jubarte and LibreOffice. */
function homeData(convert: Case[], revision: string): void {
  const side = (c: Case, k: string) => ({
    score: c.engines[k]?.score ?? 0,
    failed: c.engines[k]?.failed ?? true,
    pages: c.renders[k]?.pages ?? null,
  });
  const picks = convert
    .filter((c) => c.renders.word?.offsets.length && c.engines.jubarte && c.engines.soffice)
    // Only what home.js draws: the stem (a long, incompressible hash) was two
    // thirds of the file over the wire and never shown.
    .map((c) => ({
      id: c.id,
      state: c.state,
      pages: c.renders.word.pages,
      jubarte: side(c, "jubarte"),
      soffice: side(c, "soffice"),
      h: Object.fromEntries(
        ["word", "jubarte", "soffice"]
          .filter((k) => c.renders[k]?.offsets.length)
          .map((k) => [k, c.renders[k].offsets[0][1]]),
      ),
    }));
  out("data/home.json", JSON.stringify({ revision, cases: picks }));
}

/**
 * The case the benchmark page frames: a 2–4 page document whose jubarte
 * score is closest to jubarte's published median, so it is typical rather
 * than flattering. Deterministic for a given case list.
 */
function typicalCase(convert: Case[]): { bench: string; id: string } {
  const median = TABLES[0].rows.find((r) => r.ours)?.median;
  if (median === undefined) throw new Error("bench.ts: the first table has no jubarte row");
  const pool = convert.filter(
    (c) =>
      c.renders.word &&
      c.renders.word.pages >= 2 &&
      c.renders.word.pages <= 4 &&
      ["jubarte", "soffice", "docxide"].every((k) => c.engines[k] && !c.engines[k].failed),
  );
  pool.sort(
    (a, b) =>
      Math.abs(a.engines.jubarte.score - median) - Math.abs(b.engines.jubarte.score - median) ||
      a.id.localeCompare(b.id),
  );
  if (!pool.length) throw new Error("no case fits the benchmark page's embed");
  return { bench: "convert", id: pool[0].id };
}

function icons(): void {
  // The app icon keeps macOS's margin around its plate; a tab or a home screen
  // draws its own, so the favicons are the plate alone, edge to edge.
  const icon = readFileSync(join(ASSETS, "icon.svg"), "utf8").replace(
    'viewBox="0 0 1024 1024"',
    'viewBox="100 100 824 824"',
  );
  if (!icon.includes('viewBox="100 100 824 824"'))
    throw new Error("assets/icon.svg: no 1024 viewBox to crop");
  out("favicon.svg", icon);
  const png = (svg: string, width: number) =>
    new Resvg(svg, { fitTo: { mode: "width", value: width }, font: { loadSystemFonts: true } })
      .render()
      .asPng();
  out("favicon-32.png", png(icon, 32));
  out("apple-touch-icon.png", png(icon, 180));
  const whale = readFileSync(join(ASSETS, "whale.svg"), "utf8")
    .replace(/^[\s\S]*?<svg[^>]*>/, "")
    .replace(/<\/svg>\s*$/, "");
  const og = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
<rect width="1200" height="630" fill="#FFFFFF"/>
<rect x="0" y="0" width="12" height="630" fill="#1E5580"/>
<text x="84" y="128" font-family="Menlo, Monaco, monospace" font-size="22" letter-spacing="3" fill="#5A7183">JUBARTE.DOCX · REDLINES · PDF · ONE ENGINE</text>
<text x="84" y="250" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-weight="700" font-size="76" letter-spacing="-2" fill="#0B1E2D">Word-faithful redlines</text>
<text x="84" y="340" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-weight="700" font-size="76" letter-spacing="-2" fill="#0B1E2D">and rendering,</text>
<rect x="78" y="364" width="486" height="96" fill="#BFE3F2"/>
<text x="84" y="436" font-family="Helvetica Neue, Helvetica, Arial, sans-serif" font-weight="700" font-size="76" letter-spacing="-2" fill="#061C30">without Word.</text>
<text x="84" y="548" font-family="Menlo, Monaco, monospace" font-size="22" letter-spacing="2" fill="#1E5580">jubarte.pro</text>
<svg x="640" y="250" width="520" height="334" viewBox="0 0 560 360" fill="none">${whale}</svg>
</svg>`;
  out("og.png", png(og, 1200));
}

const HEADERS = `# Written by scripts/build.ts. Rules must not overlap: Cloudflare joins the
# values of every rule that matches.
/static/fonts/*
  Cache-Control: public, max-age=31536000, immutable
/static/css/*
  Cache-Control: public, max-age=31536000, immutable
/static/js/*
  Cache-Control: public, max-age=0, must-revalidate
/static/demo/*
  Cache-Control: public, max-age=86400
/vendor/*
  Cache-Control: public, max-age=31536000, immutable
/fixtures/*
  Cache-Control: public, max-age=604800
/data/*
  Cache-Control: public, max-age=0, must-revalidate
/*
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
`;

function seo(pages: Page[]): void {
  out(
    "robots.txt",
    `User-agent: *\nAllow: /\nDisallow: /use-cases/embed\n\nSitemap: ${ORIGIN}/sitemap.xml\n`,
  );
  const urls = pages
    .filter((p) => !p.noindex)
    .map((p) => `  <url><loc>${ORIGIN}${p.path === "/" ? "/" : p.path}</loc></url>`)
    .join("\n");
  out(
    "sitemap.xml",
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`,
  );
}

function main(): void {
  const t0 = performance.now();
  clean();
  copyStatic();
  setVendorBases(vendor());
  hashCss();
  const { engines, cases: convert, revision } = readCases("convert");
  readCases("redline");
  homeData(convert, revision);
  const pages = [
    homePage({ jubarte: engines.convert.jubarte, soffice: engines.convert.soffice }),
    live,
    demo,
    benchmarkPage(typicalCase(convert)),
    useCases,
    useCasesEmbed,
    pro,
    contact,
    download,
    privacy,
    terms,
    notFound,
  ];
  for (const page of pages) out(page.file, render(page));
  icons();
  out("_headers", HEADERS);
  seo(pages);
  console.log(
    `built ${pages.length} pages into public/ in ${Math.round(performance.now() - t0)} ms`,
  );
}

main();
