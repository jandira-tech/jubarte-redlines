import { command } from "../command.ts";
import { HOME_GROUPS } from "../data/bench.ts";
import { APP_STORE, ENGINE_VERSION } from "../data/release.ts";
import { esc, type Page, watermark } from "../layout.ts";

const CAPS = [
  {
    tag: "Compare",
    title: "DOCX → tracked DOCX",
    desc: "Insertions, deletions, moves and formatting changes as native Word revisions, on the original package, with styles, headers, notes and media intact.",
    cmd: "jubarte original.docx modified.docx -o redline.docx",
    href: "/demo",
  },
  {
    tag: "Resolve",
    title: "Accept or reject, by ID",
    desc: "List every change with a stable ID, then accept or reject globally or by ID, author and kind. Both sides of a move go together.",
    cmd: "jubarte accept redline.docx --id body:rev:12 -o accepted.docx",
    href: "/download",
  },
  {
    tag: "Edit",
    title: "Atomic JSON edit plans",
    desc: "Inspect paragraphs by stable story ID, apply replace / insert / comment plans against a source hash. Ambiguity rejects the whole plan.",
    cmd: "jubarte edit contract.docx --plan plan.json --out-dir out/",
    href: "/download",
  },
  {
    tag: "Render",
    title: "DOCX → PDF / PNG",
    desc: "An independent Word-oriented layout engine. Tracked changes painted in conventional, Word-like or custom palettes. No Office process launched.",
    cmd: "jubarte convert redline.docx --revisions word --compress",
    href: "/demo#convert",
  },
];

const SURFACES = [
  {
    id: "cli",
    tag: "CLI",
    title: "The CLI",
    cmd: "cargo install jubarte-redlines\njubarte --version",
    note: "Compare, changes, accept/reject, convert, inspect, text, edit, debug. Prebuilt archives on GitHub Releases.",
  },
  {
    id: "rust",
    tag: "Rust",
    title: "The library crate",
    cmd: "cargo add jubarte-redlines \\\n  --no-default-features",
    note: "Import path is jubarte::. No unsafe. MSRV 1.88, edition 2024.",
  },
  {
    id: "python",
    tag: "Python",
    title: "Python wheels",
    cmd: "pip install jubarte-redlines",
    note: "Immutable Document API: compare, changes, to_pdf, to_png, inspect, edit, preview. PyO3 / abi3 wheels.",
  },
  {
    id: "node",
    tag: "Node · browser",
    title: "WebAssembly packages",
    cmd: "npm install jubarte-wasm",
    note: "Full and slim builds for Node and the browser. The demo on this site runs both: slim to compare, full to render PDF.",
  },
];

const NOTES = [
  {
    t: "Real tracked changes",
    d: "The redline is a Word document with genuine insertions and deletions. Accept, reject and filter them by author in Word’s Review tab. It is not a side-by-side view and not a PDF overlay.",
  },
  {
    t: "Nothing leaves the machine",
    d: "The browser demo runs entirely inside the tab. The Mac app compares on your disk. There is no upload step to audit: load the page, go offline, and it still works.",
  },
  {
    t: "Opened in Word before it is scored",
    d: "In the benchmark, every redline is opened in Microsoft Word itself before it earns a score, and the scoreboard below counts every failure as zero.",
  },
];

// Lawyers first (PRODUCT.md): the redline scores lead the scoreboard.
const GROUPS = [...HOME_GROUPS].sort(
  (a, b) => Number(a.title.startsWith("DOCX")) - Number(b.title.startsWith("DOCX")),
);

const DEV_NOTE = {
  t: "Fail closed",
  d: 'Edit plans that hit an ambiguous anchor or a stale source hash write nothing. Output is genuine <span class="code-inline">w:ins</span> / <span class="code-inline">w:del</span> revisions, the same ones Word writes.',
};

/** A marked-up clause on paper: what a lawyer receives. Deletion first, as Word orders them. */
const REDLINE = `<figure class="hero-redline">
<div class="paper redline-paper">
<p class="doc-title">MUTUAL NON-DISCLOSURE AGREEMENT</p>
<p class="doc-sub">Draft 2 · revisions by Counterparty</p>
<p class="doc-h">7. Governing law, survival and notice</p>
<p>This Agreement is governed by the laws of the State of <del>New York</del><ins>Delaware</ins>. The duty of confidence survives for <del>two (2)</del><ins>three (3)</ins> years from disclosure, and either party may end the Agreement on <del>thirty (30)</del><ins>sixty (60)</ins> days’ written notice.</p>
</div>
<figcaption><span class="tag ins">3 insertions</span><span class="tag del">3 deletions</span><span class="redline-note">Tracked changes in the usual blue and red. An illustrative clause, made up for this page.</span></figcaption>
</figure>`;

