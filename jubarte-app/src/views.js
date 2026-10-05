// The redline's two views, and the name its PDF gets. Pure, so the tests
// (jubarte-site/test/node/app-redline-views.test.ts) import it; app.js reads
// it as window.jubarteViews.

const ADDED = new Set(["ins", "moveins"]);

/**
 * The original document, rebuilt from the redline's preview: insertions (and
 * the landing side of a move) taken out, deletions kept as plain text. A
 * paragraph the modified document added is left out; a blank one stays.
 * @param {{ runs: { kind: string, text: string }[] }[]} paragraphs
 */
export function originalParagraphs(paragraphs) {
  return paragraphs.flatMap((p) => {
    if (!p.runs.length) return [p];
    const runs = p.runs.filter((r) => !ADDED.has(r.kind)).map((r) => ({ kind: "same", text: r.text }));
    return runs.length ? [{ ...p, runs }] : [];
  });
}

/**
 * A redline's PDF takes the redline's name.
 * @param {string} name
 */
export const pdfNameFor = (name) => `${name.replace(/\.docx$/i, "")}.pdf`;

if (typeof window !== "undefined") window.jubarteViews = { originalParagraphs, pdfNameFor };
