import { COMPANY, COUNSEL_EMAIL, ENGINE_REPO, SUPPORT_EMAIL } from "../data/release.ts";
import type { Page } from "../layout.ts";

const TOPICS: [string, string, boolean][] = [
  ["redline", "Redlines in Word", true],
  ["convert", "DOCX → PDF / PNG", false],
  ["embed", "Embed the engine (Rust / Python / wasm)", false],
  ["counsel", "Legal workflow / counsel", false],
];

const body = `<main class="wrap">
<div class="contact-grid m-stack">
<div>
<p class="eyebrow">/ CONTACT</p>
<h1 class="h1">Let’s solve <span class="hl">your document problem.</span></h1>
<p class="lead">Tell us what you compare, how often, and where the output has to land. We’ll say honestly whether Jubarte is the right answer — and if it is, how to wire it in.</p>
<div class="cells">
<div class="cell-pad"><p class="kicker">Product &amp; engine</p><a class="contact-mail" href="mailto:${SUPPORT_EMAIL}">${SUPPORT_EMAIL}</a><p class="small mt-12">Mac app, CLI, crate, wheel, wasm. Bugs, Word-parity gaps, release questions.</p></div>
<div class="cell-pad"><p class="kicker">Workflows &amp; counsel</p><a class="contact-mail" href="mailto:${COUNSEL_EMAIL}">${COUNSEL_EMAIL}</a><p class="small mt-12">Legal departments, firms, legal-tech teams; contracts, governance, US / Brazil.</p></div>
</div>
<p class="hero-meta"><span>${COMPANY}</span><span>New York · São Paulo</span><a href="${ENGINE_REPO}">GitHub ↗</a></p>
</div>

<div class="form-card">
<div class="section-head"><h2>Write to us</h2><span>Opens in your mail app</span></div>
<form id="contact-form" class="form-body" novalidate data-support="${SUPPORT_EMAIL}" data-counsel="${COUNSEL_EMAIL}">
<div class="two-col m-stack">
<label class="form-field"><span>Name</span><input class="input" type="text" id="c-name" autocomplete="name" placeholder="Ana Lima"></label>
<label class="form-field"><span>Work email</span><input class="input" type="email" id="c-email" autocomplete="email" placeholder="ana@firm.com"></label>
</div>
<label class="form-field"><span>Organisation</span><input class="input" type="text" id="c-org" autocomplete="organization" placeholder="Firm, legal department, product team…"></label>
<fieldset class="form-field"><legend>What are you after</legend>
<div class="topics">${TOPICS.map(([k, l, on]) => `<label class="topic"><input type="checkbox" name="topic" value="${k}" data-label="${l}"${on ? " checked" : ""}><span>${l}</span></label>`).join("")}</div>
</fieldset>
<label class="form-field"><span>Volume</span><select class="select" id="c-volume">
<option value="a few a month">A few documents a month</option>
<option value="dozens a week">Dozens a week</option>
<option value="hundreds a day, automated">Hundreds a day, automated</option>
<option value="not sure yet">Not sure yet</option>
</select></label>
<label class="form-field"><span>Message</span><textarea class="textarea" id="c-msg" rows="5" placeholder="What you compare, where the redline has to end up, anything that has bitten you with other tools."></textarea></label>
<div class="row m-col">
<a class="btn btn-primary btn-lg" id="c-send" href="mailto:${SUPPORT_EMAIL}">Send →</a>
<span class="small">Goes to <span class="ink-strong" id="c-to">${SUPPORT_EMAIL}</span>. No documents, please — just the story.</span>
</div>
</form>
<div class="sent" id="c-sent" hidden>
<p class="note-title">Your mail app should have the draft.</p>
<p class="small mt-12">If it did not open, write to <span class="ink-strong" id="c-to-sent">${SUPPORT_EMAIL}</span> yourself. We answer within two business days. <button type="button" class="btn-text" id="c-again">Write another</button></p>
</div>
</div>
</div>
</main>`;

export const contact: Page = {
  file: "contact.html",
  path: "/contact",
  title: "Contact · Jubarte",
  description:
    "Tell us what you compare, how often and where the redline has to land. Product and engine questions, legal workflows and counsel.",
  nav: "contact",
  body,
  scripts: ["contact.js"],
  footer: [
    ["Privacy", "/privacy"],
    ["Terms", "/terms"],
    ["arthur.law", "https://arthur.law"],
  ],
};