function sheet(engine: string, label: string, hi = false): string {
  return `<figure class="strip-col">
<figcaption class="kicker${hi ? " blue" : " ink"}">${label}</figcaption>
<div class="sheet" data-engine="${engine}"><div class="pending"><span>${engine === "word" ? "loading…" : "queued"}</span></div></div>
</figure>`;
}

/** The engines that rendered the fixtures, as their manifest names them. */
export type FixtureLabels = { jubarte: string; soffice: string };

const body = (fx: FixtureLabels): string => `<main>
<section class="hero">
${watermark("hero-mark")}
<div class="wrap hero-grid m-stack">
<div>
<p class="eyebrow rule-eyebrow"><span></span>DOCX · TRACKED CHANGES · PDF · ONE ENGINE</p>
<h1 class="hero-h1">Word-faithful documents. <span class="hl">Without Word.</span></h1>
<p class="hero-lead">Compare two Word documents and get a real redline: tracked changes Word accepts, rejects and filters as its own. Use the Mac app or your browser; developers can embed the same engine.</p>
<div class="row mt-36 m-col">
<a class="btn btn-primary btn-lg" href="/demo">Try it in your browser →</a>
<a class="btn btn-outline btn-lg" href="/download">Download for Mac</a>
<a class="btn btn-text btn-lg" href="/benchmark">See the benchmark →</a>
</div>
<p class="hero-meta"><span>Browser demo free</span><span>Mac app ${APP_STORE.price}</span><span>jubarte-redlines ${ENGINE_VERSION} · AGPL-3.0</span></p>
</div>
${REDLINE}
</div>
</section>

<section class="band">
<div class="wrap band-pad-md">
<div class="cells cols-auto-260">
${NOTES.map(
  (n) =>
    `<div class="cell-pad"><p class="note-title">${n.t}</p><p class="small mt-12">${n.d}</p></div>`,
).join("\n")}
</div>
</div>
</section>

<section class="band paper-band" id="feed">
<div class="wrap band-pad-sm">
<div class="section-head"><h2>From the benchmark: a random document against Word</h2><span class="row gap-12"><span class="feed-refresh" id="feed-refresh" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 11a8 8 0 0 0-14.6-4.5L4 8"/><path d="M4 3v5h5"/><path d="M4 13a8 8 0 0 0 14.6 4.5L20 16"/><path d="M20 21v-5h-5"/></svg></span><button type="button" class="chip-btn" id="feed-pause" aria-pressed="false">pause</button><a href="/live">Live feed · coming soon →</a></span></div>
<div class="feed-grid m-stack">
<div>
<p class="kicker">Fixture</p>
<p class="feed-path" id="fx-path">loading…</p>
<div class="tags mt-12"><span class="tag state" id="fx-state">—</span><span class="tag" id="fx-pages">—</span></div>
<dl class="feed-scores">
<div><dt>Word · oracle</dt><dd>100</dd></div>
<div class="ours"><dt>${esc(fx.jubarte)}</dt><dd><span id="fx-jub">—</span> <span class="muted" id="fx-jub-pp">· …</span></dd></div>
<div><dt>${esc(fx.soffice)}</dt><dd><span id="fx-sof">—</span> <span class="muted" id="fx-sof-pp">· …</span></dd></div>
</dl>
<p class="small mt-18">Session: <span class="ink-strong" id="fx-session">first pass…</span></p>
<p class="small mt-12"><a id="fx-case" href="/use-cases">Open this case in the viewer →</a></p>
</div>
<div class="feed-sheets m-three">
${sheet("word", "Word · oracle")}
${sheet("jubarte", "jubarte", true)}
${sheet("soffice", "LibreOffice")}
</div>
</div>
<p class="small mt-24 strip-note">Page one of a document from the published set, as each engine rendered it in the benchmark run. The pick is random, not chosen: where jubarte does worse than LibreOffice, that shows here too. Scores are pixel scorer v1 against Word’s own export, 0–100. The always-on live feed is <a href="/live">coming soon</a>.</p>
</div>
</section>

<section class="band">
<div class="wrap band-pad bench-grid m-stack">
<div>
<h2 class="h2">Not a claim. A scoreboard.</h2>
<p class="body-muted mt-18">Every number below comes from <span class="code-inline">neurotic_docx_bench</span>: Word’s own PDF export is the reference, failures count as zero, and jubarte is marked author-affiliated and held to the same rules.</p>
<a class="btn btn-outline mt-24" href="/benchmark">Full results &amp; per-case viewer →</a>
</div>
<div class="bench-groups">
${GROUPS.map(
  (g) => `<div>
