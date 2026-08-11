/**
 * Compare worker.
 *
 * The redline runs off the main thread: a large DOCX comparison is seconds of
 * solid CPU, and on the main thread it would freeze the page — including the
 * very spinner meant to say "still working". Nothing here talks to the network;
 * the document bytes exist only inside this worker's memory.
 */
import init, { compareDocuments, getRevisions, initPanicHook } from "/vendor/jubarte_wasm.js";

let ready;

/**
 * Instantiate once per worker; concurrent calls share the same promise.
 * A rejected init is not cached: the next call starts over, so a transient
 * fetch/compile failure doesn't poison every comparison until page reload.
 */
function ensureReady() {
  if (!ready) {
    ready = init({ module_or_path: "/vendor/jubarte_wasm_bg.wasm" })
      .then(() => {
        initPanicHook();
      })
      .catch((err) => {
        ready = undefined;
        throw err;
      });
  }
  return ready;
}

self.onmessage = async (e) => {
  const { id, kind, original, modified, author } = e.data;

  try {
    await ensureReady();
    if (kind === "warm") {
      self.postMessage({ id, ok: true, warm: true });
      return;
    }

    const bytes = compareDocuments(
      new Uint8Array(original),
      new Uint8Array(modified),
      author || "Jubarte",
    );

    // Counting revisions off the redline we just produced keeps the summary
    // honest — it describes the file the visitor downloads, not our intent.
    let revisions = [];
    try {
      revisions = JSON.parse(getRevisions(bytes));
    } catch {
      // A summary is a nicety; never fail a good redline over it.
    }

    const buf = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    self.postMessage({ id, ok: true, result: buf, revisions }, [buf]);
  } catch (err) {
    self.postMessage({ id, ok: false, error: String(err?.message ?? err) });
  }
};
