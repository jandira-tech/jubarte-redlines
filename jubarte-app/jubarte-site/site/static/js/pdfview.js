// pdf.js, loaded only when a page first draws a PDF: thumbnails of the
// engine's output and the PNG pages the Convert tab offers. Every file comes
// from this site (CSP: script-src 'self').

const BASE = document.documentElement.dataset.pdfjs ?? "/vendor/pdfjs";

/** @type {Promise<any> | undefined} */
let lib;

function pdfjs() {
  lib ??= import(`${BASE}/build/pdf.min.mjs`).then((m) => {
    m.GlobalWorkerOptions.workerSrc = `${BASE}/build/pdf.worker.min.mjs`;
    return m;
  });
  return lib;
}

/** Open PDF bytes (copied: pdf.js detaches what it is given). */
export async function openPdf(/** @type {Uint8Array} */ bytes) {
  const m = await pdfjs();
  return m.getDocument({
    data: bytes.slice(),
    cMapUrl: `${BASE}/cmaps/`,
    cMapPacked: true,
    standardFontDataUrl: `${BASE}/standard_fonts/`,
    wasmUrl: `${BASE}/wasm/`,
    iccUrl: `${BASE}/iccs/`,
    isEvalSupported: false,
  }).promise;
}

/**
 * Draw one page (1-based) at `scale` into a new canvas.
 * @returns {Promise<HTMLCanvasElement>}
 */
export async function drawPage(
  /** @type {any} */ doc,
  /** @type {number} */ n,
  /** @type {number} */ scale,
) {
  const page = await doc.getPage(n);
  const viewport = page.getViewport({ scale });
  const canvas = document.createElement("canvas");
  canvas.width = Math.ceil(viewport.width);
  canvas.height = Math.ceil(viewport.height);
  const ctx = canvas.getContext("2d", { alpha: false });
  if (!ctx) throw new Error("no 2D canvas");
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  await page.render({ canvasContext: ctx, canvas, viewport }).promise;
  page.cleanup();
  return canvas;
}

/** A page as PNG bytes at `dpi` (PDF points are 1/72 in). */
export async function pagePng(
  /** @type {any} */ doc,
  /** @type {number} */ n,
  /** @type {number} */ dpi,
) {
  const canvas = await drawPage(doc, n, dpi / 72);
  const blob = await new Promise((resolve, reject) =>
    canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("PNG encoding failed"))), "image/png"),
  );
  return new Uint8Array(await /** @type {Blob} */ (blob).arrayBuffer());
}

/**
 * Fill `grid` with page thumbnails, `width` CSS pixels wide, drawing them as
 * they scroll into view.
 */
/** The observer drawing each grid's thumbnails; a new set retires the old one. */
const observers = new WeakMap();

export function thumbnails(
  /** @type {any} */ doc,
  /** @type {HTMLElement} */ grid,
  width = 220,
  max = 60,
) {
  observers.get(grid)?.disconnect();
  grid.textContent = "";
  const count = Math.min(doc.numPages, max);
  const ratio = window.devicePixelRatio || 1;
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        io.unobserve(e.target);
        const fig = /** @type {HTMLElement} */ (e.target);
        const n = Number(fig.dataset.page);
        doc
          .getPage(n)
          .then((/** @type {any} */ page) => {
            const scale = (width * ratio) / page.getViewport({ scale: 1 }).width;
            return drawPage(doc, n, scale);
          })
          .then((/** @type {HTMLCanvasElement} */ canvas) => {
            canvas.setAttribute("aria-label", `Page ${n}`);
            fig.querySelector(".thumb")?.replaceChildren(canvas);
          })
          .catch(() => fig.classList.add("failed"));
      }
    },
    { rootMargin: "400px" },
  );
  observers.set(grid, io);
  for (let n = 1; n <= count; n++) {
    const fig = document.createElement("figure");
    fig.className = "page-thumb";
    fig.dataset.page = String(n);
    fig.innerHTML = `<div class="thumb"><span class="spinner"></span></div><figcaption class="mono">p. ${n}</figcaption>`;
    grid.append(fig);
    io.observe(fig);
  }
  if (doc.numPages > count) {
    const more = document.createElement("p");
    more.className = "small";
    more.textContent = `First ${count} of ${doc.numPages} pages shown; the download has them all.`;
    grid.append(more);
  }
}
