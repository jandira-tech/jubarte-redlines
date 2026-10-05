import {
  HEADLINES as BENCH_HEADLINES,
  BENCH_REPO,
  TABLES as BENCH_TABLES,
  BENCH_VERSION,
  BENCH_VIEWER,
  GENERATED,
  METHOD,
  RESULTS_URL,
  type Row,
  STATES,
  stateLabel,
} from "../data/bench.ts";
import { ENGINE_REPO } from "../data/release.ts";
import { esc, type Page } from "../layout.ts";

function row(r: Row): string {
  const cls = r.ours ? " ours" : "";
  return `<div class="t-row bench-cols${cls}" role="row">
<span class="mono muted" role="cell">${esc(r.rank)}</span>
<div class="tool" role="rowheader"><div class="tool-name">${esc(r.tool)}</div><div class="tool-pin">${esc(r.pin)}</div></div>
<div class="bar-cell" role="cell"><div class="bar${r.ours ? " hi" : ""}" aria-hidden="true"><div style="width:${r.median}%"></div></div><span class="bar-note">${esc(r.note)}</span></div>
<span class="num strong" role="cell">${r.median.toFixed(2)}</span>
<span class="num" role="cell">${r.mean.toFixed(2)}</span>
<span class="num" role="cell">${r.docs.toLocaleString("en-US")}</span>
<span class="num${r.failed > 0 ? " failed" : " muted"}" role="cell">${r.failed.toLocaleString("en-US")}</span>
</div>`;
}

// Lawyers first (PRODUCT.md): the redline figures lead, the converter's follow.
// Stable sorts, so each group keeps the order bench.ts gives it.
const isConvert = (s: string) => s.startsWith("DOCX") || s.startsWith("Source DOCX");
const REDLINE_TABLES = new Set(["redlines-sample", "accept-reject"]);
const TABLES = [...BENCH_TABLES].sort(
  (a, b) => Number(!REDLINE_TABLES.has(a.id)) - Number(!REDLINE_TABLES.has(b.id)),
);
const HEADLINES = [...BENCH_HEADLINES].sort(
  (a, b) => Number(isConvert(a.label)) - Number(isConvert(b.label)),
);

