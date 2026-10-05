// Settings: the tracked-change marks the preview and PDFs use, the default
// "Revisions by" name, and the window's appearance. Stored in localStorage;
// app.js reads them through window.jubarteSettings. The pure functions are
// exported for the tests (jubarte-site/test/node/app-settings.test.ts).

export const STORAGE_KEY = "jb-settings";
export const KINDS = ["inserted", "deleted", "movedFrom", "movedTo"];
export const LINES = ["underline", "double-underline", "strike", "double-strike", "plain"];
const MARKS = ["conventional", "word", "custom"];
const APPEARANCES = ["system", "light", "dark"];

/** The engine's `conventional` palette: Litera Compare's marks. */
export const CONVENTIONAL = {
  inserted: { color: "#0000FF", line: "underline" },
  deleted: { color: "#FF0000", line: "strike" },
  movedFrom: { color: "#008000", line: "double-strike" },
  movedTo: { color: "#008000", line: "double-underline" },
};

/** A Word custom document property's text holds at most 255 characters. */
export const MAX_TEXT = 255;

const HEX = /^#[0-9a-f]{6}$/i;

/**
 * Stored settings, with anything missing or invalid replaced by its default.
 * @param {string | null | undefined} raw
 */
export function readSettings(raw) {
  let stored;
  try {
    stored = JSON.parse(raw ?? "{}");
  } catch {
    stored = {};
  }
  if (!stored || typeof stored !== "object" || Array.isArray(stored)) stored = {};
  /** @type {Record<string, { color: string, line: string }>} */
  const palette = {};
  for (const kind of KINDS) {
    const mark = stored.palette?.[kind] ?? {};
    palette[kind] = {
      color: HEX.test(mark.color) ? mark.color.toUpperCase() : CONVENTIONAL[kind].color,
      line: LINES.includes(mark.line) ? mark.line : CONVENTIONAL[kind].line,
    };
  }
  const name = typeof stored.author?.name === "string" ? stored.author.name.trim().slice(0, MAX_TEXT) : "";
  return {
    fingerprint: typeof stored.fingerprint === "string" ? cleanText(stored.fingerprint) : "",
    marks: MARKS.includes(stored.marks) ? stored.marks : "conventional",
    palette,
    author: { mode: stored.author?.mode === "fixed" && name ? "fixed" : "document", name },
    appearance: APPEARANCES.includes(stored.appearance) ? stored.appearance : "system",
    instantPreview: stored.instantPreview !== false,
  };
}

/**
 * True when every chosen document is under `limit` bytes: small enough to
 * preview the moment it is chosen (a preview spends no free use).
 * @param {number[]} sizes
 * @param {number} limit
 */
export const smallEnough = (sizes, limit) =>
  sizes.length > 0 && limit > 0 && sizes.every((size) => size < limit);

/**
 * Text as a custom document property holds it: one line, trimmed, at most
 * MAX_TEXT characters (src-tauri/src/fingerprint.rs `clean` does the same).
 * @param {string} text
 */
export function cleanText(text) {
  const line = Array.from(text.replace(/\s/g, " "))
    .filter((c) => !/[\p{Cc}\uFFFE\uFFFF]/u.test(c))
    .join("");
  return Array.from(line.trim()).slice(0, MAX_TEXT).join("");
}

const SPEC_KIND = { inserted: "inserted", deleted: "deleted", movedFrom: "moved-from", movedTo: "moved-to" };

/**
 * The engine's `--revision-palette` spec (`RevisionPalette::parse`).
 * @param {ReturnType<typeof readSettings>["palette"]} palette
 */
export const paletteSpec = (palette) =>
  KINDS.map((kind) => `${SPEC_KIND[kind]}=${palette[kind].color}:${palette[kind].line}`).join(",");

/**
 * What `convert_document` is called with.
 * @param {ReturnType<typeof readSettings>} settings
 */
export function convertArgs(settings) {
  return settings.marks === "custom"
    ? { revisions: "custom", revisionPalette: paletteSpec(settings.palette) }
    : { revisions: settings.marks, revisionPalette: null };
}

