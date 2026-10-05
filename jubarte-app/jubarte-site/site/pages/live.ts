import { SUPPORT_EMAIL } from "../data/release.ts";
import { type Page, watermark, whale } from "../layout.ts";

const PHASES = [
  [
    "Draw a fixture",
    'A random document from the 6,427-document <code class="code-inline">corpus/word</code> set.',
  ],
  ["Render with jubarte", "The current release, in-process, timed."],
  [
    "Render with LibreOffice",
    '<code class="code-inline">soffice --headless</code> on the same file, timed.',
  ],
  ["Score against Word", "Both PDFs against Word’s own export, page by page."],
];

const notify = `mailto:${SUPPORT_EMAIL}?subject=${encodeURIComponent("Live bench — tell me when it is on")}&body=${encodeURIComponent("Please email me when the live bench is running.")}`;

const body = `<div class="page-bg">
${watermark()}
<main class="wrap">
<div class="page-head head-rule">
<div>
<p class="eyebrow">/ LIVE BENCH · CORPUS/WORD · ORACLE: MICROSOFT WORD</p>
<h1 class="h1">Random documents, rendered three ways.</h1>
<p class="lead">Each pass will draw a fixture from the 6,427-document <span class="code-inline">corpus/word</span> set, render it with jubarte and LibreOffice, and score both against Word’s own PDF export — running all the time, in front of you.</p>
</div>
<span class="pill">Coming soon</span>
</div>

<div class="cells live-soon m-stack">
<div class="cell-pad live-whale">
${whale("lv", { light: true, cls: "whale-float", style: "width:180px;height:auto" })}
<p class="soon-title">Live is coming soon.</p>
<p class="small mt-12">The feed needs a renderer that runs around the clock and scores in the open; we are not going to fake one in the meantime. Everything it will show is already published: the same documents, the same engines and the same scorer.</p>
<div class="row mt-24 m-col">
<a class="btn btn-primary" href="/use-cases">Browse every use case →</a>
<a class="btn btn-outline" href="/benchmark">See the benchmark</a>
<a class="btn btn-text" href="${notify}">Email me when it’s on</a>
</div>
</div>
<div class="cell-pad">
<p class="kicker">What each pass will do</p>
<ol class="phase-list mt-12">
${PHASES.map(
  ([t, d], i) =>
    `<li><span class="phase-n">0${i + 1}</span><span><strong>${t}</strong><span class="small">${d}</span></span></li>`,
).join("\n")}
</ol>
<p class="small mt-18">Until then the home page shows a random published case with its real scores, and <a href="/demo">the demo</a> runs the engine on your own files.</p>
</div>
</div>
</main>
</div>`;

export const live: Page = {
  file: "live.html",
  path: "/live",
  title: "Live bench — coming soon · Jubarte",
  description:
    "A random Word document, rendered by jubarte and LibreOffice and scored against Word, every few seconds. Coming soon; the scored cases are browsable now.",
  nav: "live",
  body,
};