<div class="group-head"><span>${g.title}</span><span class="kicker">${g.meta}</span></div>
<div class="bars">
${g.rows
  .map(
    (r) =>
      `<div class="bar-row${"ours" in r && r.ours ? " ours" : ""}"><span>${r.name}</span><div class="bar${"ours" in r && r.ours ? " hi" : ""}"><div style="width:${r.v}%"></div></div><span class="num">${r.v.toFixed(2)}</span></div>`,
  )
  .join("\n")}
</div>${g.note ? `\n<p class="small mt-12">${g.note}</p>` : ""}
</div>`,
).join("\n")}
</div>
</div>
</section>

<section class="band dev-head" id="developers">
<div class="wrap band-pad-sm">
<div class="section-head"><h2>For developers</h2><span>the same engine as a CLI, a Rust crate, Python wheels and WebAssembly</span></div>
</div>
</section>

<section class="band">
<div class="wrap band-pad">
<div class="section-head mb-44"><h2>One engine, four jobs</h2><span>compare → resolve → edit → render</span></div>
<div class="cells cols-auto-200">
${CAPS.map(
  (c) => `<a class="cap" href="${c.href}">
<span class="cap-tag">${c.tag}</span>
<span class="cap-title">${c.title}</span>
<span class="cap-desc">${c.desc}</span>
<code class="cap-cmd">${command(c.cmd)}</code>
</a>`,
).join("\n")}
</div>
</div>
</section>

<section class="band paper-band">
<div class="wrap band-pad">
<div class="section-head mb-44"><h2>Same core, wherever you run</h2><span>Rust 1.88 · CPython ≥ 3.10 · Node ≥ 18</span></div>
<div class="installer">
<div class="tabs installer-tabs" role="tablist" aria-label="Install jubarte">
${SURFACES.map(
  (s, i) =>
    `<button type="button" role="tab" id="inst-tab-${s.id}" aria-controls="inst-${s.id}" aria-selected="${i === 0}"${i === 0 ? "" : ' tabindex="-1"'}>${s.tag}</button>`,
).join("")}
</div>
${SURFACES.map(
  (
    s,
    i,
  ) => `<div class="installer-panel" role="tabpanel" id="inst-${s.id}" aria-labelledby="inst-tab-${s.id}" tabindex="0"${i === 0 ? "" : " hidden"}>
<pre class="code-block">${command(s.cmd)}</pre>
<p class="small mt-12"><strong>${s.title}.</strong> ${s.note}</p>
</div>`,
).join("\n")}
</div>
<div class="row mt-36"><a class="btn btn-primary btn-lg" href="/download">All downloads →</a><a class="btn btn-outline btn-lg" href="/pro">See Jubarte PRO</a></div>
</div>
</section>

<section class="band">
<div class="wrap band-pad-md">
<div class="cell-pad"><p class="note-title">${DEV_NOTE.t}</p><p class="small mt-12">${DEV_NOTE.d}</p></div>
</div>
</section>
</main>`;

/** The home page; the build passes the fixture engines from cases-convert.json. */
export const homePage = (fx: FixtureLabels): Page => ({
  file: "index.html",
  path: "/",
  title: "Jubarte — Word-faithful redlines and rendering, without Word",
  description:
    "Compare two Word documents into a real redline: tracked changes Word treats as its own. Resolve them, apply edit plans, and render to PDF, in the Mac app, the browser, or from Rust, Python and Node. Scored against Microsoft Word.",
  nav: "home",
  body: body(fx),
  scripts: ["home.js"],
  footer: [
    ["Contact", "/contact"],
    ["Benchmark", "/benchmark"],
    ["GitHub", "https://github.com/jandira-tech/jubarte-redlines"],
    ["Privacy", "/privacy"],
    ["Terms", "/terms"],
  ],
});