const DECORATION = {
  underline: "underline",
  "double-underline": "underline double",
  strike: "line-through",
  "double-strike": "line-through double",
  plain: "none",
};
const VAR = { inserted: "ins", deleted: "del", movedFrom: "movfrom", movedTo: "movto" };
/** Word's own names for the lines (Track Changes Options). */
export const LINE_LABEL = {
  underline: "Underline",
  "double-underline": "Double underline",
  strike: "Strikethrough",
  "double-strike": "Double strikethrough",
  plain: "No line",
};

/** What each appearance does: the window follows it, the pages never do. */
export function appearanceNote(appearance) {
  if (appearance === "dark") return "A dark window. Document pages stay white, as Word prints them.";
  if (appearance === "light") return "A light window, whatever macOS uses.";
  return "Follows macOS. Document pages stay white, as Word prints them.";
}

/**
 * The CSS custom properties that draw custom marks in the preview, the chips
 * and the sample. Empty for the presets: the stylesheet draws those. The paper
 * is always light, so a colour is used as given there; on a dark window it is
 * lightened enough to read.
 * @param {ReturnType<typeof readSettings>} settings
 */
export function markVars(settings) {
  if (settings.marks !== "custom") return {};
  /** @type {Record<string, string>} */
  const vars = {};
  for (const kind of KINDS) {
    const { color, line } = settings.palette[kind];
    const v = VAR[kind];
    vars[`--${v}`] = `light-dark(${color}, color-mix(in oklch, ${color} 45%, white))`;
    vars[`--${v}-bg`] = `color-mix(in oklch, ${color} 14%, transparent)`;
    vars[`--${v}-mark`] = DECORATION[line];
  }
  return vars;
}

const ALL_VARS = KINDS.flatMap((kind) => ["", "-bg", "-mark"].map((s) => `--${VAR[kind]}${s}`));

/**
 * A size as Finder shows it: 1,000,000 bytes is "1 MB".
 * @param {number} bytes
 */
export const sizeLabel = (bytes) =>
  bytes >= 1_000_000 ? `${+(bytes / 1_000_000).toFixed(1)} MB` : `${Math.round(bytes / 1000)} KB`;

/* ---------- the window ---------- */

