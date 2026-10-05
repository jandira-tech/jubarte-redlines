// The contact form drafts an email in the visitor's own mail app; nothing is
// posted anywhere.

import { $ } from "./common.js";

// The addresses come from the page (site/data/release.ts), so there is one copy.
const form = $("contact-form");
const SUPPORT = form.dataset.support ?? "";
const COUNSEL = form.dataset.counsel ?? SUPPORT;

const field = (/** @type {string} */ id) => /** @type {HTMLInputElement} */ ($(id)).value.trim();
const topics = () =>
  [...document.querySelectorAll('input[name="topic"]')]
    .map((x) => /** @type {HTMLInputElement} */ (x))
    .filter((x) => x.checked);

/** Counsel questions go to counsel unless the engine is also on the table. */
export function recipient(/** @type {string[]} */ picked) {
  return picked.includes("counsel") && !picked.includes("embed") ? COUNSEL : SUPPORT;
}

function href() {
  const picked = topics();
  const to = recipient(picked.map((x) => x.value));
  const labels = picked.map((x) => x.dataset.label ?? x.value);
  const org = field("c-org");
  const subject = `Jubarte — ${labels[0] ?? "enquiry"}${org ? ` (${org})` : ""}`;
  const body = [
    `Name: ${field("c-name")}`,
    `Email: ${field("c-email")}`,
    `Organisation: ${org}`,
    `Interested in: ${labels.join(", ") || "—"}`,
    `Volume: ${/** @type {HTMLSelectElement} */ ($("c-volume")).value}`,
    "",
    /** @type {HTMLTextAreaElement} */ ($("c-msg")).value,
  ].join("\n");
  $("c-to").textContent = to;
  $("c-to-sent").textContent = to;
  return `mailto:${to}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
}

const send = /** @type {HTMLAnchorElement} */ ($("c-send"));
const refresh = () => {
  send.href = href();
};
form.addEventListener("input", refresh);
form.addEventListener("change", refresh);
form.addEventListener("submit", (e) => e.preventDefault());
send.addEventListener("click", () => {
  refresh();
  setTimeout(() => {
    form.hidden = true;
    const sent = $("c-sent");
    sent.hidden = false;
    // The Send link that had focus is hidden now.
    sent.setAttribute("tabindex", "-1");
    sent.focus();
  }, 400);
});
$("c-again").addEventListener("click", () => {
  /** @type {HTMLTextAreaElement} */ ($("c-msg")).value = "";
  form.hidden = false;
  $("c-sent").hidden = true;
  $("c-msg").focus();
  refresh();
});
refresh();
