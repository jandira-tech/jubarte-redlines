// The page side of engine-worker.js: one lazily started worker per build,
// promise-returning calls, and a "warming" signal so a page can start the
// download the moment the first file arrives.

const ENGINE = document.documentElement.dataset.engine ?? "";

/** @type {Map<string, Engine>} */
const engines = new Map();

class Engine {
  /** @param {"slim" | "full"} build */
  constructor(build) {
    const url = new URL("/static/js/engine-worker.js", location.origin);
    url.searchParams.set("build", build);
    if (ENGINE) url.searchParams.set("base", ENGINE);
    this.worker = new Worker(url, { type: "module", name: `jubarte-${build}` });
    this.seq = 0;
    /** @type {Map<number, { resolve: (v: any) => void, reject: (e: Error) => void }>} */
    this.pending = new Map();
    this.ready = new Promise((resolve, reject) => {
      this.pending.set(0, { resolve, reject });
    });
    // Nobody has to await `ready`; a failed start must not surface as an
    // unhandled rejection on top of the rejected calls.
    this.ready.catch(() => {});
    this.worker.addEventListener("message", (e) => {
      const { id, ok, result, error } = e.data;
      // The worker keeps a failed start for good; start over on the next call.
      if (id === 0 && !ok) return this.fail(build, new Error(error));
      const p = this.pending.get(id);
      if (!p) return;
      this.pending.delete(id);
      if (ok) p.resolve(result);
      else p.reject(new Error(error));
    });
    this.worker.addEventListener("error", (e) =>
      this.fail(build, new Error(e.message || "the engine failed to load")),
    );
  }

  /**
   * Reject every call still waiting, stop the worker, and let the next
   * `engine(build)` start a new one. Calls are rejected here because
   * `terminate()` drops whatever the worker had not answered yet.
   * @param {"slim" | "full"} build
   * @param {Error} err
   */
  fail(build, err) {
    for (const p of this.pending.values()) p.reject(err);
    this.pending.clear();
    this.worker.terminate();
    if (engines.get(build) === this) engines.delete(build);
  }

  /**
   * @param {string} op
   * @param {Record<string, unknown>} args
   * @param {Transferable[]} transfer
   */
  call(op, args, transfer = []) {
    const id = ++this.seq;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ id, op, args }, { transfer });
    });
  }
}

/** The engine for a build, started on first use. @param {"slim" | "full"} build */
export function engine(build) {
  let e = engines.get(build);
  if (!e) {
    e = new Engine(build);
    engines.set(build, e);
  }
  return e;
}

/**
 * Compare two documents. Inputs are copied, so the caller keeps its bytes.
 * @returns {Promise<{ docx: Uint8Array, ms: number, revisions: string }>}
 */
export function compare(
  /** @type {Uint8Array} */ original,
  /** @type {Uint8Array} */ modified,
  /** @type {string} */ author,
) {
  const a = original.slice();
  const b = modified.slice();
  return /** @type {any} */ (
    engine("slim").call("compare", { original: a, modified: b, author }, [a.buffer, b.buffer])
  );
}

/**
 * Render a document to PDF with the full build.
 * @returns {Promise<{ pdf: Uint8Array, ms: number, pages: number }>}
 */
export function toPdf(
  /** @type {Uint8Array} */ docx,
  /** @type {{ compress: boolean, revisions: string, palette?: string }} */ opts,
) {
  const copy = docx.slice();
  return /** @type {any} */ (engine("full").call("pdf", { docx: copy, ...opts }, [copy.buffer]));
}
