// The redline preview, as the Mac app draws it: walk word/document.xml into
// paragraphs of runs, where text inside w:ins / w:moveTo is inserted and text
// inside w:del / w:moveFrom is deleted. A port of `parse_preview` in the app's
// src-tauri/src/main.rs, including its limits, so both previews agree.

import { readText } from "./zip.js";

export const PREVIEW_MAX_PARAGRAPHS = 3000;
export const PREVIEW_MAX_CHARS = 300_000;

/** @typedef {"same" | "ins" | "del" | "moveins" | "movedel"} RunKind */
/** @typedef {{ kind: RunKind, text: string, author: string | null }} Run */
/**
 * A move shows where it left (its "movedel" runs) and where it landed (its
 * "moveins" runs); nothing labels it.
 * @typedef {{ runs: Run[] }} Paragraph
 * @typedef {{ paragraphs: Paragraph[], truncated: boolean }} Preview
 */

const ENTITIES = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'" };

/** Resolve the five XML entities and numeric character references. */
export function unescapeXml(/** @type {string} */ s) {
  return s.replace(/&(#x[0-9a-f]+|#\d+|amp|lt|gt|quot|apos);/gi, (m, ref) => {
    if (ref[0] === "#") {
      const code =
        ref[1] === "x" || ref[1] === "X" ? parseInt(ref.slice(2), 16) : parseInt(ref.slice(1), 10);
      return Number.isFinite(code) ? String.fromCodePoint(code) : m;
    }
    return ENTITIES[/** @type {keyof typeof ENTITIES} */ (ref.toLowerCase())] ?? m;
  });
}

const local = (/** @type {string} */ name) => name.slice(name.indexOf(":") + 1);

/** @param {string} attrs */
function authorOf(attrs) {
  const m = /\bw:author\s*=\s*(?:"([^"]*)"|'([^']*)')/.exec(attrs);
  return m ? unescapeXml(m[1] ?? m[2]) : null;
}

/** @param {string} xml @returns {Preview} */
export function parsePreview(xml) {
  /** @type {Paragraph[]} */
  const paragraphs = [];
  /** @type {Run[]} */
  let runs = [];
  /** Open revision wrappers, by local name. */
  const depth = { ins: 0, del: 0, moveTo: 0, moveFrom: 0 };
  let inRun = 0;
  let inText = false;
  let chars = 0;
  /** @type {(string | null)[]} */
  const authors = [];

  const kind = () => {
    if (depth.del > 0 || depth.moveFrom > 0) return depth.moveFrom > 0 ? "movedel" : "del";
    if (depth.ins > 0 || depth.moveTo > 0) return depth.moveTo > 0 ? "moveins" : "ins";
    return "same";
  };
  const author = () => {
    for (let i = authors.length - 1; i >= 0; i--) if (authors[i] != null) return authors[i];
    return null;
  };
  const push = (/** @type {string} */ text) => {
    chars += text.length;
    const k = kind();
    const a = author();
    const last = runs[runs.length - 1];
    if (last && last.kind === k && last.author === a) last.text += text;
    else runs.push({ kind: k, text, author: a });
  };

  const re = /<(\/?)([A-Za-z_][\w.-]*(?::[\w.-]+)?)([^>]*?)(\/?)>|<[!?][^>]*>|([^<]+)/g;
  let truncated = false;
  for (let m = re.exec(xml); m; m = re.exec(xml)) {
    if (paragraphs.length >= PREVIEW_MAX_PARAGRAPHS || chars >= PREVIEW_MAX_CHARS) {
      truncated = true;
      break;
    }
    const [, close, name, attrs, selfClose, text] = m;
    if (text !== undefined) {
      if (inText) push(unescapeXml(text));
      continue;
    }
    if (!name) continue;
    const tag = local(name);
    if (selfClose) {
      if (inRun > 0 && tag === "tab") push("\t");
      else if (inRun > 0 && (tag === "br" || tag === "cr")) push("\n");
      else if (tag === "p") paragraphs.push({ runs: [] });
      continue;
    }
    if (tag in depth) {
      const k = /** @type {keyof typeof depth} */ (tag);
      if (close) {
        depth[k]--;
        authors.pop();
      } else {
        depth[k]++;
        authors.push(authorOf(attrs));
      }
    } else if (tag === "r") {
      inRun += close ? -1 : 1;
    } else if (tag === "t" || tag === "delText") {
      inText = !close && inRun > 0;
    } else if (tag === "p" && close) {
      paragraphs.push({ runs });
      runs = [];
    }
  }
  return { paragraphs, truncated };
}