/** Filled in by the build with a representative published case. */
export function benchmarkPage(embedCase: { bench: string; id: string }): Page {
  const caseHash = `#${embedCase.bench}/${embedCase.id}`;
  const caseUrl = `/use-cases${caseHash}`;
  const body = `<main class="wrap">
<div class="intro">
<p class="eyebrow">/ BENCHMARK · NEUROTIC_DOCX_BENCH ${BENCH_VERSION} · GENERATED ${GENERATED}</p>
<h1 class="h1">Scored against Word, failures included.</h1>
<p class="lead">The oracle is Word’s own PDF export of each document (SHA-pinned). A converter’s PDF is scored 0–100 by pixel similarity; redlines are additionally opened in Word and compared against Word’s own compare. jubarte is author-affiliated (†) and plays by the same rules. Medians and means count a failed document as 0, so no tool gains by skipping a hard one.</p>
<div class="row mt-28 m-col">
<a class="btn btn-primary" href="/use-cases">Browse use cases →</a>
<a class="btn btn-outline" href="${BENCH_VIEWER}">Full viewer on GitHub Pages ↗</a>
<a class="btn btn-outline" href="${BENCH_REPO}">Harness &amp; raw JSONL ↗</a>
<a class="btn btn-text" href="/live">Live feed · coming soon →</a>
</div>
</div>

<div class="cells cols-auto-200 mt-56">
${HEADLINES.map(
  (h) => `<div class="cell-pad headline">
<p class="kicker">${esc(h.label)}</p>
<p class="headline-v"><span>${esc(h.value)}</span><span class="mono muted">vs ${esc(h.vs)}</span></p>
<p class="small mt-12">${esc(h.sub)}</p>
</div>`,
).join("\n")}
</div>

${TABLES.map(
  (t) => `<section class="mt-72" id="${t.id}">
<div class="section-head"><h2 id="${t.id}-title">${esc(t.title)}</h2><span>${esc(t.meta)}</span></div>
<p class="note mt-12">${esc(t.desc)}</p>
<div class="scroll-x"><div class="bench-table" role="table" aria-labelledby="${t.id}-title">
<div class="t-row head bench-cols" role="row"><span role="columnheader">#</span><span role="columnheader">Tool · pin</span><span role="columnheader">Median · 95% CI</span><span class="r" role="columnheader">Median (fail = 0)</span><span class="r" role="columnheader">Mean (fail = 0)</span><span class="r" role="columnheader">Scored</span><span class="r" role="columnheader">Failed</span></div>
${t.rows.map(row).join("\n")}
</div></div>
</section>`,
).join("\n")}

<section class="mt-72">
<div class="section-head"><h2>By corpus state, the 600-document conversion sample</h2><span>median · documents per state in brackets</span></div>
<div class="cells no-top cols-auto-220">
${STATES.map(
  (s) => `<div class="cell-pad">
<p class="state-name">${esc(stateLabel(s.name))} <span class="mono muted">(${s.n})</span></p>
<div class="state-rows">
${s.rows
  .map(
    (r) =>
      `<div class="state-row${r.ours ? " ours" : ""}"><div><span>${esc(r.tool)}</span><span class="mono">${r.median.toFixed(2)}</span></div><div class="bar thin${r.ours ? " hi" : ""}"><div style="width:${r.median}%"></div></div></div>`,
  )
  .join("\n")}
</div>
</div>`,
).join("\n")}
</div>
</section>

<section class="mt-72">
<div class="section-head"><h2>Per-case viewer</h2><a href="${caseUrl}">Open this case full-size →</a></div>
<p class="note mt-12">Every published case is browsable: Word’s page next to each engine’s page, with overlay, difference and multiply modes, page-count mismatches, the score table and the files themselves on Hugging Face. Below is <span class="code-inline">${esc(caseHash.slice(1))}</span> — press <span class="code-inline">r</span> inside the viewer for a random case.</p>
<div class="browser-frame mt-18">
<div class="browser-bar"><span></span><span></span><span></span><span class="url">jubarte.pro${esc(caseUrl)}</span></div>
<iframe class="case-frame" src="/use-cases/embed${caseHash}" title="Jubarte case viewer" loading="lazy"></iframe>
</div>
</section>

<section class="cells cols-auto-240 mt-72">
${METHOD.map((m) => `<div class="cell-pad"><p class="note-title">${esc(m.t)}</p><p class="small mt-12">${esc(m.d)}</p></div>`).join("\n")}
</section>

<p class="note">Figures are project-maintained and reproducible, not a neutral third-party benchmark. Competitors move: Docxodus in particular has evolved since several stamps above, so read the pins. The two headline tables are this release’s 600-item samples — state-balanced, every file’s path and sha256 listed in the engine’s <span class="code-inline">release_info/</span> — scored on ${GENERATED}. The remaining tables and the full-corpus runs live in <a href="${RESULTS_URL}">RESULTS.md</a>.</p>
</main>`;

  return {
    file: "benchmark.html",
    path: "/benchmark",
    title: "Benchmark — scored against Microsoft Word · Jubarte",
    description:
      "jubarte, LibreOffice, docxide-pdf, Docxodus and others scored against Word’s own PDF export and Word’s own compare. Failures count as zero; every case is inspectable.",
    nav: "benchmark",
    body,
    footer: [
      ["Contact", "/contact"],
      ["neurotic_docx_bench", BENCH_REPO],
      ["jubarte-redlines", ENGINE_REPO],
      ["Privacy", "/privacy"],
      ["Terms", "/terms"],
    ],
  };
}
