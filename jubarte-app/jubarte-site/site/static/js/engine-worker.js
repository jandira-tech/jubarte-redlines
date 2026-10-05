// The engine, off the main thread. One worker per build: `slim` compares,
// `full` also renders PDF (sizes: site/wasm-size.ts). The build is chosen by
// the `build` search parameter of this script's URL; both are the
// jubarte-wasm package, vendored at the version in this site's package.json.

import { restampRevisions } from "./revision-date.js";

const params = new URL(self.location.href).searchParams;
const build = params.get("build") === "full" ? "full" : "slim";
const base = params.get("base") ?? "/vendor/jubarte";

/** @type {Promise<any>} */
const ready = (async () => {
  const mod = await import(`${base}/${build}/jubarte_wasm.js`);
  await mod.default({ module_or_path: `${base}/${build}/jubarte_wasm_bg.wasm` });
  mod.initPanicHook();
  return mod;
})();

/** Every operation the pages ask for; each returns [result, transferables]. */
const OPS = {
  /** @param {any} w */
  async compare(w, { original, modified, author }) {
    const t0 = performance.now();
    const pinned = w.compareDocuments(original, modified, author);
    const ms = performance.now() - t0;
    const revisions = w.getRevisions(pinned);
    const docx = await restampRevisions(pinned);
    return [{ docx, ms, revisions }, [docx.buffer]];
  },
  revisions(w, { docx }) {
    return [w.getRevisions(docx), []];
  },
  changes(w, { docx }) {
    return [w.listChanges(docx), []];
  },
  pdf(w, { docx, compress, revisions, palette }) {
    const t0 = performance.now();
    const pdf = w.docxToPdf(docx, compress, revisions, palette ?? null);
    const ms = performance.now() - t0;
    const pages = w.pdfPageCount(pdf);
    return [{ pdf, ms, pages }, [pdf.buffer]];
  },
};

self.addEventListener("message", async (e) => {
  const { id, op, args } = e.data;
  try {
    const w = await ready;
    const fn = OPS[/** @type {keyof typeof OPS} */ (op)];
    if (!fn) throw new Error(`unknown operation ${op}`);
    const [result, transfer] = await fn(w, args ?? {});
    self.postMessage({ id, ok: true, result }, { transfer });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    self.postMessage({ id, ok: false, error: message });
  }
});

ready.then(
  () => self.postMessage({ id: 0, ok: true, result: "ready" }),
  (err) => self.postMessage({ id: 0, ok: false, error: String(err?.message ?? err) }),
);
