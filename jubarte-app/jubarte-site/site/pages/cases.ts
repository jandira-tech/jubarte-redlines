import { BENCH_REPO } from "../data/bench.ts";
import type { Page } from "../layout.ts";

const BUCKETS: [string, string][] = [
  ["any", "any"],
  ["1", "1"],
  ["2-3", "2–3"],
  ["4-10", "4–10"],
  ["11+", "11+"],
];

const KEYS: [string, string][] = [
  ["← →", "previous / next case"],
  ["r", "random case"],
  ["s", "side panel"],
  ["o", "cycle overlay mode"],
  ["[ ]", "previous / next page"],
  ["m", "more pages"],
  ["t", "scores table"],
];

const OVERLAYS: [string, string][] = [
  ["side", "side by side"],
  ["under", "under"],
  ["difference", "difference"],
  ["multiply", "multiply"],
];

function viewer(embed: boolean): string {
  return `<div class="cases${embed ? " embed" : ""}" id="viewer" data-embed="${embed}">
<div class="toolbar"><div class="wrap toolbar-in">
<div class="seg seg-dark tb-bench" role="group" aria-label="Benchmark">
<button type="button" id="bench-redline" data-bench="redline" aria-pressed="true">Compare (redline)</button>
<button type="button" id="bench-convert" data-bench="convert" aria-pressed="false">Convert DOCX → PDF</button>
</div>
<div class="row gap-8 tb-nav">
<button type="button" class="chip-btn" id="prev">‹ prev</button>
<button type="button" class="chip-btn dark" id="random">random<span class="key-hint"> · r</span></button>
<button type="button" class="chip-btn" id="next">next ›</button>
<span class="mono muted small-mono" id="pos"></span>
</div>
<div class="row gap-8 tb-views">
<button type="button" class="chip-btn" id="toggle-side" aria-pressed="${!embed}" aria-controls="side">filters<span class="key-hint"> · s</span></button>
<button type="button" class="chip-btn" id="toggle-scores" aria-pressed="${!embed}" aria-controls="scores">scores<span class="key-hint"> · t</span></button>
<button type="button" class="chip-btn" id="toggle-keys" aria-pressed="true" title="Single-key shortcuts: r s t o m [ ] ← →">keys</button>
${embed ? '<a class="chip-btn" id="open-full" href="/use-cases" target="_top">open ↗</a>' : ""}
</div>
</div></div>

<main class="wrap cases-grid${embed ? "" : " with-side"}" id="cases-main" tabindex="-1">
<h1 class="sr-only">Cases: Word’s page beside jubarte’s and LibreOffice’s, with the scores</h1>
<aside class="side" id="side"${embed ? " hidden" : ""} aria-label="Filters">
<div class="side-head"><span>/ Filters</span><button type="button" class="btn-text small-mono" id="clear">clear</button></div>
<div class="side-sec"><p class="label">Group</p><div class="side-list" id="groups"></div></div>
<div class="side-sec"><p class="label">Pages in Word</p><div class="buckets" id="buckets">${BUCKETS.map(([k, l]) => `<button type="button" data-bucket="${k}" aria-pressed="${k === "any"}">${l}</button>`).join("")}</div></div>
<div class="side-sec"><p class="label row-between"><span>jubarte score ≤</span><span class="ink" id="max-score-v">100</span></p><input type="range" id="max-score" min="0" max="100" value="100" aria-label="Maximum jubarte score"></div>
<div class="side-sec"><label class="check"><input type="checkbox" id="differ">Page count differs from Word</label></div>
<div class="side-sec"><p class="label">Engines shown</p><div class="side-list" id="engines"></div></div>
<div class="side-sec"><p class="label">Keys</p><dl class="keys">${KEYS.map(([k, d]) => `<dt>${k}</dt><dd>${d}</dd>`).join("")}</dl></div>
</aside>

<section class="stage">
<p class="sr-only" id="case-live" role="status"></p>
<div class="stage-head m-col">
<div class="stage-id">
<p class="label" id="case-kicker">loading cases…</p>
<p class="case-path" id="case-path"></p>
<div class="tags mt-12">
<span class="tag state" id="case-state"></span>
<span class="tag" id="case-pages"></span>
<a class="tag link" id="case-link" href="#">#permalink</a>
<a class="tag link" id="case-folder" href="#" target="_blank" rel="noopener">files on Hugging Face ↗</a>
</div>
</div>
<div class="stage-ctl">
<div class="seg seg-dark mono-seg" role="group" aria-label="Overlay mode" id="overlays">${OVERLAYS.map(([k, l]) => `<button type="button" data-ov="${k}" aria-pressed="${k === "side"}">${l}</button>`).join("")}</div>
<div class="row gap-10 zoom-row"><label class="label" for="slider" id="slider-label">zoom</label><input type="range" id="slider" min="0" max="100" value="50"><span class="mono slider-v" id="slider-v">100%</span></div>
<div class="row gap-10 page-nav"><button type="button" class="chip-btn" id="page-prev" aria-label="Previous page">‹</button><span class="mono" id="page-label">p. 1</span><button type="button" class="chip-btn" id="page-next" aria-label="Next page">›</button></div>
</div>
</div>

<p class="empty-filter" id="no-match" hidden>No published case matches these filters. <button type="button" class="btn-text" id="clear-2">Clear filters</button></p>
<div class="case-pages" id="pages"></div>

<div class="more" id="more" hidden>
<p class="label">All drawn pages · Word vs jubarte · click to view</p>
<div class="more-strip" id="more-strip"></div>
</div>
<button type="button" class="btn-text small-mono mt-18" id="toggle-more" aria-controls="more" aria-expanded="false">more pages<span class="key-hint"> · m</span></button>

<section class="scores" id="scores"${embed ? " hidden" : ""}>
<div class="section-head"><h2 id="scores-title">Scores for this case</h2><span>pixel score · ink overlap · text match</span></div>
<div class="scroll-x"><div class="score-table" role="table" aria-labelledby="scores-title">
<div class="t-row head score-cols" role="row"><span role="columnheader">Engine</span><span class="r" role="columnheader">Pages</span><span class="r" role="columnheader">Score</span><span class="r" role="columnheader">Ink overlap</span><span class="r" role="columnheader">Text match</span><span class="r" role="columnheader">Files</span></div>
<div id="score-rows" role="rowgroup"></div>
</div></div>
</section>

<p class="note" id="cases-foot">Pages are WebP renders of the PDFs each engine produced in the benchmark run (at most the first 12 per document); scores are the benchmark’s own. Every file — the source documents, Word’s references and each engine’s output — is linked at the revision this site was built from on Hugging Face. Harness: <a href="${BENCH_REPO}" target="_top">neurotic_docx_bench</a>.</p>
</section>
</main>
</div>`;
}

export const useCases: Page = {
  file: "use-cases.html",
  path: "/use-cases",
  title: "Use cases — every benchmark document, page by page · Jubarte",
  description:
    "Browse the published neurotic_docx_bench cases: Word’s page beside jubarte, LibreOffice, docxide-pdf, Docxodus and superdoc, with overlays, filters, scores and the original files on Hugging Face.",
  nav: "usecases",
  body: viewer(false),
  scripts: ["cases.js"],
  footer: [
    ["Contact", "/contact"],
    ["Results", "/benchmark"],
    ["GitHub", BENCH_REPO],
    ["Privacy", "/privacy"],
    ["Terms", "/terms"],
  ],
  credit: "© MMXXVI Jandira Technologies · neurotic_docx_bench",
};

export const useCasesEmbed: Page = {
  file: "use-cases/embed.html",
  path: "/use-cases/embed",
  title: "Case viewer · Jubarte",
  description: "Embedded neurotic_docx_bench case viewer.",
  nav: "usecases",
  body: viewer(true),
  scripts: ["cases.js"],
  noindex: true,
  bare: true,
};
