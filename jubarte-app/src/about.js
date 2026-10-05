// The About window and the Terms of Use and Privacy Policy, read in the app.
// Their words come from data/facts.jsonl compiled into the app (the `about`
// and `legal_document` commands in src-tauri/src/facts.rs), the same records
// jubarte.pro prints, so the app shows them offline and never a stale copy.

const invoke =
  globalThis.window?.__TAURI__?.core?.invoke ||
  ((cmd) => Promise.reject(new Error(`Tauri core invoke is not available (command: ${cmd})`)));

const $ = (id) => document.getElementById(id);

/** Opens a web or mail address outside the app (`open` on macOS). */
export const openOutside = (url) => invoke("open_path", { path: url }).catch(() => {});

/** True for an address the app hands to the browser or the mail app. */
export const isOutside = (href) => /^(https?:|mailto:)/i.test(href ?? "");

// A link inside these windows leaves the app instead of replacing it.
function keepLinksOutside(root) {
  root.addEventListener("click", (e) => {
    const a = /** @type {HTMLElement} */ (e.target).closest("a[href]");
    if (!a) return;
    const href = a.getAttribute("href");
    if (a.dataset.legal) {
      e.preventDefault();
      openLegal(a.dataset.legal);
    } else if (isOutside(href)) {
      e.preventDefault();
      openOutside(href);
    }
  });
}

let about = null;
async function openAbout() {
  const dialog = $("about");
  if (!dialog || dialog.open) return;
  about ??= await invoke("about").catch(() => null);
  if (about) {
    $("about-version").textContent = `Version ${about.version}`;
    $("about-engine").textContent = `jubarte-redlines ${about.engine}`;
    $("about-engine").href = about.engine_repo;
    $("about-site").href = about.site;
    $("about-site").textContent = about.site.replace(/^https:\/\//, "");
    $("about-support").href = `mailto:${about.support_email}`;
    $("about-support").textContent = about.support_email;
    $("about-copyright").textContent = `${about.copyright}. All rights reserved.`;
  }
  $("about-pro").hidden = !window.jubarte?.entitled;
  dialog.showModal();
}

const docs = new Map();
async function openLegal(doc) {
  const dialog = $("legal");
  if (!dialog) return;
  if (!docs.has(doc)) {
    const d = await invoke("legal_document", { doc }).catch(() => null);
    if (!d) {
      // No compiled copy to show (a dev build without Tauri): the site has it.
      openOutside(`https://jubarte.pro/${doc}`);
      return;
    }
    docs.set(doc, d);
  }
  const d = docs.get(doc);
  $("legal-title").textContent = d.title;
  $("legal-updated").textContent = `Last updated: ${d.updated}`;
  $("legal-web").href = d.url;
  $("legal-web").textContent = d.url.replace(/^https:\/\//, "");
  // Our own words, compiled into the app: not page or user input.
  $("legal-body").innerHTML = d.sections
    .map((s) => (s.heading ? `<h3>${s.heading}</h3>\n${s.html}` : s.html))
    .join("\n");
  $("legal-body").scrollTop = 0;
  if (!dialog.open) dialog.showModal();
}

if (typeof document !== "undefined" && $("about") && $("legal")) {
  keepLinksOutside($("about"));
  keepLinksOutside($("legal"));
  window.jubarteAbout = { open: openAbout, legal: openLegal };
}
