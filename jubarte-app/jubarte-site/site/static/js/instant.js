// Small documents preview the moment they are in, as the Mac app does
// (Settings › Preview there). The page's checkbox turns it off; the choice is
// kept for the Demo and the App page alike. The size comes from
// data/facts.jsonl (app.instant_preview_max_bytes), written into the checkbox.

export const KEY = "jb-instant";

/**
 * True when every document is under `limit` bytes.
 * @param {number[]} sizes
 * @param {number} limit
 */
export const smallEnough = (sizes, limit) =>
  sizes.length > 0 && limit > 0 && sizes.every((size) => size < limit);

/**
 * Wires the page's checkboxes (one per panel, kept in step) and answers, for
 * the documents just chosen (their sizes), whether to preview them now.
 * @param {HTMLInputElement[]} boxes
 * @returns {(sizes: number[]) => boolean}
 */
export function instantPreview(...boxes) {
  let on = true;
  try {
    on = localStorage.getItem(KEY) !== "off";
  } catch {
    /* storage blocked: on, as the app starts */
  }
  for (const box of boxes) {
    box.checked = on;
    box.addEventListener("change", () => {
      on = box.checked;
      for (const other of boxes) other.checked = on;
      try {
        localStorage.setItem(KEY, on ? "on" : "off");
      } catch {
        /* storage blocked: the choice lasts for this page */
      }
    });
  }
  const limit = Number(boxes[0]?.dataset.limit) || 0;
  return (sizes) => on && smallEnough(sizes, limit);
}
