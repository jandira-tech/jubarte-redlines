// The Privacy Policy and Terms of Use. Each section is an item of
// data/facts.jsonl (legal.privacy.*, legal.terms.*), shared with the Mac app;
// `{{key}}` in a section is that fact. The app sections are the policy the App
// Store listing links to (first published 2026-07-15); "This website" covers
// what jubarte.pro itself does. Keep both factual to the code: the worker in
// src/index.ts and the app's verify-worker.

import { fact, fill, sections } from "../data/facts.ts";
import { COMPANY, SUPPORT_EMAIL } from "../data/release.ts";
import type { Page } from "../layout.ts";

const UPDATED = new Date(`${fact<string>("legal.updated")}T00:00:00Z`).toLocaleDateString("en-US", {
  timeZone: "UTC",
  year: "numeric",
  month: "long",
  day: "numeric",
});
const MAIL = `<a href="mailto:${SUPPORT_EMAIL}">${SUPPORT_EMAIL}</a>`;

function legal(eyebrow: string, title: string, sections: string): string {
  return `<main class="wrap legal">
<p class="eyebrow">/ ${eyebrow}</p>
<h1 class="h1">${title}</h1>
<p class="small mt-12">Last updated: ${UPDATED}</p>
${sections}
<h2>Contact</h2>
<p>${COMPANY} · ${MAIL}</p>
</main>`;
}

/** A legal page's sections, from data/facts.jsonl (legal.<doc>.<section>). */
const body = (doc: "privacy" | "terms") =>
  `\n${sections(`legal.${doc}`)
    .map((x) => fill(x.heading === null ? x.html : `<h2>${x.heading}</h2>\n${x.html}`))
    .join("\n\n")}\n`;

const PRIVACY = body("privacy");
const TERMS = body("terms");

export const privacy: Page = {
  file: "privacy.html",
  path: "/privacy",
  title: "Privacy Policy · Jubarte",
  description:
    "Jubarte compares documents on your Mac and in your browser; their contents are never uploaded. What the app’s subscription check and this website do and don’t collect.",
  nav: null,
  body: legal("PRIVACY POLICY", "Privacy Policy", PRIVACY),
};

export const terms: Page = {
  file: "terms.html",
  path: "/terms",
  title: "Terms of Use · Jubarte",
  description: "The terms for the Jubarte Mac app, its subscription and the jubarte.pro website.",
  nav: null,
  body: legal("TERMS OF USE", "Terms of Use", TERMS),
};

export const notFound: Page = {
  file: "404.html",
  path: "/404",
  title: "Not found · Jubarte",
  description: "This page does not exist.",
  nav: null,
  noindex: true,
  body: `<div class="page-bg"><main class="wrap limit-page">
<p class="eyebrow">/ 404</p>
<h1 class="h1">This page swam off.</h1>
<p class="lead">Nothing lives at this address. Old links from the design previews redirect; this one did not match any.</p>
<div class="row mt-28 m-col"><a class="btn btn-primary" href="/">Home →</a><a class="btn btn-outline" href="/demo">Try the demo</a><a class="btn btn-text" href="/use-cases">Browse use cases</a></div>
</main></div>`,
};
