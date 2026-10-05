// The keyboard side of a tab set (no DOM), so Node tests can import it.

/**
 * The tab a key selects in a set of `n` tabs where tab `i` has focus, following
 * the APG tabs pattern: the arrows move one tab and wrap, Home and End jump to
 * the ends. Any other key returns null and is left to the browser.
 * @param {string} key a KeyboardEvent.key
 * @param {number} i
 * @param {number} n
 * @returns {number | null}
 */
export function tabKey(key, i, n) {
  if (key === "ArrowRight") return (i + 1) % n;
  if (key === "ArrowLeft") return (i + n - 1) % n;
  if (key === "Home") return 0;
  if (key === "End") return n - 1;
  return null;
}
