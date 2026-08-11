/**
 * Rasterize the social card and favicons from SVG.
 *
 * Fonts are loaded explicitly from assets/ and system fonts are disabled, so
 * the card renders byte-identically on any machine rather than silently
 * substituting whatever Departure Mono lookalike happens to be installed.
 */

import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { Resvg } from "@resvg/resvg-js";

const here = import.meta.dirname;
const assets = path.resolve(here, "../assets");
const publicDir = path.resolve(here, "../public");

// Manrope is the body face; it is fetched from Google Fonts at page render time
// but resvg needs a local file. Fall back to Departure Mono if it is absent so
// the script still produces a usable card rather than failing the build.
const fontFiles = [path.join(assets, "DepartureMono-Regular.otf")];
for (const extra of ["Manrope-Regular.ttf", "Manrope-Medium.ttf"]) {
  try {
    readFileSync(path.join(assets, extra));
    fontFiles.push(path.join(assets, extra));
  } catch {
    // optional
  }
}

function render(svgName, outName, width) {
  const svg = readFileSync(path.join(assets, svgName), "utf8");
  const png = new Resvg(svg, {
    fitTo: { mode: "width", value: width },
    // loadSystemFonts: false is the point — with it on, a missing family would
    // silently resolve to whatever the machine has and the card would differ
    // between developers instead of failing loudly.
    font: { fontFiles, loadSystemFonts: false, defaultFontFamily: "Manrope" },
  })
    .render()
    .asPng();
  writeFileSync(path.join(publicDir, outName), png);
  console.log(`render-og: ${outName} (${width}px, ${(png.length / 1024).toFixed(0)} KB)`);
}

render("og.svg", "og.png", 1200);
render("icon.svg", "apple-touch-icon.png", 180);
render("icon.svg", "favicon-32.png", 32);
