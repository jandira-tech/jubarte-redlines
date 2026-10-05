// Small helpers every page shares. Plain ES module; format.js holds the pure ones.

import { nameParts } from "./format.js";

export { size } from "./format.js";

/** @param {string} id */
export const $ = (id) => /** @type {HTMLElement} */ (document.getElementById(id));

/** A file name as nodes with a break chance after each "_", "-" and ".". @param {string} name */
export function fileName(name) {
  return nameParts(name).flatMap((part, i) => (i ? [document.createElement("wbr"), part] : [part]));
}

/** @param {number} ms */
export function date(ms) {
  return new Date(ms).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}

/** @param {number} ms */
export function seconds(ms) {
  return ms < 1000 ? `${Math.max(1, Math.round(ms))} ms` : `${(ms / 1000).toFixed(1)} s`;
}

/**
 * A toast that hides itself. The element is a role=status live region that
 * stays in the accessibility tree (class "off" fades it), so a screen reader
 * hears each change of its text.
 */
export function toaster(/** @type {HTMLElement} */ el, ttl = 3800) {
  /** @type {ReturnType<typeof setTimeout> | undefined} */
  let timer;
  return (/** @type {string} */ text, kind = "info") => {
    el.textContent = text;
    el.dataset.kind = kind;
    el.classList.remove("off");
    clearTimeout(timer);
    timer = setTimeout(
      () => {
        el.classList.add("off");
        el.textContent = "";
      },
      kind === "error" ? ttl * 2 : ttl,
    );
  };
}

/**
 * Save bytes under a file name through a temporary object URL.
 * @param {Uint8Array} bytes
 * @param {string} name
 * @param {string} type
 */
export function download(bytes, name, type) {
  // The engine's buffers are never shared, so they are valid Blob parts.
  const part = /** @type {Uint8Array<ArrayBuffer>} */ (bytes);
  const url = URL.createObjectURL(new Blob([part], { type }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 30_000);
}

export const DOCX_TYPE = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

/** `.docx` files only, ignoring Word's `~$` owner lock files. @param {Iterable<File>} files */
export function docxOnly(files) {
  return [...files].filter((f) => /\.docx$/i.test(f.name) && !f.name.startsWith("~$"));
}

/** Open the file picker; resolves with the chosen files (possibly none). */
export function pickFiles(multiple = true) {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".docx,application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    input.multiple = multiple;
    input.addEventListener("change", () => resolve([...(input.files ?? [])]));
    input.addEventListener("cancel", () => resolve([]));
    input.click();
  });
}

/**
 * Make an element a drop target: highlights while dragging over it and hands
 * the dropped files to `onFiles`.
 */
export function dropTarget(
  /** @type {HTMLElement} */ el,
  /** @type {(files: File[]) => void} */ onFiles,
) {
  let depth = 0;
  el.addEventListener("dragenter", (e) => {
    e.preventDefault();
    depth++;
    el.classList.add("over");
  });
  el.addEventListener("dragover", (e) => e.preventDefault());
  el.addEventListener("dragleave", () => {
    if (--depth <= 0) {
      depth = 0;
      el.classList.remove("over");
    }
  });
  el.addEventListener("drop", (e) => {
    e.preventDefault();
    e.stopPropagation();
    depth = 0;
    el.classList.remove("over");
    onFiles([...(e.dataTransfer?.files ?? [])]);
  });
}

/** Stop the browser from opening a file dropped beside a target. */
export function swallowStrayDrops() {
  for (const type of ["dragover", "drop"]) {
    window.addEventListener(type, (e) => e.preventDefault());
  }
}

/** True when typing in a field, so single-key shortcuts stay out of the way. */
export function typing(/** @type {Event} */ e) {
  const t = /** @type {HTMLElement | null} */ (e.target);
  return !!t && (/^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName) || t.isContentEditable);
}

// The mobile menu closes when a link in it is followed, on Escape, or on a
// press outside it.
const menu = document.querySelector("details.nav-menu");
if (menu) {
  menu.addEventListener("click", (e) => {
    if (/** @type {HTMLElement} */ (e.target).closest("a")) menu.removeAttribute("open");
  });
  document.addEventListener("keydown", (e) => {
    if (e.key !== "Escape" || !menu.hasAttribute("open")) return;
    menu.removeAttribute("open");
    // The link that had focus is hidden now: focus goes back to the button.
    menu.querySelector("summary")?.focus();
  });
  document.addEventListener("pointerdown", (e) => {
    if (!menu.contains(/** @type {Node} */ (e.target))) menu.removeAttribute("open");
  });
}