function init() {
  const $ = (id) => document.getElementById(id);
  const dialog = /** @type {HTMLDialogElement} */ ($("settings"));
  const instantToggle = /** @type {HTMLInputElement} */ ($("instant-toggle"));
  let settings = readSettings(localStorage.getItem(STORAGE_KEY));
  // Each kind's lines: a row of "Aa", each drawn with the line it puts on text.
  for (const group of dialog.querySelectorAll(".lines[data-lines]")) {
    const kind = group.dataset.lines;
    group.append(
      ...LINES.map((line) => {
        const choice = document.createElement("label");
        choice.className = "line";
        choice.title = LINE_LABEL[line];
        const input = document.createElement("input");
        Object.assign(input, { type: "radio", name: `line-${kind}`, value: line });
        input.dataset.kind = kind;
        input.setAttribute("aria-label", LINE_LABEL[line]);
        const sample = document.createElement("span");
        sample.textContent = "Aa";
        sample.style.textDecoration = DECORATION[line];
        choice.append(input, sample);
        return choice;
      }),
    );
  }

  const apply = () => {
    const root = document.documentElement;
    for (const name of ALL_VARS) root.style.removeProperty(name);
    for (const [name, value] of Object.entries(markVars(settings))) root.style.setProperty(name, value);
    root.style.colorScheme = settings.appearance === "system" ? "" : settings.appearance;
    // The panel's footer holds the same switch as Settings › Preview.
    if (instantToggle) instantToggle.checked = settings.instantPreview;
  };
  // app.js marks a shown PDF stale on this event.
  const marksChanged = () => window.dispatchEvent(new Event("jb-marks"));
  const save = () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    apply();
  };

  // The form shows the stored settings.
  const fill = () => {
    dialog.querySelector(`input[name="marks"][value="${settings.marks}"]`).checked = true;
    $("palette").hidden = settings.marks !== "custom";
    for (const kind of KINDS) {
      dialog.querySelector(`input[type="color"][data-kind="${kind}"]`).value = settings.palette[kind].color.toLowerCase();
      dialog.querySelector(`input[name="line-${kind}"][value="${settings.palette[kind].line}"]`).checked = true;
    }
    dialog.querySelector(`input[name="author-mode"][value="${settings.author.mode}"]`).checked = true;
    $("author-name").value = settings.author.name;
    $("fingerprint").value = settings.fingerprint;
    dialog.querySelector(`input[name="appearance"][value="${settings.appearance}"]`).checked = true;
    $("appearance-note").textContent = appearanceNote(settings.appearance);
    $("instant-preview").checked = settings.instantPreview;
  };

  dialog.addEventListener("input", (e) => {
    const el = /** @type {HTMLInputElement} */ (e.target);
    if (el.name === "marks") {
      settings.marks = el.value;
      $("palette").hidden = settings.marks !== "custom";
      save();
      marksChanged();
    } else if (el.dataset.kind) {
      const mark = settings.palette[el.dataset.kind];
      if (el.type === "color") mark.color = el.value.toUpperCase();
      else mark.line = el.value;
      save();
      marksChanged();
    } else if (el.name === "author-mode" || el.id === "author-name") {
      const name = $("author-name").value.trim().slice(0, MAX_TEXT);
      const fixed = dialog.querySelector('input[name="author-mode"][value="fixed"]').checked;
      // Typing a name is choosing it.
      if (el.id === "author-name" && name) dialog.querySelector('input[name="author-mode"][value="fixed"]').checked = true;
      settings.author = { mode: (fixed || el.id === "author-name") && name ? "fixed" : "document", name };
      save();
      window.jubarteRefreshAuthor?.();
    } else if (el.id === "fingerprint") {
      settings.fingerprint = cleanText(el.value);
      save();
    } else if (el.name === "appearance") {
      settings.appearance = el.value;
      $("appearance-note").textContent = appearanceNote(settings.appearance);
      save();
    } else if (el.id === "instant-preview") {
      settings.instantPreview = /** @type {HTMLInputElement} */ (el).checked;
      save();
    }
  });

  // The panel's footer: Instant preview, the same setting. Turned on with
  // the documents already in, the preview starts.
  instantToggle?.addEventListener("change", () => {
    settings.instantPreview = instantToggle.checked;
    save();
    if (dialog.open) fill();
    if (settings.instantPreview) window.jubartePreviewNow?.();
  });

  $("settings-reset").addEventListener("click", () => {
    settings = readSettings(null);
    save();
    fill();
    marksChanged();
    window.jubarteRefreshAuthor?.();
  });

  const open = () => {
    if (dialog.open) return;
    fill();
    dialog.showModal();
  };
  $("open-settings").addEventListener("click", open);
  // ⌘, opens Settings, as in every Mac app.
  document.addEventListener("keydown", (e) => {
    if (e.key === "," && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      open();
    }
  });

  // The size under which a document previews at once (data/facts.jsonl,
  // compiled into the app). Unknown, nothing previews on its own.
  let instantLimit = 0;
  window.__TAURI__?.core
    ?.invoke("instant_preview_limit")
    .then((n) => {
      instantLimit = Number(n) || 0;
      $("instant-limit").textContent = sizeLabel(instantLimit);
    })
    .catch(() => {});

  window.jubarteSettings = {
    open,
    /** app.js: preview these documents (their sizes) the moment they are chosen? */
    previewAtOnce: (/** @type {number[]} */ sizes) => settings.instantPreview && smallEnough(sizes, instantLimit),
    /** The menu bar's View › Appearance. */
    setAppearance(value) {
      if (!APPEARANCES.includes(value)) return;
      settings.appearance = value;
      save();
      if (dialog.open) fill();
    },
    convertArgs: () => convertArgs(settings),
    fixedAuthor: () => (settings.author.mode === "fixed" ? settings.author.name : ""),
    fingerprint: () => settings.fingerprint,
  };
  apply();
}

if (typeof document !== "undefined" && document.getElementById("settings")) init();