/**
 * The author a redline should carry: `dc:creator`, else `cp:lastModifiedBy`,
 * from docProps/core.xml — trimmed, "" when neither is usable.
 * @param {string} coreXml
 */
export function authorFromCore(coreXml) {
  const pick = (/** @type {string} */ tag) => {
    const m = new RegExp(`<(?:[\\w.-]+:)?${tag}\\b[^>]*>([\\s\\S]*?)</(?:[\\w.-]+:)?${tag}>`).exec(
      coreXml,
    );
    return m ? unescapeXml(m[1]).trim() : "";
  };
  return pick("creator") || pick("lastModifiedBy");
}

/** The modified document's author, read in the browser; "" when absent. */
export async function documentAuthor(/** @type {Uint8Array} */ docx) {
  try {
    const core = await readText(docx, "docProps/core.xml");
    return core ? authorFromCore(core) : "";
  } catch {
    return "";
  }
}

/** The preview of a redline package. */
export async function redlinePreview(/** @type {Uint8Array} */ docx) {
  const xml = await readText(docx, "word/document.xml");
  if (!xml) throw new Error("word/document.xml is missing");
  return parsePreview(xml);
}

const TAG = { ins: "ins", del: "del", moveins: "span", movedel: "span" };

/** Draw a preview into a `.paper` element, as the app does. */
export function drawPreview(/** @type {HTMLElement} */ paper, /** @type {Preview} */ preview) {
  paper.textContent = "";
  const frag = document.createDocumentFragment();
  for (const para of preview.paragraphs) {
    const p = document.createElement("p");
    if (!para.runs.length) p.className = "blank";
    for (const run of para.runs) {
      if (run.kind === "same") {
        p.append(run.text);
        continue;
      }
      const el = document.createElement(TAG[run.kind]);
      if (run.kind === "moveins" || run.kind === "movedel") el.className = run.kind;
      el.textContent = run.text;
      if (run.author) el.title = run.author;
      p.append(el);
    }
    frag.append(p);
  }
  paper.append(frag);
}

/**
 * Count revisions the way the app's chips do, from `getRevisions` JSON
 * (`type`: Inserted / Deleted / Moved / FormatChanged).
 * @param {string} json
 */
export function countRevisions(json) {
  const counts = { inserted: 0, deleted: 0, moved: 0, format: 0 };
  for (const r of JSON.parse(json)) {
    const t = String(r.type ?? r.revisionType ?? "").toLowerCase();
    if (t.startsWith("insert")) counts.inserted++;
    else if (t.startsWith("delet")) counts.deleted++;
    else if (t.startsWith("move")) counts.moved++;
    else if (t.startsWith("format")) counts.format++;
  }
  return counts;
}

/**
 * The revision chips, which double as the legend: each count wears its
 * revision's mark, so it needs no sign. A zero count is plain (no revision, no
 * revision mark), and formatting changes, which are rare, show only when there
 * are some.
 * @param {ReturnType<typeof countRevisions>} counts
 */
export function revisionChips(counts) {
  /** @type {[string, number, string][]} */
  const chips = [
    ["ins", counts.inserted, "inserted"],
    ["del", counts.deleted, "deleted"],
    ["mov", counts.moved, "moved"],
  ];
  if (counts.format > 0) chips.push(["", counts.format, "formatted"]);
  return chips
    .map(([cls, n, label]) => `<span class="tag${n && cls ? ` ${cls}` : ""}">${n} ${label}</span>`)
    .join("");
}
