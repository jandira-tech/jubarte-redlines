// The site's reader of data/facts.jsonl, the append-only log of every value the
// site and the app print that can change. scripts/facts.py is its only writer:
// each line is { id: uuidv7, ts, key, value, source }, and a key's value is its
// latest record (uuidv7 ids sort by time); a null value retires the key.

import { readFileSync } from "node:fs";

/** One line of the log. */
export type Fact = { id: string; ts: string; key: string; value: unknown; source: string };

/** A section of a legal page: its place, its heading (none for the opening), its HTML. */
export type Section = { order: number; heading: string | null; html: string };

const LOG = new URL("../../../data/facts.jsonl", import.meta.url);

/** Each live key's latest value. */
export function fold(text: string): Map<string, unknown> {
  const records = text
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line) as Fact);
  // Lowercase uuidv7 strings sort as their timestamps do.
  records.sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  const out = new Map<string, unknown>();
  for (const r of records) {
    if (r.value === null) out.delete(r.key);
    else out.set(r.key, r.value);
  }
  return out;
}

const FACTS = fold(readFileSync(LOG, "utf8"));

/** The value of `key`; a missing key fails the build rather than print a blank. */
export function fact<T>(key: string): T {
  if (!FACTS.has(key)) throw new Error(`data/facts.jsonl has no ${key}`);
  return FACTS.get(key) as T;
}

const esc = (s: string) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

/** `text` with each `{{key}}` replaced by that fact, escaped for HTML. */
export function fill(text: string): string {
  return text.replace(/\{\{([a-z0-9_.-]+)\}\}/g, (_, key: string) => esc(String(fact(key))));
}

/** The sections filed under `prefix` (e.g. "legal.terms"), in order. */
export function sections(prefix: string): Section[] {
  return [...FACTS]
    .filter(([key]) => key.startsWith(`${prefix}.`))
    .map(([, value]) => value as Section)
    .sort((a, b) => a.order - b.order);
}
