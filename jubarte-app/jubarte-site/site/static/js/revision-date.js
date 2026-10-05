// jubarte-wasm's compareDocuments takes no date, so every revision it writes
// carries the engine's reproducible default, 1970-01-01T00:00:00Z. A redline
// someone opens in Word should say when it was made, as the Mac app's does.

import { rewriteParts } from "./zip.js";

const PINNED = 'w:date="1970-01-01T00:00:00Z"';

/** A moment as Word writes `w:date`: UTC, whole seconds, `Z`. */
export function wordDate(/** @type {Date} */ at = new Date()) {
  return at.toISOString().replace(/\.\d{3}Z$/, "Z");
}

/** The redline with the engine's pinned revision dates set to `date`. */
export function restampRevisions(/** @type {Uint8Array} */ docx, date = wordDate()) {
  const stamp = `w:date="${date}"`;
  return rewriteParts(
    docx,
    (name) => name.startsWith("word/") && name.endsWith(".xml"),
    (_, xml) => (xml.includes(PINNED) ? xml.replaceAll(PINNED, stamp) : null),
  );
}
