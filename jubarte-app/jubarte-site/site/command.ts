import { esc } from "./layout.ts";

/**
 * A shell command, escaped, with each word kept whole: a line wraps between
 * words, never at the hyphen inside `jubarte-redlines` (site.css .tok). Copying
 * it still gives the plain command.
 */
export function command(text: string): string {
  return text
    .split(/(\s+)/)
    .map((part) => (/\S/.test(part) ? `<span class="tok">${esc(part)}</span>` : part))
    .join("");
}
