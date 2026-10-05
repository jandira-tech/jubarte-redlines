// Pure formatting helpers (no DOM), so Node tests can import them.

/**
 * Bytes as the site prints them: kilobytes with one decimal under 10 KB, so a
 * 1,009-byte and a 1,100-byte file read alike (1.0 KB, 1.1 KB), not "1009 B"
 * beside "1 KB".
 * @param {number} n
 */
export function size(n) {
  if (n < 10_240) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1_048_576) return `${Math.round(n / 1024)} KB`;
  return `${(n / 1_048_576).toFixed(1)} MB`;
}

/**
 * A file name cut after each "_", "-" and ".", the places a long name may wrap,
 * so "NDA_v2_counterparty.docx" breaks between its parts, not inside a word.
 * @param {string} name
 */
export function nameParts(name) {
  return name.split(/(?<=[_.-])/);
}
