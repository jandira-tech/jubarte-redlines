// "Email me the download link" on a phone: a mailto addressed to the visitor.

import { $ } from "./common.js";

const input = /** @type {HTMLInputElement} */ ($("send-email"));
const btn = /** @type {HTMLAnchorElement} */ ($("send-btn"));
const base = btn.href;

input.addEventListener("input", () => {
  const to = input.value.trim().replace(/[\s?&#]/g, "");
  btn.href = base.replace(/^mailto:[^?]*/, `mailto:${to}`);
});
